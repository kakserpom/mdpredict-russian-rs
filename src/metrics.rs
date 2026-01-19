//! Text metrics structure based on the research paper
//! "Diagnostic value of the structural characteristics of written speech
//! in patients with schizophrenia" (Smerchinskaya et al., 2026)

use serde::{Deserialize, Serialize};

/// All structural characteristics of written speech analyzed in the study
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextMetrics {
    // Basic metrics
    /// Total word count (absolute value)
    pub total_words: usize,
    /// Total sentence count
    pub total_sentences: usize,

    // Sentence structure (absolute counts)
    /// Run-on sentences (бессоюзные предложения)
    pub run_on_sentences: usize,
    /// Compound sentences (сложносочиненные предложения)
    pub compound_sentences: usize,
    /// Complex sentences (сложноподчиненные предложения)
    pub complex_sentences: usize,
    /// Simple sentences (простые предложения)
    pub simple_sentences: usize,

    // Lexical metrics (as percentage of total words)
    /// Lexical diversity index (unique words / total words * 100)
    pub lexical_diversity_index: f64,

    // Predicates (as percentage)
    /// External predicates - verbs related to external/visible actions ("иду", "говорят")
    pub external_predicates: f64,
    /// Internal predicates - verbs denoting mental/emotional states ("думаю", "чувствую")
    pub internal_predicates: f64,

    // Verb metrics (as percentage)
    /// Verbs in active voice
    pub active_voice_verbs: f64,
    /// Past tense verbs
    pub past_tense_verbs: f64,
    /// Present tense verbs
    pub present_tense_verbs: f64,
    /// Future tense verbs
    pub future_tense_verbs: f64,
    /// Infinitives
    pub infinitives: f64,
    /// Non-finite verb forms (participles, gerunds - причастия, деепричастия)
    pub non_finite_verb_forms: f64,

    // Parts of speech (as percentage)
    /// Adjectives
    pub adjectives: f64,
    /// Nouns
    pub nouns: f64,
    /// Adverbs
    pub adverbs: f64,

    // Pronouns (as percentage)
    /// 1st person singular pronouns (я, меня, мне, мной, мною)
    pub first_person_singular_pronouns: f64,
    /// 1st person plural pronouns (мы, нас, нам, нами)
    pub first_person_plural_pronouns: f64,
    /// 2nd person singular pronouns (ты, тебя, тебе, тобой, тобою)
    pub second_person_singular_pronouns: f64,
    /// 2nd person plural pronouns (вы, вас, вам, вами)
    pub second_person_plural_pronouns: f64,
    /// 3rd person singular pronouns (он, она, оно, его, её, ему, ей, им, ею)
    pub third_person_singular_pronouns: f64,
    /// 3rd person plural pronouns (они, их, им, ими)
    pub third_person_plural_pronouns: f64,

    // Special indices (as percentage)
    /// Filler words index (слова-паразиты: "же", "ведь", "ну", "вот", "короче")
    pub filler_words_index: f64,
    /// Stop words index / водность (general phrases, terms / total words)
    pub stop_words_index: f64,

    // Function words (as percentage)
    /// Prepositions
    pub prepositions: f64,
    /// Conjunctions
    pub conjunctions: f64,

    // Semantic categories (as percentage)
    /// Words meaning "social interaction" (constructions with "мы" and 1st person plural verbs)
    pub social_interaction_words: f64,
    /// Words meaning "emotions" ("весело", "грустно", "печальный", etc.)
    pub emotion_words: f64,

    // Egocentrism index
    /// Egocentrism index - pronouns "Я" and derivatives ("меня", "мой"), including reflexive ("себя")
    pub egocentrism_index: f64,
}

impl TextMetrics {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate percentage from count and total words
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn percentage(count: usize, total: usize) -> f64 {
        if total == 0 {
            0.0
        } else {
            (count as f64 / total as f64) * 100.0
        }
    }
}

/// Diagnostic group classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticGroup {
    /// Healthy participants
    Healthy,
    /// Schizophrenia patients
    Schizophrenia,
    /// Personality disorder patients
    PersonalityDisorder,
    /// Bipolar affective disorder patients
    BipolarDisorder,
}

impl std::fmt::Display for DiagnosticGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticGroup::Healthy => write!(f, "Психически здоровые лица"),
            DiagnosticGroup::Schizophrenia => write!(f, "Шизофрения"),
            DiagnosticGroup::PersonalityDisorder => write!(f, "Расстройство личности"),
            DiagnosticGroup::BipolarDisorder => write!(f, "Биполярное аффективное расстройство"),
        }
    }
}

/// Classification result with confidence scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// Primary diagnosis
    pub primary_diagnosis: DiagnosticGroup,
    /// Confidence score for the primary diagnosis (0.0 - 1.0)
    pub confidence: f64,
    /// Scores for each diagnostic group
    pub group_scores: GroupScores,
}

