//! Morphological analysis for Russian text
//! Rule-based approach using dictionaries and suffix patterns

use crate::dictionaries::{PREPOSITIONS, ALL_CONJUNCTIONS, COORDINATING_CONJUNCTIONS, SUBORDINATING_CONJUNCTIONS, FIRST_PERSON_SINGULAR, FIRST_PERSON_PLURAL, SECOND_PERSON_SINGULAR, SECOND_PERSON_PLURAL, THIRD_PERSON_SINGULAR, THIRD_PERSON_PLURAL, POSSESSIVE_FIRST_PERSON, INTERNAL_PREDICATES, EXTERNAL_PREDICATES, ends_with_any, INFINITIVE_ENDINGS, PARTICIPLE_ENDINGS, PAST_TENSE_ENDINGS, KNOWN_ADVERBS, ADJECTIVE_ENDINGS, FILLER_WORDS, STOP_WORDS, EMOTION_WORDS};

/// Part of speech categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Preposition,
    Conjunction,
    Numeral,
    Particle,
    Interjection,
    Unknown,
}

/// Verb tense
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbTense {
    Past,
    Present,
    Future,
    Infinitive,
    Unknown,
}

/// Verb form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbForm {
    Finite,       // личная форма
    Infinitive,   // инфинитив
    Participle,   // причастие
    Gerund,       // деепричастие
    Unknown,
}

/// Predicate type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateType {
    External,  // внешние действия
    Internal,  // внутренние переживания
    Neither,
}

/// Pronoun person
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PronounPerson {
    First,
    Second,
    Third,
    Reflexive,
    Unknown,
}

/// Pronoun number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PronounNumber {
    Singular,
    Plural,
    Unknown,
}

/// Word analysis result
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct WordAnalysis {
    pub word: String,
    pub pos: PartOfSpeech,
    pub verb_tense: Option<VerbTense>,
    pub verb_form: Option<VerbForm>,
    pub predicate_type: Option<PredicateType>,
    pub pronoun_person: Option<PronounPerson>,
    pub pronoun_number: Option<PronounNumber>,
    pub is_filler: bool,
    pub is_stop_word: bool,
    pub is_emotion_word: bool,
    pub is_social_interaction: bool,
    pub is_egocentrism_marker: bool,
}

impl WordAnalysis {
    #[must_use] 
    pub fn new(word: &str) -> Self {
        Self {
            word: word.to_string(),
            pos: PartOfSpeech::Unknown,
            verb_tense: None,
            verb_form: None,
            predicate_type: None,
            pronoun_person: None,
            pronoun_number: None,
            is_filler: false,
            is_stop_word: false,
            is_emotion_word: false,
            is_social_interaction: false,
            is_egocentrism_marker: false,
        }
    }
}

/// Morphological analyzer
pub struct MorphAnalyzer;

