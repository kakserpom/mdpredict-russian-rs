//! Main text analyzer that computes all metrics
//! Based on the methodology from the research paper

use crate::metrics::TextMetrics;
use crate::rsmorph::{
    PartOfSpeech, PredicateType, PronounNumber, PronounPerson, RsMorphAnalyzer, VerbForm, VerbTense,
};
use crate::sentence::{SentenceAnalyzer, SentenceType};
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

/// Main text analyzer
pub struct TextAnalyzer {
    morph: RsMorphAnalyzer,
    sentence_analyzer: SentenceAnalyzer,
}

impl TextAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            morph: RsMorphAnalyzer::new(),
            sentence_analyzer: SentenceAnalyzer::new(),
        }
    }

    /// Analyze text and compute all metrics
    #[must_use]
    pub fn analyze(&self, text: &str) -> TextMetrics {
        let mut metrics = TextMetrics::new();

        // Extract words
        let words = Self::extract_words(text);
        let total_words = words.len();
        metrics.total_words = total_words;

        if total_words == 0 {
            return metrics;
        }

        // Analyze sentences
        let sentence_analyses = self.sentence_analyzer.analyze_text(text);
        metrics.total_sentences = sentence_analyses.len();

        // Count sentence types
        for analysis in &sentence_analyses {
            match analysis.sentence_type {
                SentenceType::Simple => metrics.simple_sentences += 1,
                SentenceType::Compound => metrics.compound_sentences += 1,
                SentenceType::Complex => metrics.complex_sentences += 1,
                SentenceType::RunOn => metrics.run_on_sentences += 1,
            }
        }

        // Calculate lexical diversity
        let unique_words: HashSet<_> = words.iter().map(|w| w.to_lowercase()).collect();
        metrics.lexical_diversity_index = TextMetrics::percentage(unique_words.len(), total_words);

        // Analyze each word
        let mut counters = WordCounters::default();

        for word in &words {
            let analysis = self.morph.analyze(word);

            // Count parts of speech
            match analysis.pos {
                PartOfSpeech::Noun => counters.nouns += 1,
                PartOfSpeech::Adjective => counters.adjectives += 1,
                PartOfSpeech::Adverb => counters.adverbs += 1,
                PartOfSpeech::Preposition => counters.prepositions += 1,
                PartOfSpeech::Conjunction => counters.conjunctions += 1,
                PartOfSpeech::Pronoun => {
                    Self::count_pronoun(&analysis, &mut counters);
                }
                PartOfSpeech::Verb => {
                    Self::count_verb(&analysis, word, &mut counters);
                }
                _ => {}
            }

            // Count special categories
            if analysis.is_filler {
                counters.filler_words += 1;
            }
            if analysis.is_stop_word {
                counters.stop_words += 1;
            }
            if analysis.is_emotion_word {
                counters.emotion_words += 1;
            }
            if analysis.is_social_interaction {
                counters.social_interaction_words += 1;
            }
            if analysis.is_egocentrism_marker {
                counters.egocentrism_markers += 1;
            }
        }

        // Convert counts to percentages
        Self::counters_to_metrics(&counters, total_words, &mut metrics);

        metrics
    }

    /// Extract words from text using Unicode word segmentation
    fn extract_words(text: &str) -> Vec<String> {
        text.unicode_words()
            .filter(|s| s.chars().any(char::is_alphabetic))
            .map(ToString::to_string)
            .collect()
    }

    /// Count pronoun types
    fn count_pronoun(analysis: &crate::rsmorph::WordAnalysis, counters: &mut WordCounters) {
        match (analysis.pronoun_person, analysis.pronoun_number) {
            (Some(PronounPerson::First), Some(PronounNumber::Singular)) => {
                counters.first_person_singular += 1;
            }
            (Some(PronounPerson::First), Some(PronounNumber::Plural)) => {
                counters.first_person_plural += 1;
            }
            (Some(PronounPerson::Second), Some(PronounNumber::Singular)) => {
                counters.second_person_singular += 1;
            }
            (Some(PronounPerson::Second), Some(PronounNumber::Plural)) => {
                counters.second_person_plural += 1;
            }
            (Some(PronounPerson::Third), Some(PronounNumber::Singular)) => {
                counters.third_person_singular += 1;
            }
            (Some(PronounPerson::Third), Some(PronounNumber::Plural)) => {
                counters.third_person_plural += 1;
            }
            (Some(PronounPerson::First), Some(PronounNumber::Unknown)) => {
                // Possessive pronouns - count as egocentrism
                counters.first_person_singular += 1;
            }
            (Some(PronounPerson::Reflexive), _) => {
                // Reflexive pronouns себя, etc.
                counters.first_person_singular += 1;
            }
            _ => {}
        }
    }

    /// Count verb types
    fn count_verb(
        analysis: &crate::rsmorph::WordAnalysis,
        word: &str,
        counters: &mut WordCounters,
    ) {
        // Count by tense
        match analysis.verb_tense {
            Some(VerbTense::Past) => counters.past_tense += 1,
            Some(VerbTense::Present) => counters.present_tense += 1,
            Some(VerbTense::Future) => counters.future_tense += 1,
            Some(VerbTense::Infinitive) => counters.infinitives += 1,
            _ => {}
        }

        // Count by form
        if let Some(VerbForm::Participle | VerbForm::Gerund) = analysis.verb_form {
            counters.non_finite_forms += 1;
        }

        // Count by predicate type
        match analysis.predicate_type {
            Some(PredicateType::External) => counters.external_predicates += 1,
            Some(PredicateType::Internal) => counters.internal_predicates += 1,
            _ => {}
        }

        // Count active voice
        if RsMorphAnalyzer::is_active_voice(word) {
            counters.active_voice += 1;
        }
    }

    /// Convert word counters to metric percentages
    fn counters_to_metrics(counters: &WordCounters, total: usize, metrics: &mut TextMetrics) {
        metrics.nouns = TextMetrics::percentage(counters.nouns, total);
        metrics.adjectives = TextMetrics::percentage(counters.adjectives, total);
        metrics.adverbs = TextMetrics::percentage(counters.adverbs, total);
        metrics.prepositions = TextMetrics::percentage(counters.prepositions, total);
        metrics.conjunctions = TextMetrics::percentage(counters.conjunctions, total);

        // Pronouns
        metrics.first_person_singular_pronouns =
            TextMetrics::percentage(counters.first_person_singular, total);
        metrics.first_person_plural_pronouns =
            TextMetrics::percentage(counters.first_person_plural, total);
        metrics.second_person_singular_pronouns =
            TextMetrics::percentage(counters.second_person_singular, total);
        metrics.second_person_plural_pronouns =
            TextMetrics::percentage(counters.second_person_plural, total);
        metrics.third_person_singular_pronouns =
            TextMetrics::percentage(counters.third_person_singular, total);
        metrics.third_person_plural_pronouns =
            TextMetrics::percentage(counters.third_person_plural, total);

        // Verbs
        metrics.past_tense_verbs = TextMetrics::percentage(counters.past_tense, total);
        metrics.present_tense_verbs = TextMetrics::percentage(counters.present_tense, total);
        metrics.future_tense_verbs = TextMetrics::percentage(counters.future_tense, total);
        metrics.infinitives = TextMetrics::percentage(counters.infinitives, total);
        metrics.non_finite_verb_forms = TextMetrics::percentage(counters.non_finite_forms, total);
        metrics.active_voice_verbs = TextMetrics::percentage(counters.active_voice, total);

        // Predicates
        metrics.external_predicates = TextMetrics::percentage(counters.external_predicates, total);
        metrics.internal_predicates = TextMetrics::percentage(counters.internal_predicates, total);

        // Special categories
        metrics.filler_words_index = TextMetrics::percentage(counters.filler_words, total);
        metrics.stop_words_index = TextMetrics::percentage(counters.stop_words, total);
        metrics.emotion_words = TextMetrics::percentage(counters.emotion_words, total);
        metrics.social_interaction_words =
            TextMetrics::percentage(counters.social_interaction_words, total);
        metrics.egocentrism_index = TextMetrics::percentage(counters.egocentrism_markers, total);
    }
}