/// Scores for each diagnostic group
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupScores {
    pub healthy: f64,
    pub schizophrenia: f64,
    pub personality_disorder: f64,
    pub bipolar_disorder: f64,
}

/// Reference values from the research paper (Table 2)
#[derive(Debug, Clone)]
pub struct ReferenceValues {
    pub group: DiagnosticGroup,
    pub metrics: TextMetrics,
    pub std_dev: TextMetrics,
}

impl ReferenceValues {
    /// Create reference values for schizophrenia group based on the paper (Table 2)
    #[must_use]
    pub fn schizophrenia() -> Self {
        let mut metrics = TextMetrics::new();
        metrics.total_words = 19; // average: 19.39
        metrics.run_on_sentences = 0;
        metrics.compound_sentences = 0;
        metrics.complex_sentences = 0;
        metrics.simple_sentences = 2;
        metrics.lexical_diversity_index = 73.61;
        metrics.external_predicates = 13.33;
        metrics.internal_predicates = 2.87;
        metrics.active_voice_verbs = 15.02;
        metrics.past_tense_verbs = 10.69;
        metrics.present_tense_verbs = 3.08;
        metrics.future_tense_verbs = 0.26;
        metrics.infinitives = 1.56;
        metrics.non_finite_verb_forms = 0.41;
        metrics.adjectives = 5.60;
        metrics.nouns = 35.25;
        metrics.adverbs = 2.85;
        metrics.first_person_singular_pronouns = 4.63;
        metrics.first_person_plural_pronouns = 0.52;
        metrics.second_person_singular_pronouns = 0.30;
        metrics.second_person_plural_pronouns = 0.00;
        metrics.third_person_singular_pronouns = 0.62;
        metrics.third_person_plural_pronouns = 0.05;
        metrics.filler_words_index = 0.52;
        metrics.stop_words_index = 0.22;
        metrics.prepositions = 14.58;
        metrics.conjunctions = 4.71;
        metrics.social_interaction_words = 1.02;
        metrics.emotion_words = 0.77;
        metrics.egocentrism_index = 9.99;

        // Standard deviations from Table 2
        let mut std_dev = TextMetrics::new();
        std_dev.total_words = 15; // SD: 15.21
        std_dev.lexical_diversity_index = 22.30;
        std_dev.external_predicates = 9.69;
        std_dev.internal_predicates = 4.54;
        std_dev.past_tense_verbs = 9.08;
        std_dev.present_tense_verbs = 5.63;
        std_dev.first_person_singular_pronouns = 5.07;
        std_dev.emotion_words = 2.20;
        std_dev.social_interaction_words = 2.92;
        std_dev.non_finite_verb_forms = 1.75;

        Self {
            group: DiagnosticGroup::Schizophrenia,
            metrics,
            std_dev,
        }
    }

    /// Create reference values for healthy participants based on the paper (Table 2)
    #[must_use]
    pub fn healthy() -> Self {
        let mut metrics = TextMetrics::new();
        metrics.total_words = 85; // average: 84.76
        metrics.run_on_sentences = 1;
        metrics.compound_sentences = 1;
        metrics.complex_sentences = 1;
        metrics.simple_sentences = 5;
        metrics.lexical_diversity_index = 64.60;
        metrics.external_predicates = 10.26;
        metrics.internal_predicates = 6.61;
        metrics.active_voice_verbs = 14.59;
        metrics.past_tense_verbs = 9.55;
        metrics.present_tense_verbs = 4.80;
        metrics.future_tense_verbs = 0.24;
        metrics.infinitives = 1.32;
        metrics.non_finite_verb_forms = 0.80;
        metrics.adjectives = 7.57;
        metrics.nouns = 26.51;
        metrics.adverbs = 5.41;
        metrics.first_person_singular_pronouns = 6.74;
        metrics.first_person_plural_pronouns = 0.65;
        metrics.second_person_singular_pronouns = 0.00;
        metrics.second_person_plural_pronouns = 0.00;
        metrics.third_person_singular_pronouns = 1.01;
        metrics.third_person_plural_pronouns = 0.52;
        metrics.filler_words_index = 0.47;
        metrics.stop_words_index = 0.19;
        metrics.prepositions = 10.90;
        metrics.conjunctions = 5.89;
        metrics.social_interaction_words = 0.86;
        metrics.emotion_words = 0.99;
        metrics.egocentrism_index = 12.09;

        // Standard deviations from Table 2
        let mut std_dev = TextMetrics::new();
        std_dev.total_words = 67; // SD: 67.38
        std_dev.lexical_diversity_index = 14.52;
        std_dev.external_predicates = 5.54;
        std_dev.internal_predicates = 6.86;
        std_dev.past_tense_verbs = 6.05;
        std_dev.present_tense_verbs = 4.14;
        std_dev.first_person_singular_pronouns = 4.43;
        std_dev.emotion_words = 1.55;
        std_dev.social_interaction_words = 1.52;
        std_dev.non_finite_verb_forms = 1.40;

        Self {
            group: DiagnosticGroup::Healthy,
            metrics,
            std_dev,
        }
    }

