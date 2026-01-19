//! Sentence structure analysis
//! Determines sentence types: simple, compound, complex, run-on

use crate::morphology::MorphAnalyzer;
use regex::Regex;
use std::sync::LazyLock;

static SENTENCE_SPLITTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[.!?]+\s*").unwrap());

static CLAUSE_BOUNDARY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[,;:\-–—]").unwrap());

/// Sentence type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentenceType {
    /// Simple sentence (простое предложение) - one independent clause
    Simple,
    /// Compound sentence (сложносочиненное) - independent clauses joined by coordinating conjunctions
    Compound,
    /// Complex sentence (сложноподчиненное) - independent + dependent clause with subordinating conjunction
    Complex,
    /// Run-on sentence (бессоюзное) - clauses without conjunctions
    RunOn,
}

/// Sentence analysis result
#[derive(Debug, Clone)]
pub struct SentenceAnalysis {
    pub text: String,
    pub sentence_type: SentenceType,
    pub clause_count: usize,
    pub word_count: usize,
    pub has_coordinating_conjunction: bool,
    pub has_subordinating_conjunction: bool,
}

/// Sentence analyzer
pub struct SentenceAnalyzer {
    morph: MorphAnalyzer,
}

impl SentenceAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            morph: MorphAnalyzer::new(),
        }
    }

    /// Split text into sentences
    #[must_use]
    pub fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let cleaned = text.trim();
        if cleaned.is_empty() {
            return Vec::new();
        }

        SENTENCE_SPLITTER
            .split(cleaned)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Analyze a single sentence
    #[must_use]
    pub fn analyze_sentence(&self, sentence: &str) -> SentenceAnalysis {
        let words = Self::extract_words(sentence);
        let word_count = words.len();

        let has_coordinating = Self::has_coordinating_conjunction(&words);
        let has_subordinating = Self::has_subordinating_conjunction(&words);

        // Count potential clause boundaries
        let clause_boundaries = CLAUSE_BOUNDARY.find_iter(sentence).count();

        // Estimate clause count based on commas, conjunctions, and sentence length
        let clause_count = self.estimate_clause_count(&words, clause_boundaries);

        // Determine sentence type
        let sentence_type =
            Self::determine_sentence_type(clause_count, has_coordinating, has_subordinating);

        SentenceAnalysis {
            text: sentence.to_string(),
            sentence_type,
            clause_count,
            word_count,
            has_coordinating_conjunction: has_coordinating,
            has_subordinating_conjunction: has_subordinating,
        }
    }

    /// Extract words from sentence
    fn extract_words(sentence: &str) -> Vec<String> {
        sentence
            .split(|c: char| !c.is_alphanumeric())
            .map(str::to_lowercase)
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Check for coordinating conjunctions
    fn has_coordinating_conjunction(words: &[String]) -> bool {
        words
            .iter()
            .any(|w| MorphAnalyzer::is_coordinating_conjunction(w))
    }

    /// Check for subordinating conjunctions
    fn has_subordinating_conjunction(words: &[String]) -> bool {
        words
            .iter()
            .any(|w| MorphAnalyzer::is_subordinating_conjunction(w))
    }

    /// Estimate the number of clauses in a sentence
    fn estimate_clause_count(&self, words: &[String], clause_boundaries: usize) -> usize {
        // A clause typically needs at least a subject and predicate
        // We use several heuristics:

        // 1. Count verbs (potential predicates)
        let verb_count = self.count_potential_verbs(words);

        // 2. Consider punctuation boundaries
        let punct_estimate = if clause_boundaries > 0 {
            clause_boundaries + 1
        } else {
            1
        };

        // 3. Consider conjunctions
        let conj_count = words
            .iter()
            .filter(|w| {
                MorphAnalyzer::is_coordinating_conjunction(w)
                    || MorphAnalyzer::is_subordinating_conjunction(w)
            })
            .count();

        // Take the minimum of verb count and punctuation estimate
        // but at least 1, and consider conjunctions
        let base_estimate = verb_count.min(punct_estimate).max(1);

        // If we have conjunctions, we likely have multiple clauses
        if conj_count > 0 && base_estimate == 1 {
            conj_count + 1
        } else {
            base_estimate
        }
    }

    /// Count potential verbs in the word list
    fn count_potential_verbs(&self, words: &[String]) -> usize {
        let mut count = 0;
        for word in words {
            let analysis = self.morph.analyze(word);
            if analysis.pos == crate::morphology::PartOfSpeech::Verb {
                count += 1;
            }
        }
        // Each clause should have at least one verb
        count.max(1)
    }

    /// Determine sentence type based on clause count and conjunctions
    fn determine_sentence_type(
        clause_count: usize,
        has_coordinating: bool,
        has_subordinating: bool,
    ) -> SentenceType {
        if clause_count <= 1 {
            // Single clause = simple sentence
            SentenceType::Simple
        } else if has_subordinating {
            // Has subordinating conjunction = complex sentence
            SentenceType::Complex
        } else if has_coordinating {
            // Has coordinating conjunction = compound sentence
            SentenceType::Compound
        } else {
            // Multiple clauses without conjunctions = run-on sentence
            SentenceType::RunOn
        }
    }

    /// Analyze all sentences in a text
    #[must_use]
    pub fn analyze_text(&self, text: &str) -> Vec<SentenceAnalysis> {
        self.split_into_sentences(text)
            .iter()
            .map(|s| self.analyze_sentence(s))
            .collect()
    }
}

impl Default for SentenceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_splitting() {
        let analyzer = SentenceAnalyzer::new();

        let text = "Это первое предложение. А это второе! И третье?";
        let sentences = analyzer.split_into_sentences(text);

        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn test_simple_sentence() {
        let analyzer = SentenceAnalyzer::new();

        let analysis = analyzer.analyze_sentence("Я иду домой");
        assert_eq!(analysis.sentence_type, SentenceType::Simple);
    }

    #[test]
    fn test_compound_sentence() {
        let analyzer = SentenceAnalyzer::new();

        let analysis = analyzer.analyze_sentence("Я иду домой, а он идёт в школу");
        // Should detect coordinating conjunction "а"
        assert!(analysis.has_coordinating_conjunction);
    }

    #[test]
    fn test_complex_sentence() {
        let analyzer = SentenceAnalyzer::new();

        let analysis = analyzer.analyze_sentence("Я знаю, что он придёт");
        // Should detect subordinating conjunction "что"
        assert!(analysis.has_subordinating_conjunction);
    }
}