impl Default for TextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal counter structure
#[derive(Default)]
struct WordCounters {
    // Parts of speech
    nouns: usize,
    adjectives: usize,
    adverbs: usize,
    prepositions: usize,
    conjunctions: usize,

    // Pronouns
    first_person_singular: usize,
    first_person_plural: usize,
    second_person_singular: usize,
    second_person_plural: usize,
    third_person_singular: usize,
    third_person_plural: usize,

    // Verb tenses
    past_tense: usize,
    present_tense: usize,
    future_tense: usize,
    infinitives: usize,
    non_finite_forms: usize,
    active_voice: usize,

    // Predicates
    external_predicates: usize,
    internal_predicates: usize,

    // Special categories
    filler_words: usize,
    stop_words: usize,
    emotion_words: usize,
    social_interaction_words: usize,
    egocentrism_markers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_analysis() {
        let analyzer = TextAnalyzer::new();

        let text = "Я помню как катался на велосипеде и упал.";
        let metrics = analyzer.analyze(text);

        assert!(metrics.total_words > 0);
        assert!(metrics.total_sentences > 0);
    }

    #[test]
    fn test_pronoun_counting() {
        let analyzer = TextAnalyzer::new();

        let text = "Я иду домой. Мы идём вместе. Они тоже идут.";
        let metrics = analyzer.analyze(text);

        assert!(metrics.first_person_singular_pronouns > 0.0);
        assert!(metrics.first_person_plural_pronouns > 0.0);
        assert!(metrics.third_person_plural_pronouns > 0.0);
    }

    #[test]
    fn test_schizophrenia_example() {
        let analyzer = TextAnalyzer::new();

        // Example from the paper (schizophrenia patient)
        let text = "Как я катался на 3-колёсном велосипеде и упал. 3–4 года";
        let metrics = analyzer.analyze(text);

        // Should have past tense verbs
        assert!(metrics.past_tense_verbs > 0.0);
        // Should have first person singular pronoun
        assert!(metrics.first_person_singular_pronouns > 0.0);
    }

    #[test]
    fn test_healthy_example() {
        let analyzer = TextAnalyzer::new();

        // Example from the paper (healthy participant)
        let text = "Ко мне подходит мама, склоняется надо мной и просит поднять руки. \
                    Я поднимаю руки и на меня натягивается блузка с короткими рукавами-фонариками.";
        let metrics = analyzer.analyze(text);

        // Should have present tense verbs
        assert!(metrics.present_tense_verbs > 0.0);
        // Should have higher word count
        assert!(metrics.total_words > 15);
    }
}
