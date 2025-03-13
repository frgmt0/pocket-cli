use crate::models::{Entry, SearchAlgorithm};
use crate::storage::StorageManager;
use anyhow::{Result, anyhow};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use rust_stemmers::{Algorithm, Stemmer};
use regex::Regex;

/// Result of a search operation
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The entry that matched
    pub entry: Entry,
    
    /// The content of the entry
    pub content: String,
    
    /// The similarity score (0.0 to 1.0)
    pub score: f64,

    /// Backpack name (None for general pocket)
    pub backpack: Option<String>,

    /// Highlighted snippets
    pub highlights: Vec<String>,
}

/// Search engine for finding entries
pub struct SearchEngine {
    storage: StorageManager,
    index: Arc<RwLock<SearchIndex>>,
    stemmer: Stemmer,
    stopwords: HashSet<String>,
}

/// Index structure to speed up searches
struct SearchIndex {
    /// Maps terms to documents
    term_docs: HashMap<String, HashMap<String, f64>>,
    
    /// Maps document IDs to their backpack
    doc_backpack: HashMap<String, Option<String>>,
    
    /// Document frequencies (for IDF calculation)
    doc_frequencies: HashMap<String, usize>,
    
    /// Total document count (for IDF calculation)
    total_docs: usize,
    
    /// Term frequency in each document (for TF calculation)
    term_frequencies: HashMap<String, HashMap<String, usize>>,
    
    /// Average document length (for BM25)
    average_doc_length: f64,
    
    /// Document lengths (for BM25)
    doc_lengths: HashMap<String, usize>,
    
    /// Last time the index was updated
    last_updated: Instant,
    
    /// Whether the index needs a rebuild
    needs_rebuild: bool,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new(storage: StorageManager) -> Self {
        let index = Arc::new(RwLock::new(SearchIndex::new()));
        let stemmer = Stemmer::create(Algorithm::English);
        
        // Common English stopwords
        let stopwords: HashSet<String> = vec![
            "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "with", 
            "by", "about", "as", "of", "from", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would", "shall",
            "should", "can", "could", "may", "might", "must", "this", "that", "these",
            "those", "i", "you", "he", "she", "it", "we", "they", "their", "my", "your",
            "his", "her", "its", "our", "not"
        ].into_iter().map(|s| s.to_string()).collect();
        
        let engine = Self { 
            storage, 
            index,
            stemmer,
            stopwords,
        };
        
        // Initialize the index in the background
        let index_clone = engine.index.clone();
        let storage_clone = engine.storage.clone();
        std::thread::spawn(move || {
            let _ = SearchIndex::build(index_clone, storage_clone);
        });
        
        engine
    }