impl MorphAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Analyze a single word
    #[must_use]
    pub fn analyze(&self, word: &str) -> WordAnalysis {
        let word_lower = word.to_lowercase();
        let mut analysis = WordAnalysis::new(&word_lower);

        // Check special categories first
        analysis.is_filler = Self::is_filler_word(&word_lower);
        analysis.is_stop_word = Self::is_stop_word(&word_lower);
        analysis.is_emotion_word = Self::is_emotion_word(&word_lower);
        analysis.is_egocentrism_marker = Self::is_egocentrism_marker(&word_lower);

        // Determine part of speech
        if Self::is_preposition(&word_lower) {
            analysis.pos = PartOfSpeech::Preposition;
        } else if Self::is_conjunction(&word_lower) {
            analysis.pos = PartOfSpeech::Conjunction;
        } else if let Some((person, number)) = Self::get_pronoun_info(&word_lower) {
            analysis.pos = PartOfSpeech::Pronoun;
            analysis.pronoun_person = Some(person);
            analysis.pronoun_number = Some(number);
        } else if let Some((tense, form, pred_type)) = Self::analyze_verb(&word_lower) {
            analysis.pos = PartOfSpeech::Verb;
            analysis.verb_tense = Some(tense);
            analysis.verb_form = Some(form);
            analysis.predicate_type = Some(pred_type);
        } else if Self::is_adverb(&word_lower) {
            analysis.pos = PartOfSpeech::Adverb;
        } else if Self::is_adjective(&word_lower) {
            analysis.pos = PartOfSpeech::Adjective;
        } else if Self::is_noun(&word_lower) {
            analysis.pos = PartOfSpeech::Noun;
        }

        // Check for social interaction
        analysis.is_social_interaction = Self::is_social_interaction_word(&word_lower);

        analysis
    }

    fn is_preposition(word: &str) -> bool {
        PREPOSITIONS.contains(word)
    }

    fn is_conjunction(word: &str) -> bool {
        ALL_CONJUNCTIONS.contains(word)
    }

    #[must_use]
    pub fn is_coordinating_conjunction(word: &str) -> bool {
        COORDINATING_CONJUNCTIONS.contains(word.to_lowercase().as_str())
    }

    #[must_use]
    pub fn is_subordinating_conjunction(word: &str) -> bool {
        SUBORDINATING_CONJUNCTIONS.contains(word.to_lowercase().as_str())
    }

    fn get_pronoun_info(word: &str) -> Option<(PronounPerson, PronounNumber)> {
        if FIRST_PERSON_SINGULAR.contains(word) {
            return Some((PronounPerson::First, PronounNumber::Singular));
        }
        if FIRST_PERSON_PLURAL.contains(word) {
            return Some((PronounPerson::First, PronounNumber::Plural));
        }
        if SECOND_PERSON_SINGULAR.contains(word) {
            return Some((PronounPerson::Second, PronounNumber::Singular));
        }
        if SECOND_PERSON_PLURAL.contains(word) {
            return Some((PronounPerson::Second, PronounNumber::Plural));
        }
        if THIRD_PERSON_SINGULAR.contains(word) {
            return Some((PronounPerson::Third, PronounNumber::Singular));
        }
        if THIRD_PERSON_PLURAL.contains(word) {
            return Some((PronounPerson::Third, PronounNumber::Plural));
        }
        if POSSESSIVE_FIRST_PERSON.contains(word) {
            // Check for reflexive pronouns
            if word == "себя" || word == "себе" || word == "собой" || word == "собою" {
                return Some((PronounPerson::Reflexive, PronounNumber::Unknown));
            }
            return Some((PronounPerson::First, PronounNumber::Unknown));
        }
        None
    }

    fn analyze_verb(word: &str) -> Option<(VerbTense, VerbForm, PredicateType)> {
        // Check if it's in our predicate dictionaries first
        let is_internal = INTERNAL_PREDICATES.contains(word);
        let is_external = EXTERNAL_PREDICATES.contains(word);

        let pred_type = if is_internal {
            PredicateType::Internal
        } else if is_external {
            PredicateType::External
        } else {
            PredicateType::Neither
        };

        // If it's in our verb dictionaries, it's definitely a verb
        if is_internal || is_external {
            let (tense, form) = Self::determine_verb_tense_form(word);
            return Some((tense, form, pred_type));
        }

        // Check by endings
        // Check for infinitive first
        if ends_with_any(word, INFINITIVE_ENDINGS) {
            return Some((VerbTense::Infinitive, VerbForm::Infinitive, pred_type));
        }

        // Check for participles
        if ends_with_any(word, PARTICIPLE_ENDINGS) {
            return Some((VerbTense::Unknown, VerbForm::Participle, pred_type));
        }

        // Check for gerunds (деепричастия) - more specific check
        if Self::is_gerund(word) {
            return Some((VerbTense::Unknown, VerbForm::Gerund, pred_type));
        }

        // Check for past tense
        if ends_with_any(word, PAST_TENSE_ENDINGS) && word.len() > 3 {
            return Some((VerbTense::Past, VerbForm::Finite, pred_type));
        }

        // Check for present tense
        if Self::looks_like_present_tense(word) {
            return Some((VerbTense::Present, VerbForm::Finite, pred_type));
        }

        None
    }

    fn determine_verb_tense_form(word: &str) -> (VerbTense, VerbForm) {
        if ends_with_any(word, INFINITIVE_ENDINGS) {
            return (VerbTense::Infinitive, VerbForm::Infinitive);
        }
        if ends_with_any(word, PARTICIPLE_ENDINGS) {
            return (VerbTense::Unknown, VerbForm::Participle);
        }
        if Self::is_gerund(word) {
            return (VerbTense::Unknown, VerbForm::Gerund);
        }
        if ends_with_any(word, PAST_TENSE_ENDINGS) {
            return (VerbTense::Past, VerbForm::Finite);
        }
        // Default to present for known verbs not matching other patterns
        (VerbTense::Present, VerbForm::Finite)
    }

    fn is_gerund(word: &str) -> bool {
        // Gerunds typically end in -я, -а, -в, -вши, -вшись
        // But we need to be careful not to confuse with other words
        let gerund_patterns = ["ая", "яя", "ив", "ав", "ев", "ув", "ывая", "ивая", "вшись", "вши"];
        for pattern in &gerund_patterns {
            if word.ends_with(pattern) && word.len() > pattern.len() + 2 {
                return true;
            }
        }
        false
    }

    fn looks_like_present_tense(word: &str) -> bool {
        // Check common present tense patterns
        let present_endings = ["ю", "ешь", "ёшь", "ет", "ём", "ем", "ете", "ют",
                               "у", "ишь", "ит", "им", "ите", "ят", "ат"];

        for ending in &present_endings {
            if word.ends_with(ending) && word.len() > ending.len() + 2 {
                // Additional heuristic: check if the stem looks verbal
                let stem = &word[..word.len() - ending.len()];
                if !stem.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    fn is_adverb(word: &str) -> bool {
        if KNOWN_ADVERBS.contains(word) {
            return true;
        }
        // Most Russian adverbs end in -о or -е (derived from adjectives)
        // But we need to be careful as many other words end this way too
        if word.len() > 4 && (word.ends_with("о") || word.ends_with("е")) {
            // Check if it could be an adverb derived from adjective
            // Usually these are longer words
            let without_suffix = &word[..word.len() - 2]; // remove -о/-е (2 bytes for Cyrillic)
            if without_suffix.len() > 3 {
                return true;
            }
        }
        false
    }

    fn is_adjective(word: &str) -> bool {
        // Check by common adjective endings
        ends_with_any(word, ADJECTIVE_ENDINGS) && word.len() > 4
    }

    fn is_noun(word: &str) -> bool {
        // This is a fallback - if nothing else matches and word is long enough
        // Default to noun as Russian has many nouns
        word.len() > 2
    }

    fn is_filler_word(word: &str) -> bool {
        FILLER_WORDS.contains(word)
    }

    fn is_stop_word(word: &str) -> bool {
        STOP_WORDS.contains(word)
    }

    fn is_emotion_word(word: &str) -> bool {
        EMOTION_WORDS.contains(word)
    }

    fn is_egocentrism_marker(word: &str) -> bool {
        FIRST_PERSON_SINGULAR.contains(word) || POSSESSIVE_FIRST_PERSON.contains(word)
    }

    fn is_social_interaction_word(word: &str) -> bool {
        // Social interaction involves "мы" or 1st person plural verb forms
        if FIRST_PERSON_PLURAL.contains(word) {
            return true;
        }
        // Check for 1st person plural verb endings (-ем, -им, -ём)
        if word.ends_with("ем") || word.ends_with("им") || word.ends_with("ём") {
            return true;
        }
        false
    }

    /// Check if a word is a verb in active voice (approximation)
    #[must_use]
    pub fn is_active_voice(word: &str) -> bool {
        // In Russian, passive voice is typically formed with:
        // 1. Short passive participles (-н, -т endings)
        // 2. Reflexive verbs with -ся/-сь (sometimes)
        // 3. Analytical constructions with быть + participle

        // For simplicity, we consider non-reflexive verbs as active voice
        // and verbs without passive participle endings
        !word.ends_with("ся") && !word.ends_with("сь")
    }
}

impl Default for MorphAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pronoun_detection() {
        let analyzer = MorphAnalyzer::new();

        let analysis = analyzer.analyze("я");
        assert_eq!(analysis.pos, PartOfSpeech::Pronoun);
        assert_eq!(analysis.pronoun_person, Some(PronounPerson::First));
        assert_eq!(analysis.pronoun_number, Some(PronounNumber::Singular));

        let analysis = analyzer.analyze("мы");
        assert_eq!(analysis.pos, PartOfSpeech::Pronoun);
        assert_eq!(analysis.pronoun_person, Some(PronounPerson::First));
        assert_eq!(analysis.pronoun_number, Some(PronounNumber::Plural));
    }

    #[test]
    fn test_verb_detection() {
        let analyzer = MorphAnalyzer::new();

        let analysis = analyzer.analyze("думаю");
        assert_eq!(analysis.pos, PartOfSpeech::Verb);
        assert_eq!(analysis.predicate_type, Some(PredicateType::Internal));

        let analysis = analyzer.analyze("иду");
        assert_eq!(analysis.pos, PartOfSpeech::Verb);
        assert_eq!(analysis.predicate_type, Some(PredicateType::External));
    }

    #[test]
    fn test_preposition_detection() {
        let analyzer = MorphAnalyzer::new();

        let analysis = analyzer.analyze("в");
        assert_eq!(analysis.pos, PartOfSpeech::Preposition);

        let analysis = analyzer.analyze("на");
        assert_eq!(analysis.pos, PartOfSpeech::Preposition);
    }
}
