#![warn(clippy::pedantic)]
//! # mdpredict-russian
//!
//! Mental Disorder Prediction based on structural characteristics of Russian written speech.
//!
//! A Rust library for analyzing structural characteristics of written speech
//! to assist in mental health research. Based on the research paper:
//!
//! "Diagnostic value of the structural characteristics of written speech
//! in patients with schizophrenia" (Smerchinskaya, Tregubenko, Isaeva, 2026)
//!
//! # Features
//!
//! - Morphological analysis of Russian text
//! - Sentence structure classification
//! - Computation of 19+ structural speech characteristics
//! - Classification into diagnostic groups (Schizophrenia, Bipolar, Personality Disorder, Healthy)
//!
//! # Example
//!
//! ```
//! use mdpredict_russian::{TextAnalyzer, Classifier};
//!
//! let analyzer = TextAnalyzer::new();
//! let classifier = Classifier::new();
//!
//! let text = "Я помню как катался на велосипеде и упал.";
//! let metrics = analyzer.analyze(text);
//! let result = classifier.classify(&metrics);
//!
//! println!("Classification: {}", result.primary_diagnosis);
//! println!("Confidence: {:.1}%", result.confidence * 100.0);
//! ```
//!
//! # Disclaimer
//!
//! This tool is for research purposes only and should NOT be used as a
//! substitute for professional medical diagnosis. Always consult a
//! qualified healthcare professional for mental health assessments.

pub mod analyzer;
pub mod classifier;
pub mod dictionaries;
pub mod metrics;
pub mod morphology;
pub mod rsmorph;
pub mod sentence;

// Re-export main types
pub use analyzer::TextAnalyzer;
pub use classifier::Classifier;
pub use metrics::{ClassificationResult, DiagnosticGroup, GroupScores, TextMetrics};
pub use rsmorph::{RsMorphAnalyzer, PartOfSpeech, PredicateType, VerbForm, VerbTense};
pub use sentence::{SentenceAnalyzer, SentenceType};

/// Convenience function to analyze text and get classification
#[must_use] 
pub fn analyze_and_classify(text: &str) -> (TextMetrics, ClassificationResult) {
    let analyzer = TextAnalyzer::new();
    let classifier = Classifier::new();

    let metrics = analyzer.analyze(text);
    let result = classifier.classify(&metrics);

    (metrics, result)
}

/// Get a full analysis report for text
#[must_use] 
pub fn get_full_report(text: &str) -> String {
    let analyzer = TextAnalyzer::new();
    let classifier = Classifier::new();

    let metrics = analyzer.analyze(text);
    let result = classifier.classify(&metrics);

    classifier.get_detailed_report(&metrics, &result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_and_classify() {
        let text = "Я помню как мы гуляли в парке с мамой.";
        let (metrics, result) = analyze_and_classify(text);

        assert!(metrics.total_words > 0);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_full_report() {
        let text = "Я помню как катался на велосипеде и упал.";
        let report = get_full_report(text);

        assert!(report.contains("АНАЛИЗ"));
        assert!(report.contains("РЕЗУЛЬТАТ"));
    }

    #[test]
    fn test_schizophrenia_pattern() {
        // Example from paper: short text, past tense, external predicates
        let text = "Как я катался на 3-колёсном велосипеде и упал. 3–4 года";
        let (metrics, _result) = analyze_and_classify(text);

        // Schizophrenia texts tend to be shorter
        assert!(metrics.total_words < 30);
        // Should have past tense verbs
        assert!(metrics.past_tense_verbs > 0.0 || metrics.external_predicates > 0.0);
    }

    #[test]
    fn test_healthy_pattern() {
        // Example from paper: longer text, present tense, emotional
        let text = "Ко мне подходит мама, склоняется надо мной и просит поднять руки. \
                    Я поднимаю руки и на меня натягивается блузка с короткими рукавами-фонариками. \
                    Кофточка из прозрачного плохо тянущегося материала отделана блестящим люрексом \
                    сильно колется. Но ничего, я потерплю. Это часть костюма и без нее никак не обойтись.";
        let (metrics, _result) = analyze_and_classify(text);

        // Healthy texts tend to be longer
        assert!(metrics.total_words > 30);
        // Should have present tense verbs
        assert!(metrics.present_tense_verbs > 0.0);
    }
}