    /// Search for entries matching a query
    pub fn search(&self, query: &str, limit: usize, backpack: Option<&str>, algorithm: SearchAlgorithm) -> Result<Vec<SearchResult>> {
        // Check if we need to rebuild the index
        {
            let index = self.index.read().map_err(|_| anyhow!("Failed to acquire read lock on search index"))?;
            if index.needs_rebuild || index.last_updated.elapsed() > Duration::from_secs(300) {  // Rebuild every 5 minutes
                // Release the read lock before acquiring the write lock
                drop(index);
                
                // Rebuild the index
                SearchIndex::build(self.index.clone(), self.storage.clone())?;
            }
        }
        
        // Tokenize and process the query
        let processed_query = self.preprocess_text(query);
        
        let index = self.index.read().map_err(|_| anyhow!("Failed to acquire read lock on search index"))?;
        
        // Perform the search based on algorithm
        let mut results = match algorithm {
            SearchAlgorithm::Semantic => {
                // Use BM25 ranking for semantic search
                self.bm25_search(&processed_query, &index, backpack)?
            },
            SearchAlgorithm::Literal => {
                // Use fuzzy matching for literal search
                self.fuzzy_search(query, &index, backpack)?
            }
        };
        
        // Sort by score (highest first)
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal)
        });
        
        // Limit the number of results
        if results.len() > limit {
            results.truncate(limit);
        }
        
        // Generate highlights
        results = self.generate_highlights(results, query)?;
        
        Ok(results)
    }
    
    /// Perform BM25 search
    fn bm25_search(&self, query: &str, index: &SearchIndex, backpack_filter: Option<&str>) -> Result<Vec<SearchResult>> {
        let query_terms = self.tokenize(query);
        let mut scores: HashMap<String, f64> = HashMap::new();
        
        // BM25 parameters
        let k1 = 1.2; // Controls term frequency saturation
        let b = 0.75; // Controls document length normalization
        
        for term in query_terms {
            if self.stopwords.contains(&term) {
                continue;
            }
            
            let stemmed_term = self.stemmer.stem(&term).to_string();
            
            // Skip if term not in index
            if !index.doc_frequencies.contains_key(&stemmed_term) {
                continue;
            }
            
            // Calculate IDF (Inverse Document Frequency)
            let df = *index.doc_frequencies.get(&stemmed_term).unwrap_or(&1);
            let idf = (index.total_docs as f64 - df as f64 + 0.5) / (df as f64 + 0.5);
            let idf = (1.0 + idf).ln();
            
            // For each document containing this term
            if let Some(term_docs) = index.term_docs.get(&stemmed_term) {
                for (doc_id, _) in term_docs {
                    // Skip if not in the requested backpack
                    if let Some(filter) = backpack_filter {
                        if let Some(doc_backpack) = index.doc_backpack.get(doc_id) {
                            if *doc_backpack != Some(filter.to_string()) {
                                continue;
                            }
                        }
                    }
                    
                    // Get term frequency
                    let tf = *index.term_frequencies
                        .get(doc_id)
                        .and_then(|terms| terms.get(&stemmed_term))
                        .unwrap_or(&0);
                    
                    // Get document length
                    let doc_len = *index.doc_lengths.get(doc_id).unwrap_or(&1);
                    
                    // BM25 formula
                    let numerator = tf as f64 * (k1 + 1.0);
                    let denominator = tf as f64 + k1 * (1.0 - b + b * doc_len as f64 / index.average_doc_length);
                    let score = idf * numerator / denominator;
                    
                    *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                }
            }
        }
        
        // Convert scores to results
        let mut results = Vec::new();
        for (doc_id, score) in scores {
            // Normalize score to 0-1 range
            let normalized_score = (score / 10.0).min(1.0);
            
            if normalized_score > 0.1 {  // Minimum threshold
                // Get backpack
                let backpack = index.doc_backpack.get(&doc_id).cloned().unwrap_or(None);
                
                // Load entry and content
                if let Ok((entry, content)) = self.storage.load_entry(&doc_id, backpack.as_deref()) {
                    results.push(SearchResult {
                        entry,
                        content,
                        score: normalized_score,
                        backpack,
                        highlights: Vec::new(),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Perform fuzzy search using n-gram matching
    fn fuzzy_search(&self, query: &str, _index: &SearchIndex, backpack_filter: Option<&str>) -> Result<Vec<SearchResult>> {
        // Get all entries that might match
        let mut entries: Vec<(Entry, String, Option<String>)> = Vec::new();
        
        // If backpack is specified, only search in that backpack
        if let Some(backpack) = backpack_filter {
            let backpack_entries = self.storage.list_entries(Some(backpack))?;
            for entry in backpack_entries {
                let (entry, content) = self.storage.load_entry(&entry.id, Some(backpack))?;
                entries.push((entry, content, Some(backpack.to_string())));
            }
        } else {
            // Get entries from all backpacks
            let backpacks = self.storage.list_backpacks()?;
            for backpack in backpacks {
                let backpack_entries = self.storage.list_entries(Some(&backpack.name))?;
                for entry in backpack_entries {
                    let (entry, content) = self.storage.load_entry(&entry.id, Some(&backpack.name))?;
                    entries.push((entry, content, Some(backpack.name.clone())));
                }
            }
            
            // Also get entries from the general pocket
            let general_entries = self.storage.list_entries(None)?;
            for entry in general_entries {
                let (entry, content) = self.storage.load_entry(&entry.id, None)?;
                entries.push((entry, content, None));
            }
        }
        
        // Calculate fuzzy match scores
        let mut results = Vec::new();
        
        for (entry, content, backpack) in entries {
            // Calculate fuzzy match score
            let score = self.calculate_fuzzy_similarity(query, &content);
            
            if score > 0.2 {  // Higher threshold for fuzzy matching
                results.push(SearchResult {
                    entry,
                    content,
                    score,
                    backpack,
                    highlights: Vec::new(),
                });
            }
        }
        
        Ok(results)
    }

    /// Calculate fuzzy similarity between query and content using n-grams
    fn calculate_fuzzy_similarity(&self, query: &str, content: &str) -> f64 {
        if query.is_empty() || content.is_empty() {
            return 0.0;
        }
        
        // Generate n-grams for query and content (trigrams are common for fuzzy matching)
        let query_ngrams = self.generate_ngrams(query, 3);
        let content_ngrams = self.generate_ngrams(content, 3);
        
        // Calculate Jaccard similarity coefficient
        let intersection: HashSet<_> = query_ngrams.intersection(&content_ngrams).collect();
        let union: HashSet<_> = query_ngrams.union(&content_ngrams).collect();
        
        if union.is_empty() {
            return 0.0;
        }
        
        intersection.len() as f64 / union.len() as f64
    }
    
    /// Generate n-grams from text
    fn generate_ngrams(&self, text: &str, n: usize) -> HashSet<String> {
        let text = text.to_lowercase();
        
        let mut ngrams = HashSet::new();
        if text.len() < n {
            ngrams.insert(text);
            return ngrams;
        }
        
        // Generate character n-grams
        for i in 0..=text.len() - n {
            let ngram: String = text.chars().skip(i).take(n).collect();
            ngrams.insert(ngram);
        }
        
        ngrams
    }
    
    /// Tokenize text into terms
    fn tokenize(&self, text: &str) -> Vec<String> {
        // Regex to extract words
        let word_regex = Regex::new(r"\b[\w']+\b").unwrap();
        
        word_regex.find_iter(text.to_lowercase().as_str())
            .map(|m| m.as_str().to_string())
            .collect()
    }
    
    /// Preprocess text for indexing/searching
    fn preprocess_text(&self, text: &str) -> String {
        // Tokenize
        let tokens = self.tokenize(text);
        
        // Filter out stopwords and apply stemming
        tokens.iter()
            .filter(|token| !self.stopwords.contains(*token))
            .map(|token| self.stemmer.stem(token).to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
    
    /// Generate meaningful highlights for search results
    fn generate_highlights(&self, results: Vec<SearchResult>, query: &str) -> Result<Vec<SearchResult>> {
        let mut highlighted_results = Vec::new();
        
        // Create regex for finding query terms (with word boundaries)
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let regex_pattern = query_terms.iter()
            .map(|term| format!(r"\b{}\b", regex::escape(term)))
            .collect::<Vec<String>>()
            .join("|");
        
        let term_regex = Regex::new(&regex_pattern)
            .map_err(|e| anyhow!("Failed to create regex: {}", e))?;
        
        for result in results {
            let mut highlights = Vec::new();
            let content = &result.content;
            
            // Find best context for each match
            let mut matches = term_regex.find_iter(content).peekable();
            
            if matches.peek().is_none() {
                // No exact matches, find fuzzy matches
                highlights.push(self.get_context_snippet(content, 0, 150));
            } else {
                // Generate snippets around each match, limiting to 3 highlights
                let mut current_pos = 0;
                let mut highlight_count = 0;
                
                for m in term_regex.find_iter(content) {
                    if highlight_count >= 3 {
                        break;
                    }
                    
                    // Skip if too close to previous highlight
                    if m.start() < current_pos + 50 && current_pos > 0 {
                        continue;
                    }
                    
                    // Get snippet context
                    let snippet = self.get_context_snippet(content, m.start(), 150);
                    highlights.push(snippet);
                    
                    current_pos = m.end();
                    highlight_count += 1;
                }
            }
            
            // Create a copy with highlights
            highlighted_results.push(SearchResult {
                entry: result.entry,
                content: result.content,
                score: result.score,
                backpack: result.backpack,
                highlights,
            });
        }
        
        Ok(highlighted_results)
    }
    
    /// Get a context snippet around a position
    fn get_context_snippet(&self, content: &str, position: usize, length: usize) -> String {
        let content_len = content.len();
        
        // Calculate start position
        let start = if position > length / 2 {
            position - length / 2
        } else {
            0
        };
        
        // Find word boundary for start
        let mut start_pos = start;
        while start_pos > 0 && content.chars().nth(start_pos) != Some(' ') {
            start_pos -= 1;
        }
        
        // Calculate end position
        let end = (start_pos + length).min(content_len);
        
        // Find word boundary for end
        let mut end_pos = end;
        while end_pos < content_len && content.chars().nth(end_pos) != Some(' ') {
            end_pos += 1;
        }
        
        // Extract snippet
        let mut result = String::new();
        if start_pos > 0 {
            result.push_str("...");
        }
        
        result.push_str(&content[start_pos..end_pos.min(content_len)]);
        
        if end_pos < content_len {
            result.push_str("...");
        }
        
        result
    }
}

impl SearchIndex {
    /// Create a new empty search index
    fn new() -> Self {
        Self {
            term_docs: HashMap::new(),
            doc_backpack: HashMap::new(),
            doc_frequencies: HashMap::new(),
            total_docs: 0,
            term_frequencies: HashMap::new(),
            average_doc_length: 0.0,
            doc_lengths: HashMap::new(),
            last_updated: Instant::now(),
            needs_rebuild: true,
        }
    }
    
    /// Tokenize content for indexing
    fn tokenize_content(text: &str) -> Vec<String> {
        // Create stemmer and stopwords
        let stemmer = Stemmer::create(Algorithm::English);
        
        // Common English stopwords
        let stopwords: HashSet<String> = vec![
            "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "with", 
            "by", "about", "as", "of", "from", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would", "shall",
            "should", "can", "could", "may", "might", "must", "this", "that", "these",
            "those", "i", "you", "he", "she", "it", "we", "they", "their", "my", "your",
            "his", "her", "its", "our", "not"
        ].into_iter().map(|s| s.to_string()).collect();
        
        // Tokenize the text
        let word_regex = Regex::new(r"\b[\w']+\b").unwrap();
        
        word_regex.find_iter(text.to_lowercase().as_str())
            .map(|m| m.as_str().to_string())
            .filter(|token| !stopwords.contains(token))
            .map(|token| stemmer.stem(&token).to_string())
            .collect()
    }
    
    /// Build or rebuild the search index
    fn build(index: Arc<RwLock<SearchIndex>>, storage: StorageManager) -> Result<()> {
        let mut index_guard = index.write().unwrap();
        
        // Clear the index
        index_guard.term_docs.clear();
        index_guard.doc_backpack.clear();
        index_guard.doc_frequencies.clear();
        index_guard.term_frequencies.clear();
        index_guard.doc_lengths.clear();
        
        // Get all entries from all backpacks
        let backpacks = storage.list_backpacks()?;
        let mut all_entries = Vec::new();
        
        // Add general entries
        for entry in storage.list_entries(None)? {
            all_entries.push((entry, None));
        }
        
        // Add backpack entries
        for backpack in backpacks {
            for entry in storage.list_entries(Some(&backpack.name))? {
                all_entries.push((entry, Some(backpack.name.clone())));
            }
        }
        
        // Process each entry
        let mut total_length = 0;
        for (entry, backpack) in all_entries {
            // Load the entry content
            let content = match storage.load_entry_content(&entry.id, backpack.as_deref()) {
                Ok(content) => content,
                Err(_) => continue, // Skip entries with missing content
            };
            
            // Get summary if available
            let mut summary_text = String::new();
            if let Some(summary_json) = entry.get_metadata("summary") {
                if let Ok(summary) = crate::utils::SummaryMetadata::from_json(summary_json) {
                    summary_text = summary.summary;
                }
            }
            
            // Combine content and summary for indexing, giving summary higher weight
            let combined_text = if !summary_text.is_empty() {
                format!("{} {} {}", summary_text, summary_text, content) // Repeat summary to give it more weight
            } else {
                content.clone()
            };
            
            // Tokenize and process the content
            let tokens = SearchIndex::tokenize_content(&combined_text);
            let doc_length = tokens.len();
            total_length += doc_length;
            
            // Store document length
            index_guard.doc_lengths.insert(entry.id.clone(), doc_length);
            
            // Store backpack info
            index_guard.doc_backpack.insert(entry.id.clone(), backpack.clone());
            
            // Process each token
            let mut term_counts = HashMap::new();
            for token in tokens {
                *term_counts.entry(token.clone()).or_insert(0) += 1;
                
                // Add to term-document index
                let docs = index_guard.term_docs.entry(token.clone()).or_insert_with(HashMap::new);
                docs.insert(entry.id.clone(), 0.0); // Score will be calculated later
            }
            
            // Store term frequencies for this document
            index_guard.term_frequencies.insert(entry.id.clone(), term_counts);
        }
        
        // Calculate average document length
        if !index_guard.doc_lengths.is_empty() {
            index_guard.average_doc_length = total_length as f64 / index_guard.doc_lengths.len() as f64;
        }
        
        // Calculate document frequencies
        let mut doc_frequencies = HashMap::new();
        for (term, docs) in &index_guard.term_docs {
            doc_frequencies.insert(term.clone(), docs.len());
        }
        
        // Update document frequencies
        index_guard.doc_frequencies = doc_frequencies;
        
        // Update total document count
        index_guard.total_docs = index_guard.doc_lengths.len();
        
        // Mark as updated
        index_guard.last_updated = Instant::now();
        index_guard.needs_rebuild = false;
        
        Ok(())
    }
} 