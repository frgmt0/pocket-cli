use crate::models::{Entry, SearchAlgorithm};
use crate::storage::StorageManager;
use anyhow::Result;
use similar::TextDiff;
use std::cmp::Ordering;

/// Result of a search operation
pub struct SearchResult {
    /// The entry that matched
    pub entry: Entry,
    
    /// The content of the entry
    pub content: String,
    
    /// The similarity score (0.0 to 1.0)
    pub score: f64,
}

/// Search engine for finding entries
pub struct SearchEngine {
    storage: StorageManager,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new(storage: StorageManager) -> Self {
        Self { storage }
    }

    /// Search for entries matching a query
    pub fn search(&self, query: &str, limit: usize, backpack: Option<&str>, algorithm: SearchAlgorithm) -> Result<Vec<SearchResult>> {
        // Get all entries from the specified backpack or general pocket
        let entries = self.storage.list_entries(backpack)?;
        
        // If no entries, return empty results
        if entries.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        // Process each entry
        for entry in entries {
            // Load the content
            let (entry, content) = self.storage.load_entry(&entry.id, backpack)?;
            
            // Calculate similarity score based on the algorithm
            let score = match algorithm {
                SearchAlgorithm::Semantic => self.calculate_semantic_similarity(query, &content),
                SearchAlgorithm::Literal => self.calculate_literal_similarity(query, &content),
            };

            // Only include results with a minimum score
            if score > 0.1 {
                results.push(SearchResult {
                    entry,
                    content,
                    score,
                });
            }
        }

        // Sort by score (highest first)
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal)
        });

        // Limit the number of results
        if results.len() > limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Calculate semantic similarity between query and content
    /// 
    /// This is a simple implementation using text similarity.
    /// In a more advanced version, this could use embeddings or other NLP techniques.
    fn calculate_semantic_similarity(&self, query: &str, content: &str) -> f64 {
        // For now, we'll use a simple text similarity algorithm
        // In a real implementation, this would use more advanced NLP techniques
        
        // Convert to lowercase for better matching
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        // Use TextDiff for similarity
        let diff = TextDiff::from_chars(&query_lower, &content_lower);
        let total_changes = diff.iter_all_changes().count() as f64;
        let max_len = query_lower.len().max(content_lower.len()) as f64;
        
        if max_len == 0.0 {
            return 0.0;
        }
        
        // Calculate similarity as 1 - (changes / max_length)
        // This is a simple metric that gives higher scores to texts with fewer changes
        1.0 - (total_changes / (2.0 * max_len)).min(1.0)
    }

    /// Calculate literal similarity (exact text matching)
    fn calculate_literal_similarity(&self, query: &str, content: &str) -> f64 {
        // Simple contains check
        if content.contains(query) {
            1.0
        } else {
            // Check for partial matches
            let query_words: Vec<&str> = query.split_whitespace().collect();
            let mut matches = 0;
            
            for word in &query_words {
                if content.contains(word) {
                    matches += 1;
                }
            }
            
            if query_words.is_empty() {
                0.0
            } else {
                matches as f64 / query_words.len() as f64
            }
        }
    }

    /// Get highlighted content with matching parts emphasized
    pub fn get_highlighted_content(&self, content: &str, query: &str, max_length: usize) -> String {
        // Find the first occurrence of the query or a part of it
        let query_parts: Vec<&str> = query.split_whitespace().collect();
        
        // Try to find the best context to show
        let mut best_pos = 0;
        let mut best_len = 0;
        
        for (i, _) in content.char_indices() {
            let window = &content[i..std::cmp::min(i + 100, content.len())];
            let mut matches = 0;
            
            for part in &query_parts {
                if window.to_lowercase().contains(&part.to_lowercase()) {
                    matches += 1;
                }
            }
            
            if matches > best_len {
                best_len = matches;
                best_pos = i;
            }
        }
        
        // Extract context around the match
        let start = if best_pos > max_length / 2 {
            best_pos - max_length / 2
        } else {
            0
        };
        
        let end = std::cmp::min(start + max_length, content.len());
        let context = &content[start..end];
        
        // Add ellipsis if we're not showing the beginning or end
        let mut result = String::new();
        if start > 0 {
            result.push_str("...");
        }
        result.push_str(context);
        if end < content.len() {
            result.push_str("...");
        }
        
        result
    }
} 