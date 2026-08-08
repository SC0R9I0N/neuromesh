//! Frequency-based keyword extraction with a stopword filter.

use std::collections::HashMap;

const STOPWORDS: &[&str] = &[
    "about", "above", "after", "again", "against", "along", "also", "always", "among",
    "another", "anything", "around", "because", "been", "before", "being", "below",
    "between", "both", "cannot", "could", "does", "doing", "down", "during", "each",
    "either", "else", "even", "ever", "every", "everything", "from", "further", "gets",
    "goes", "going", "have", "having", "here", "however", "into", "itself", "just",
    "like", "likely", "made", "make", "makes", "many", "maybe", "might", "more", "most",
    "much", "must", "myself", "need", "needs", "never", "nothing", "once", "only",
    "onto", "other", "others", "over", "perhaps", "quite", "rather", "really", "same",
    "several", "shall", "should", "since", "some", "something", "still", "such", "sure",
    "take", "than", "that", "their", "theirs", "them", "then", "there", "these", "they",
    "things", "this", "those", "though", "through", "thus", "under", "until", "upon",
    "used", "using", "very", "want", "well", "were", "what", "when", "where", "whether",
    "which", "while", "whose", "will", "with", "within", "without", "would", "your",
    "yours",
];

pub fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(&word)
}

/// Returns up to `max` keywords ordered by frequency (ties broken alphabetically).
/// Lowercases, splits on non-alphanumerics, drops short words, digits, and stopwords.
pub fn extract_keywords(text: &str, max: usize) -> Vec<String> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for word in text.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        if word.len() < 4 || word.chars().all(|c| c.is_ascii_digit()) || is_stopword(word) {
            continue;
        }
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }
    let mut ranked: Vec<(String, u32)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(max);
    ranked.into_iter().map(|(word, _)| word).collect()
}
