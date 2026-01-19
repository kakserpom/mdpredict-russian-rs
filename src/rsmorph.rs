//! `RsMorphy`-based morphological analysis for Russian text
//! Uses the `OpenCorpora` dictionary for accurate POS tagging

use rsmorphy::prelude::*;

use crate::dictionaries::{
    EMOTION_WORDS, EXTERNAL_PREDICATES, FILLER_WORDS, FIRST_PERSON_PLURAL, FIRST_PERSON_SINGULAR,
    INTERNAL_PREDICATES, POSSESSIVE_FIRST_PERSON, SECOND_PERSON_PLURAL, SECOND_PERSON_SINGULAR,
    SOCIAL_FAMILY_WORDS, STOP_WORDS, THIRD_PERSON_PLURAL, THIRD_PERSON_SINGULAR,
};

/// Part of speech categories (matching our existing enum)
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
    Finite,
    Infinitive,
    Participle,
    Gerund,
    Unknown,
}

/// Predicate type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateType {
    External,
    Internal,
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
    pub lemma: Option<String>,
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
            lemma: None,
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

/// RsMorphy-based morphological analyzer
pub struct RsMorphAnalyzer {
    analyzer: MorphAnalyzer,
}

impl RsMorphAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        // Load dictionary from the rsmorphy-dict-ru crate
        let analyzer = MorphAnalyzer::from_file(rsmorphy_dict_ru::DICT_PATH);
        Self { analyzer }
    }

    /// Analyze a single word using rsmorphy
    #[must_use]
    pub fn analyze(&self, word: &str) -> WordAnalysis {
        let word_lower = word.to_lowercase();
        let mut analysis = WordAnalysis::new(&word_lower);

        // Check special categories first (using our dictionaries)
        analysis.is_filler = FILLER_WORDS.contains(word_lower.as_str());
        analysis.is_stop_word = STOP_WORDS.contains(word_lower.as_str());
        analysis.is_emotion_word = EMOTION_WORDS.contains(word_lower.as_str());
        analysis.is_egocentrism_marker = FIRST_PERSON_SINGULAR.contains(word_lower.as_str())
            || POSSESSIVE_FIRST_PERSON.contains(word_lower.as_str());
        // Check for social/family words
        if SOCIAL_FAMILY_WORDS.contains(word_lower.as_str()) {
            analysis.is_social_interaction = true;
        }

        // Check pronouns using our dictionaries (more reliable for this purpose)
        if let Some((person, number)) = Self::check_pronoun_dictionaries(&word_lower) {
            analysis.pos = PartOfSpeech::Pronoun;
            analysis.pronoun_person = Some(person);
            analysis.pronoun_number = Some(number);
            analysis.is_social_interaction = matches!(person, PronounPerson::First)
                && matches!(number, PronounNumber::Plural);
            return analysis;
        }

        // Parse with rsmorphy
        let parses = self.analyzer.parse(&word_lower);

        // Always check predicate type using our dictionaries first
        // (more reliable than rsmorphy for this specific use case)
        let predicate_type = Self::check_predicate_type(&word_lower, &word_lower);
        if predicate_type != PredicateType::Neither {
            analysis.predicate_type = Some(predicate_type);
            // If it's a predicate, it's effectively a verb for our purposes
            if analysis.pos == PartOfSpeech::Unknown || analysis.pos == PartOfSpeech::Conjunction {
                analysis.pos = PartOfSpeech::Verb;
            }
        }

        if let Some(parse) = parses.first() {
            // Get lemma (normal form)
            let normal_form = parse.lex.get_normal_form(&self.analyzer);
            analysis.lemma = Some(normal_form.to_string());

            // Extract POS and other info from grammemes
            let tag = parse.lex.get_tag(&self.analyzer);
            let grammemes = &tag.grammemes;

            // Only override POS if we didn't already set it from predicate check
            if analysis.predicate_type.is_none() {
                analysis.pos = Self::extract_pos(grammemes);
            }

            // If it's a verb, extract tense and form
            if analysis.pos == PartOfSpeech::Verb {
                analysis.verb_tense = Some(Self::extract_verb_tense(grammemes));
                analysis.verb_form = Some(Self::extract_verb_form(grammemes));

                // Check predicate type using lemma if not already set
                if analysis.predicate_type.is_none() {
                    let lemma = analysis.lemma.as_deref().unwrap_or(&word_lower);
                    let pred_type = Self::check_predicate_type(&word_lower, lemma);
                    if pred_type != PredicateType::Neither {
                        analysis.predicate_type = Some(pred_type);
                    }
                }
            }

            // Check for social interaction (1st person plural verbs)
            if analysis.pos == PartOfSpeech::Verb && Self::is_first_person_plural(grammemes) {
                analysis.is_social_interaction = true;
            }
        }

        analysis
    }

    /// Check if word is a pronoun using our dictionaries
    fn check_pronoun_dictionaries(word: &str) -> Option<(PronounPerson, PronounNumber)> {
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
            if word == "себя" || word == "себе" || word == "собой" || word == "собою" {
                return Some((PronounPerson::Reflexive, PronounNumber::Unknown));
            }
            return Some((PronounPerson::First, PronounNumber::Unknown));
        }
        None
    }

    /// Check if grammeme set contains a specific tag
    fn has_grammeme(grammemes: &GrammemeSet, tag: &str) -> bool {
        grammemes.set.contains(&Grammeme::new(tag))
    }

    /// Extract part of speech from grammemes
    fn extract_pos(grammemes: &GrammemeSet) -> PartOfSpeech {
        // Check for main POS tags in OpenCorpora format
        if Self::has_grammeme(grammemes, "NOUN") {
            PartOfSpeech::Noun
        } else if Self::has_grammeme(grammemes, "VERB")
            || Self::has_grammeme(grammemes, "INFN")
            || Self::has_grammeme(grammemes, "PRTF")
            || Self::has_grammeme(grammemes, "PRTS")
            || Self::has_grammeme(grammemes, "GRND")
        {
            PartOfSpeech::Verb
        } else if Self::has_grammeme(grammemes, "ADJF") || Self::has_grammeme(grammemes, "ADJS") {
            PartOfSpeech::Adjective
        } else if Self::has_grammeme(grammemes, "ADVB") {
            PartOfSpeech::Adverb
        } else if Self::has_grammeme(grammemes, "NPRO") {
            PartOfSpeech::Pronoun
        } else if Self::has_grammeme(grammemes, "PREP") {
            PartOfSpeech::Preposition
        } else if Self::has_grammeme(grammemes, "CONJ") {
            PartOfSpeech::Conjunction
        } else if Self::has_grammeme(grammemes, "NUMR") {
            PartOfSpeech::Numeral
        } else if Self::has_grammeme(grammemes, "PRCL") {
            PartOfSpeech::Particle
        } else if Self::has_grammeme(grammemes, "INTJ") {
            PartOfSpeech::Interjection
        } else {
            PartOfSpeech::Unknown
        }
    }

    /// Extract verb tense from grammemes
    fn extract_verb_tense(grammemes: &GrammemeSet) -> VerbTense {
        if Self::has_grammeme(grammemes, "INFN") {
            VerbTense::Infinitive
        } else if Self::has_grammeme(grammemes, "past") {
            VerbTense::Past
        } else if Self::has_grammeme(grammemes, "pres") {
            VerbTense::Present
        } else if Self::has_grammeme(grammemes, "futr") {
            VerbTense::Future
        } else {
            VerbTense::Unknown
        }
    }

    /// Extract verb form from grammemes
    fn extract_verb_form(grammemes: &GrammemeSet) -> VerbForm {
        if Self::has_grammeme(grammemes, "INFN") {
            VerbForm::Infinitive
        } else if Self::has_grammeme(grammemes, "PRTF") || Self::has_grammeme(grammemes, "PRTS") {
            VerbForm::Participle
        } else if Self::has_grammeme(grammemes, "GRND") {
            VerbForm::Gerund
        } else if Self::has_grammeme(grammemes, "VERB") {
            VerbForm::Finite
        } else {
            VerbForm::Unknown
        }
    }

    /// Check if verb is 1st person plural
    fn is_first_person_plural(grammemes: &GrammemeSet) -> bool {
        Self::has_grammeme(grammemes, "1per") && Self::has_grammeme(grammemes, "plur")
    }

    /// Check predicate type (external/internal) using lemma
    fn check_predicate_type(word: &str, lemma: &str) -> PredicateType {
        // Check both word and lemma in our predicate dictionaries
        if INTERNAL_PREDICATES.contains(word) || INTERNAL_PREDICATES.contains(lemma) {
            PredicateType::Internal
        } else if EXTERNAL_PREDICATES.contains(word) || EXTERNAL_PREDICATES.contains(lemma) {
            PredicateType::External
        } else {
            PredicateType::Neither
        }
    }

    /// Check if a word is a verb in active voice (approximation)
    #[must_use]
    pub fn is_active_voice(word: &str) -> bool {
        !word.ends_with("ся") && !word.ends_with("сь")
    }

    /// Check if a word is a coordinating conjunction
    #[must_use]
    pub fn is_coordinating_conjunction(word: &str) -> bool {
        crate::dictionaries::COORDINATING_CONJUNCTIONS.contains(word.to_lowercase().as_str())
    }

    /// Check if a word is a subordinating conjunction
    #[must_use]
    pub fn is_subordinating_conjunction(word: &str) -> bool {
        crate::dictionaries::SUBORDINATING_CONJUNCTIONS.contains(word.to_lowercase().as_str())
    }
}

