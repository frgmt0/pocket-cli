use anyhow::{Result, anyhow};
use std::collections::HashMap;

#[cfg(feature = "ml-summarization")]
use std::sync::Arc;
#[cfg(feature = "ml-summarization")]
use rust_bert::bart::{BartForConditionalGeneration, BartConfig, BartModelResources};
#[cfg(feature = "ml-summarization")]
use rust_bert::resources::{RemoteResource, Resource, LocalResource};
#[cfg(feature = "ml-summarization")]
use rust_bert::pipelines::summarization::{SummarizationConfig, SummarizationModel};
#[cfg(feature = "ml-summarization")]
use rust_tokenizers::tokenizer::{BartTokenizer, Tokenizer, TruncationStrategy};
#[cfg(feature = "ml-summarization")]
use tch::{nn, Device, Tensor};
#[cfg(feature = "ml-summarization")]
use rust_bert::RustBertError;

// Singleton pattern for the summarization model to avoid reloading
#[cfg(feature = "ml-summarization")]
lazy_static::lazy_static! {
    static ref SUMMARIZATION_MODEL: std::sync::Mutex<Option<Box<SummarizationModel>>> = std::sync::Mutex::new(None);
}

/// Initialize the summarization model (if not already initialized)
#[cfg(feature = "ml-summarization")]
pub fn initialize_summarization_model() -> Result<()> {
    let mut model_guard = SUMMARIZATION_MODEL.lock().unwrap();
    
    if model_guard.is_some() {
        return Ok(());
    }
    
    let config_resource = Resource::Remote(RemoteResource::from_pretrained(
        "distilbart-cnn-6-6-config.json",
    ));
    let vocab_resource = Resource::Remote(RemoteResource::from_pretrained(
        "distilbart-cnn-6-6-vocab.json",
    ));
    let merges_resource = Resource::Remote(RemoteResource::from_pretrained(
        "distilbart-cnn-6-6-merges.txt",
    ));
    let model_resource = Resource::Remote(RemoteResource::from_pretrained(
        "distilbart-cnn-6-6-model.safetensors",
    ));
    
    let generate_config = rust_bert::pipelines::generation::GenerateConfig {
        max_length: 100,  // Short summary
        min_length: 10,   // Reasonable minimum
        do_sample: false, // deterministic generation
        early_stopping: true,
        no_repeat_ngram_size: 3,
        num_beams: 3,     // Low beam count to conserve memory
        temperature: 1.0,
        top_k: 50,
        top_p: 0.95,
        repetition_penalty: 1.2,
        length_penalty: 1.0,
        ..Default::default()
    };
    
    let model_resources = BartModelResources {
        config_resource,
        vocab_resource,
        merges_resource,
        model_resource,
    };
    
    let summarization_config = SummarizationConfig {
        model_resource: model_resources,
        min_length: 10,
        max_length: 100,
        early_stopping: true,
        num_beams: 3,
        device: Device::cuda_if_available(), // Use CUDA if available, otherwise CPU
        ..Default::default()
    };
    
    let model = SummarizationModel::new(summarization_config)?;
    *model_guard = Some(Box::new(model));
    
    Ok(())
}

/// Summarize text content - returns short summary
/// 
/// Optimized for minimal memory usage
pub fn summarize_text(text: &str) -> Result<String> {
    // If text is very short, don't summarize
    if text.split_whitespace().count() < 20 {
        return Ok(text.to_string());
    }
    
    #[cfg(feature = "ml-summarization")]
    {
        // Initialize model if needed
        initialize_summarization_model()?;
        
        let guard = SUMMARIZATION_MODEL.lock().unwrap();
        
        if let Some(model) = &*guard {
            // Truncate text if it's very long to conserve memory
            let truncated_text = if text.len() > 4000 {
                text.chars().take(4000).collect::<String>()
            } else {
                text.to_string()
            };
            
            let input_texts = vec![&truncated_text];
            let output = model.summarize(&input_texts)?;
            
            // If summarization failed or returned nothing, use a fallback
            if output.is_empty() {
                return fallback_summarize_text(text);
            }
            
            return Ok(output[0].clone());
        }
    }
    
    // If ML summarization is not available or failed, use fallback
    fallback_summarize_text(text)
}

