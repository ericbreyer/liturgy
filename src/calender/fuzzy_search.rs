//! Fuzzy search implementation using Levenshtein distance and n-gram analysis
//!
//! This module provides custom fuzzy matching algorithms that combine multiple
//! string similarity techniques for better search results.

use std::collections::HashSet;

/// Custom fuzzy search implementation using Levenshtein distance and n-gram analysis
pub fn fuzzy_search_best_n<'a>(query: &str, candidates: &[&'a str], n: usize) -> Vec<(&'a str, f32)> {
    let query_lower = query.to_lowercase();
    let mut scores: Vec<(&'a str, f32)> = Vec::new();
    
    for candidate in candidates {
        let candidate_lower = candidate.to_lowercase();
        let score = calculate_fuzzy_score(&query_lower, &candidate_lower);
        if score > 0.0 {
            scores.push((candidate, score));
        }
    }
    
    // Sort by score descending and take top n
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores.into_iter().take(n).collect()
}

/// Calculate fuzzy match score combining multiple algorithms
fn calculate_fuzzy_score(query: &str, candidate: &str) -> f32 {
    // Exact match gets highest score
    if query == candidate {
        return 1.0;
    }
    
    // Substring matches get high scores
    if candidate.contains(query) {
        return 0.9 - (candidate.len() - query.len()) as f32 / candidate.len() as f32 * 0.1;
    }
    
    if query.contains(candidate) {
        return 0.85;
    }
    
    // Prefix matching
    if candidate.starts_with(query) {
        return 0.8;
    }
    
    // Word-based matching
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let candidate_words: Vec<&str> = candidate.split_whitespace().collect();
    
    let word_score = calculate_word_overlap_score(&query_words, &candidate_words);
    if word_score > 0.5 {
        return word_score * 0.7;
    }
    
    // N-gram similarity
    let ngram_score = calculate_ngram_similarity(query, candidate, 2);
    let trigram_score = calculate_ngram_similarity(query, candidate, 3);
    let combined_ngram = (ngram_score * 0.6 + trigram_score * 0.4).max(0.0);
    
    // Levenshtein distance based score
    let lev_score = calculate_levenshtein_score(query, candidate);
    
    // Combine scores with weights
    let final_score = (combined_ngram * 0.4 + lev_score * 0.6).max(0.0);
    
    // Filter out very low scores
    if final_score < 0.2 {
        0.0
    } else {
        final_score
    }
}

/// Calculate word overlap score
fn calculate_word_overlap_score(query_words: &[&str], candidate_words: &[&str]) -> f32 {
    if query_words.is_empty() || candidate_words.is_empty() {
        return 0.0;
    }
    
    let mut matches = 0;
    for query_word in query_words {
        for candidate_word in candidate_words {
            if candidate_word.contains(query_word) || query_word.contains(candidate_word) {
                matches += 1;
                break;
            }
        }
    }
    
    matches as f32 / query_words.len() as f32
}

/// Calculate n-gram similarity using Jaccard index
fn calculate_ngram_similarity(s1: &str, s2: &str, n: usize) -> f32 {
    let ngrams1 = get_ngrams(s1, n);
    let ngrams2 = get_ngrams(s2, n);
    
    if ngrams1.is_empty() && ngrams2.is_empty() {
        return 1.0;
    }
    
    if ngrams1.is_empty() || ngrams2.is_empty() {
        return 0.0;
    }
    
    let set1: HashSet<&str> = ngrams1.iter().map(|s| s.as_str()).collect();
    let set2: HashSet<&str> = ngrams2.iter().map(|s| s.as_str()).collect();
    
    let intersection = set1.intersection(&set2).count();
    let union = set1.union(&set2).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Generate n-grams from a string
fn get_ngrams(s: &str, n: usize) -> Vec<String> {
    if s.len() < n {
        return vec![s.to_string()];
    }
    
    let chars: Vec<char> = s.chars().collect();
    let mut ngrams = Vec::new();
    
    for i in 0..=chars.len().saturating_sub(n) {
        let ngram: String = chars[i..i+n].iter().collect();
        ngrams.push(ngram);
    }
    
    ngrams
}

/// Calculate Levenshtein-based score
fn calculate_levenshtein_score(s1: &str, s2: &str) -> f32 {
    let distance = levenshtein_distance(s1, s2);
    let max_len = s1.len().max(s2.len());
    
    if max_len == 0 {
        return 1.0;
    }
    
    let normalized_distance = distance as f32 / max_len as f32;
    (1.0 - normalized_distance).max(0.0)
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let s1_len = s1_chars.len();
    let s2_len = s2_chars.len();
    
    if s1_len == 0 {
        return s2_len;
    }
    if s2_len == 0 {
        return s1_len;
    }
    
    let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];
    
    // Initialize first row and column
    for i in 0..=s1_len {
        matrix[i][0] = i;
    }
    for j in 0..=s2_len {
        matrix[0][j] = j;
    }
    
    // Fill the matrix
    for i in 1..=s1_len {
        for j in 1..=s2_len {
            let cost = if s1_chars[i-1] == s2_chars[j-1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i-1][j] + 1)           // deletion
                .min(matrix[i][j-1] + 1)                  // insertion
                .min(matrix[i-1][j-1] + cost);            // substitution
        }
    }
    
    matrix[s1_len][s2_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let candidates = vec!["hello", "world"];
        let results = fuzzy_search_best_n("hello", &candidates, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "hello");
        assert_eq!(results[0].1, 1.0);
    }

    #[test]
    fn test_substring_match() {
        let candidates = vec!["hello world", "goodbye"];
        let results = fuzzy_search_best_n("hello", &candidates, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "hello world");
        assert!(results[0].1 > 0.8);
    }

    #[test]
    fn test_fuzzy_match() {
        let candidates = vec!["hello", "helo", "help"];
        let results = fuzzy_search_best_n("hello", &candidates, 5);
        assert!(results.len() >= 2);
        // Should find exact match first
        assert_eq!(results[0].0, "hello");
        assert_eq!(results[0].1, 1.0);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("hello", "help"), 2);
    }

    #[test]
    fn test_ngram_similarity() {
        let score = calculate_ngram_similarity("hello", "hello", 2);
        assert_eq!(score, 1.0);
        
        let score = calculate_ngram_similarity("hello", "helo", 2);
        assert!(score > 0.5);
    }
}