impl Default for RsMorphAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verb_detection() {
        let analyzer = RsMorphAnalyzer::new();

        let analysis = analyzer.analyze("думаю");
        assert_eq!(analysis.pos, PartOfSpeech::Verb);
        assert_eq!(analysis.verb_tense, Some(VerbTense::Present));
        assert_eq!(analysis.predicate_type, Some(PredicateType::Internal));

        let analysis = analyzer.analyze("иду");
        assert_eq!(analysis.pos, PartOfSpeech::Verb);
        assert_eq!(analysis.predicate_type, Some(PredicateType::External));
    }

    #[test]
    fn test_past_tense() {
        let analyzer = RsMorphAnalyzer::new();

        let analysis = analyzer.analyze("катался");
        assert_eq!(analysis.pos, PartOfSpeech::Verb);
        assert_eq!(analysis.verb_tense, Some(VerbTense::Past));
    }

    #[test]
    fn test_pronoun_detection() {
        let analyzer = RsMorphAnalyzer::new();

        let analysis = analyzer.analyze("я");
        assert_eq!(analysis.pos, PartOfSpeech::Pronoun);
        assert_eq!(analysis.pronoun_person, Some(PronounPerson::First));
        assert_eq!(analysis.pronoun_number, Some(PronounNumber::Singular));

        let analysis = analyzer.analyze("мы");
        assert_eq!(analysis.pos, PartOfSpeech::Pronoun);
        assert_eq!(analysis.pronoun_person, Some(PronounPerson::First));
        assert_eq!(analysis.pronoun_number, Some(PronounNumber::Plural));
        assert!(analysis.is_social_interaction);
    }

    #[test]
    fn test_noun_detection() {
        let analyzer = RsMorphAnalyzer::new();

        let analysis = analyzer.analyze("велосипед");
        assert_eq!(analysis.pos, PartOfSpeech::Noun);
    }

    #[test]
    fn test_adjective_detection() {
        let analyzer = RsMorphAnalyzer::new();

        let analysis = analyzer.analyze("красивый");
        assert_eq!(analysis.pos, PartOfSpeech::Adjective);
    }

    #[test]
    fn test_lemmatization() {
        let analyzer = RsMorphAnalyzer::new();

        let analysis = analyzer.analyze("катался");
        assert!(analysis.lemma.is_some());
        // Lemma should be "кататься"
        let lemma = analysis.lemma.unwrap();
        assert!(lemma.starts_with("катат"), "Lemma was: {}", lemma);
    }

    #[test]
    fn test_predicate_detection() {
        let analyzer = RsMorphAnalyzer::new();

        // Test internal predicates
        let internal_words = ["казалось", "думаю", "помню", "чувствую"];
        println!("\nInternal predicate detection:");
        for word in &internal_words {
            let analysis = analyzer.analyze(word);
            println!("  {}: pos={:?}, predicate={:?}, lemma={:?}",
                word, analysis.pos, analysis.predicate_type, analysis.lemma);
        }

        // Test external predicates
        let external_words = ["шла", "упала", "плакала", "смеялись"];
        println!("\nExternal predicate detection:");
        for word in &external_words {
            let analysis = analyzer.analyze(word);
            println!("  {}: pos={:?}, predicate={:?}, lemma={:?}",
                word, analysis.pos, analysis.predicate_type, analysis.lemma);
        }
    }
}