/// Alternative lightweight summarization method that uses a rule-based approach
/// when the ML model is unavailable or fails
fn fallback_summarize_text(text: &str) -> Result<String> {
    // Simple extractive summarization based on sentence importance
    
    // Split into sentences
    let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .collect();
    
    if sentences.len() <= 2 {
        return Ok(text.to_string());
    }
    
    // Count word frequencies to calculate sentence importance
    let mut word_freqs = HashMap::new();
    
    for sentence in &sentences {
        for word in sentence.split_whitespace() {
            let word = word.trim().to_lowercase();
            if word.len() > 2 { // Skip very short words
                *word_freqs.entry(word).or_insert(0) += 1;
            }
        }
    }
    
    // Calculate sentence scores based on word frequencies
    let mut sentence_scores: Vec<(usize, f64)> = sentences.iter().enumerate()
        .map(|(i, &sentence)| {
            let words = sentence.split_whitespace()
                .map(|w| w.trim().to_lowercase())
                .filter(|w| w.len() > 2)
                .collect::<Vec<_>>();
            
            let score = words.iter()
                .map(|word| word_freqs.get(word).unwrap_or(&0))
                .sum::<u32>() as f64 / words.len().max(1) as f64;
            
            (i, score)
        })
        .collect();
    
    // Sort by score (highest first)
    sentence_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Take top 3 sentences or 20% of total sentences, whichever is greater
    let num_sentences = sentences.len().max(3).min((sentences.len() as f64 * 0.2).ceil() as usize);
    
    // Get indices of top sentences and sort them by original position
    let mut top_indices: Vec<usize> = sentence_scores.iter()
        .take(num_sentences)
        .map(|(idx, _)| *idx)
        .collect();
    
    top_indices.sort();
    
    // Reconstruct summary from selected sentences
    let summary = top_indices.iter()
        .map(|&idx| sentences[idx])
        .collect::<Vec<_>>()
        .join(". ");
    
    Ok(format!("{}.", summary))
}

pub struct SummaryMetadata {
    pub summary: String,
    pub is_auto_generated: bool,
}

impl SummaryMetadata {
    pub fn new(summary: String, is_auto_generated: bool) -> Self {
        Self {
            summary,
            is_auto_generated,
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "summary": self.summary,
            "auto_generated": self.is_auto_generated,
        }).to_string()
    }
    
    pub fn from_json(json: &str) -> Result<Self> {
        let parsed: serde_json::Value = serde_json::from_str(json)?;
        
        let summary = parsed["summary"].as_str()
            .ok_or_else(|| anyhow!("Missing 'summary' field in summary metadata"))?
            .to_string();
            
        let is_auto_generated = parsed["auto_generated"].as_bool()
            .unwrap_or(true);
            
        Ok(Self {
            summary,
            is_auto_generated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fallback_summarization() {
        let long_text = "This is the first sentence about programming. This is a second sentence about Rust language features. 
        This third sentence covers memory safety in systems programming. The fourth sentence discusses performance implications.
        This is the fifth sentence on concurrency. The sixth sentence is about pattern matching features.
        Seventh sentence looks at the module system. Eighth sentence considers error handling approaches.";
        
        let result = fallback_summarize_text(long_text).unwrap();
        assert!(!result.is_empty());
        assert!(result.len() < long_text.len());
    }
    
    #[test]
    fn test_summary_metadata_serialization() {
        let metadata = SummaryMetadata::new("Test summary".to_string(), true);
        let json = metadata.to_json();
        let deserialized = SummaryMetadata::from_json(&json).unwrap();
        
        assert_eq!(metadata.summary, deserialized.summary);
        assert_eq!(metadata.is_auto_generated, deserialized.is_auto_generated);
    }
} 