    /// Create reference values for personality disorder based on the paper (Table 2)
    #[must_use]
    pub fn personality_disorder() -> Self {
        let mut metrics = TextMetrics::new();
        metrics.total_words = 22; // average: 21.56
        metrics.run_on_sentences = 1;
        metrics.compound_sentences = 0;
        metrics.complex_sentences = 0;
        metrics.simple_sentences = 1;
        metrics.lexical_diversity_index = 71.95;
        metrics.external_predicates = 13.59;
        metrics.internal_predicates = 5.48;
        metrics.active_voice_verbs = 14.47;
        metrics.past_tense_verbs = 7.60;
        metrics.present_tense_verbs = 6.77;
        metrics.future_tense_verbs = 0.23;
        metrics.infinitives = 0.74;
        metrics.non_finite_verb_forms = 0.63;
        metrics.adjectives = 5.24;
        metrics.nouns = 31.36;
        metrics.adverbs = 4.16;
        metrics.first_person_singular_pronouns = 7.06;
        metrics.first_person_plural_pronouns = 0.98;
        metrics.second_person_singular_pronouns = 0.42;
        metrics.second_person_plural_pronouns = 0.20;
        metrics.third_person_singular_pronouns = 1.86;
        metrics.third_person_plural_pronouns = 0.06;
        metrics.filler_words_index = 0.27;
        metrics.stop_words_index = 0.17;
        metrics.prepositions = 13.40;
        metrics.conjunctions = 5.02;
        metrics.social_interaction_words = 2.15;
        metrics.emotion_words = 1.43;
        metrics.egocentrism_index = 10.36;

        // Standard deviations from Table 2
        let mut std_dev = TextMetrics::new();
        std_dev.total_words = 17; // SD: 16.81
        std_dev.lexical_diversity_index = 18.62;
        std_dev.external_predicates = 8.75;
        std_dev.internal_predicates = 5.65;
        std_dev.past_tense_verbs = 7.41;
        std_dev.present_tense_verbs = 6.75;
        std_dev.first_person_singular_pronouns = 5.35;
        std_dev.emotion_words = 2.77;
        std_dev.social_interaction_words = 2.85;
        std_dev.non_finite_verb_forms = 1.83;

        Self {
            group: DiagnosticGroup::PersonalityDisorder,
            metrics,
            std_dev,
        }
    }

    /// Create reference values for bipolar disorder based on the paper (Table 2)
    #[must_use]
    pub fn bipolar_disorder() -> Self {
        let mut metrics = TextMetrics::new();
        metrics.total_words = 25; // average: 24.94
        metrics.run_on_sentences = 0;
        metrics.compound_sentences = 0;
        metrics.complex_sentences = 0;
        metrics.simple_sentences = 2;
        metrics.lexical_diversity_index = 69.82;
        metrics.external_predicates = 14.28;
        metrics.internal_predicates = 3.88;
        metrics.active_voice_verbs = 14.33;
        metrics.past_tense_verbs = 8.62;
        metrics.present_tense_verbs = 6.10;
        metrics.future_tense_verbs = 0.38;
        metrics.infinitives = 0.88;
        metrics.non_finite_verb_forms = 1.84;
        metrics.adjectives = 5.23;
        metrics.nouns = 33.54;
        metrics.adverbs = 3.72;
        metrics.first_person_singular_pronouns = 8.65;
        metrics.first_person_plural_pronouns = 0.92;
        metrics.second_person_singular_pronouns = 0.10;
        metrics.second_person_plural_pronouns = 0.00;
        metrics.third_person_singular_pronouns = 0.44;
        metrics.third_person_plural_pronouns = 0.38;
        metrics.filler_words_index = 0.89;
        metrics.stop_words_index = 0.21;
        metrics.prepositions = 17.65;
        metrics.conjunctions = 4.49;
        metrics.social_interaction_words = 1.43;
        metrics.emotion_words = 1.72;
        metrics.egocentrism_index = 10.83;

        // Standard deviations from Table 2
        let mut std_dev = TextMetrics::new();
        std_dev.total_words = 23; // SD: 22.82
        std_dev.lexical_diversity_index = 20.68;
        std_dev.external_predicates = 8.27;
        std_dev.internal_predicates = 5.44;
        std_dev.past_tense_verbs = 8.80;
        std_dev.present_tense_verbs = 6.78;
        std_dev.first_person_singular_pronouns = 5.83;
        std_dev.emotion_words = 3.02;
        std_dev.social_interaction_words = 2.87;
        std_dev.non_finite_verb_forms = 2.77;

        Self {
            group: DiagnosticGroup::BipolarDisorder,
            metrics,
            std_dev,
        }
    }
}
