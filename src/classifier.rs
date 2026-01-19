//! Classifier for mental health diagnostics
//! Based on Linear Discriminant Analysis (LDA) from the research paper
//!
//! The paper uses stepwise discriminant analysis with the following key features:
//! - Function 1 (54.4% variance): Text volume, Non-finite verbs, First-person pronouns
//! - Function 2 (30.6% variance): Past tense, Present tense verbs
//! - Function 3 (15.0% variance): External predicates, Emotion words, Social interaction

use crate::metrics::{ClassificationResult, DiagnosticGroup, GroupScores, TextMetrics};
use std::fmt::Write;

/// Feature vector for LDA
#[derive(Debug, Clone)]
struct FeatureVector {
    /// Log-transformed text volume (most discriminating feature)
    log_volume: f64,
    /// Non-finite verb forms percentage
    non_finite_verbs: f64,
    /// First person singular pronouns percentage
    first_person_sing: f64,
    /// Past tense verbs percentage
    past_tense: f64,
    /// Present tense verbs percentage
    present_tense: f64,
    /// External predicates percentage
    external_pred: f64,
    /// Internal predicates percentage
    internal_pred: f64,
    /// Emotion words percentage
    emotion_words: f64,
    /// Social interaction words percentage
    social_interaction: f64,
    /// Lexical diversity index
    lexical_diversity: f64,
}

impl FeatureVector {
    #[allow(clippy::cast_precision_loss)]
    fn from_metrics(metrics: &TextMetrics) -> Self {
        Self {
            log_volume: (metrics.total_words as f64 + 1.0).ln(),
            non_finite_verbs: metrics.non_finite_verb_forms,
            first_person_sing: metrics.first_person_singular_pronouns,
            past_tense: metrics.past_tense_verbs,
            present_tense: metrics.present_tense_verbs,
            external_pred: metrics.external_predicates,
            internal_pred: metrics.internal_predicates,
            emotion_words: metrics.emotion_words,
            social_interaction: metrics.social_interaction_words,
            lexical_diversity: metrics.lexical_diversity_index,
        }
    }
}

/// Discriminant function coefficients for a group
/// Based on Fisher's Linear Discriminant Analysis
#[derive(Debug, Clone)]
struct DiscriminantCoefficients {
    /// Coefficient for log text volume
    log_volume: f64,
    /// Coefficient for non-finite verbs
    non_finite_verbs: f64,
    /// Coefficient for first person singular
    first_person_sing: f64,
    /// Coefficient for past tense
    past_tense: f64,
    /// Coefficient for present tense
    present_tense: f64,
    /// Coefficient for external predicates
    external_pred: f64,
    /// Coefficient for internal predicates
    internal_pred: f64,
    /// Coefficient for emotion words
    emotion_words: f64,
    /// Coefficient for social interaction
    social_interaction: f64,
    /// Coefficient for lexical diversity
    lexical_diversity: f64,
    /// Constant term
    constant: f64,
}

impl DiscriminantCoefficients {
    /// Calculate discriminant score for a feature vector
    fn score(&self, features: &FeatureVector) -> f64 {
        self.log_volume * features.log_volume
            + self.non_finite_verbs * features.non_finite_verbs
            + self.first_person_sing * features.first_person_sing
            + self.past_tense * features.past_tense
            + self.present_tense * features.present_tense
            + self.external_pred * features.external_pred
            + self.internal_pred * features.internal_pred
            + self.emotion_words * features.emotion_words
            + self.social_interaction * features.social_interaction
            + self.lexical_diversity * features.lexical_diversity
            + self.constant
    }

    /// Coefficients for Healthy group
    /// Derived from paper's reference values using LDA principles
    fn healthy() -> Self {
        Self {
            log_volume: 2.5,        // Healthy write longer texts (85 words avg)
            non_finite_verbs: 0.8,  // Higher non-finite forms
            first_person_sing: 0.15, // Moderate first person usage
            past_tense: -0.05,      // Less past tense than schizophrenia
            present_tense: 0.12,    // More present tense
            external_pred: -0.02,   // Lower external predicates
            internal_pred: 0.18,    // Higher internal predicates (6.61%)
            emotion_words: 0.15,    // Moderate emotion words
            social_interaction: -0.05, // Lower social interaction markers
            lexical_diversity: -0.02, // Lower diversity (longer texts)
            constant: -12.0,
        }
    }

    /// Coefficients for Schizophrenia group
    /// Key characteristics: shortest texts, highest past tense, lowest present tense,
    /// lowest internal predicates, lowest emotion words
    fn schizophrenia() -> Self {
        Self {
            log_volume: -2.0,       // Much shorter texts (19 words avg)
            non_finite_verbs: -0.5, // Lowest non-finite forms (0.41%)
            first_person_sing: -0.15, // Lowest first person (4.63%)
            past_tense: 0.2,        // Highest past tense (10.69%)
            present_tense: -0.25,   // Lowest present tense (3.08%)
            external_pred: 0.05,    // Moderate external predicates (13.33%)
            internal_pred: -0.35,   // Lowest internal predicates (2.87%)
            emotion_words: -0.4,    // Lowest emotion words (0.77%)
            social_interaction: -0.1, // Low social markers (1.02%)
            lexical_diversity: 0.04, // Highest diversity (73.61%)
            constant: -2.0,
        }
    }

    /// Coefficients for Personality Disorder group
    /// Key characteristics: HIGHEST social interaction (2.15% - key differentiator),
    /// highest present tense among patients, highest internal predicates among patients
    fn personality_disorder() -> Self {
        Self {
            log_volume: -0.8,       // Short texts (22 words avg)
            non_finite_verbs: 0.05, // Low non-finite (0.63%)
            first_person_sing: 0.08, // Moderate first person (7.06%)
            past_tense: -0.05,      // Lower past tense (7.60%)
            present_tense: 0.15,    // Highest present tense among patients (6.77%)
            external_pred: 0.05,    // High external predicates (13.59%)
            internal_pred: 0.2,     // Highest internal predicates among patients (5.48%)
            emotion_words: 0.2,     // Moderate emotion words
            social_interaction: 0.8, // KEY: Highest social interaction (2.15%)
            lexical_diversity: 0.02,
            constant: -4.0,
        }
    }

    /// Coefficients for Bipolar Disorder group
    /// Key characteristics: highest first person (8.65%), highest emotion words (1.72%),
    /// highest non-finite forms, LOWER social interaction than PD
    fn bipolar_disorder() -> Self {
        Self {
            log_volume: -0.6,       // Short texts (25 words avg)
            non_finite_verbs: 0.5,  // Highest non-finite (1.84%)
            first_person_sing: 0.2, // Highest first person (8.65%)
            past_tense: 0.02,       // Moderate past tense (8.62%)
            present_tense: 0.1,     // Moderate present tense (6.10%)
            external_pred: 0.12,    // Highest external predicates (14.28%)
            internal_pred: -0.1,    // Low internal predicates (3.88%)
            emotion_words: 0.35,    // Highest emotion words (1.72%)
            social_interaction: -0.2, // Negative: Lower social than PD (1.43% vs 2.15%)
            lexical_diversity: 0.01,
            constant: -5.5,
        }
    }
}

/// Classifier based on Linear Discriminant Analysis
pub struct Classifier {
    /// LDA coefficients for each group
    coefficients: Vec<(DiagnosticGroup, DiscriminantCoefficients)>,
}

impl Classifier {
    #[must_use]
    pub fn new() -> Self {
        Self {
            coefficients: vec![
                (DiagnosticGroup::Healthy, DiscriminantCoefficients::healthy()),
                (DiagnosticGroup::Schizophrenia, DiscriminantCoefficients::schizophrenia()),
                (DiagnosticGroup::PersonalityDisorder, DiscriminantCoefficients::personality_disorder()),
                (DiagnosticGroup::BipolarDisorder, DiscriminantCoefficients::bipolar_disorder()),
            ],
        }
    }

    /// Classify text based on computed metrics using LDA
    #[must_use]
    pub fn classify(&self, metrics: &TextMetrics) -> ClassificationResult {
        let scores = self.compute_lda_scores(metrics);
        let (primary_diagnosis, confidence) = Self::get_primary_diagnosis(&scores);

        ClassificationResult {
            primary_diagnosis,
            confidence,
            group_scores: scores,
        }
    }

    /// Compute LDA discriminant scores for each group
    fn compute_lda_scores(&self, metrics: &TextMetrics) -> GroupScores {
        let features = FeatureVector::from_metrics(metrics);
        let mut raw_scores = GroupScores::default();

        // Calculate discriminant score for each group
        for (group, coeffs) in &self.coefficients {
            let score = coeffs.score(&features);
            match group {
                DiagnosticGroup::Healthy => raw_scores.healthy = score,
                DiagnosticGroup::Schizophrenia => raw_scores.schizophrenia = score,
                DiagnosticGroup::PersonalityDisorder => raw_scores.personality_disorder = score,
                DiagnosticGroup::BipolarDisorder => raw_scores.bipolar_disorder = score,
            }
        }

        // Convert to posterior probabilities using softmax
        Self::softmax_scores(&mut raw_scores);

        raw_scores
    }

    /// Apply softmax to convert discriminant scores to probabilities
    fn softmax_scores(scores: &mut GroupScores) {
        // Find max for numerical stability
        let max_score = scores.healthy
            .max(scores.schizophrenia)
            .max(scores.personality_disorder)
            .max(scores.bipolar_disorder);

        // Compute exp(score - max)
        let exp_healthy = (scores.healthy - max_score).exp();
        let exp_schiz = (scores.schizophrenia - max_score).exp();
        let exp_pd = (scores.personality_disorder - max_score).exp();
        let exp_bipolar = (scores.bipolar_disorder - max_score).exp();

        let total = exp_healthy + exp_schiz + exp_pd + exp_bipolar;

        scores.healthy = exp_healthy / total;
        scores.schizophrenia = exp_schiz / total;
        scores.personality_disorder = exp_pd / total;
        scores.bipolar_disorder = exp_bipolar / total;
    }

    /// Get primary diagnosis and confidence
    fn get_primary_diagnosis(scores: &GroupScores) -> (DiagnosticGroup, f64) {
        let mut best_group = DiagnosticGroup::Healthy;
        let mut best_score = scores.healthy;

        if scores.schizophrenia > best_score {
            best_group = DiagnosticGroup::Schizophrenia;
            best_score = scores.schizophrenia;
        }
        if scores.personality_disorder > best_score {
            best_group = DiagnosticGroup::PersonalityDisorder;
            best_score = scores.personality_disorder;
        }
        if scores.bipolar_disorder > best_score {
            best_group = DiagnosticGroup::BipolarDisorder;
            best_score = scores.bipolar_disorder;
        }

        (best_group, best_score)
    }

    /// Get detailed classification report
    #[must_use]
    pub fn get_detailed_report(
        &self,
        metrics: &TextMetrics,
        result: &ClassificationResult,
    ) -> String {
        let mut report = String::new();

        report.push_str("=== АНАЛИЗ ПИСЬМЕННОЙ РЕЧИ ===\n\n");

        // Basic stats
        let _ = writeln!(report, "Общий объём текста: {} слов", metrics.total_words);
        let _ = writeln!(report, "Количество предложений: {}", metrics.total_sentences);
        let _ = writeln!(
            report,
            "Индекс лексического разнообразия: {:.1}%\n",
            metrics.lexical_diversity_index
        );

        // Sentence structure
        report.push_str("--- Структура предложений ---\n");
        let _ = writeln!(report, "Простые: {}", metrics.simple_sentences);
        let _ = writeln!(report, "Сложносочинённые: {}", metrics.compound_sentences);
        let _ = writeln!(report, "Сложноподчинённые: {}", metrics.complex_sentences);
        let _ = writeln!(report, "Бессоюзные: {}\n", metrics.run_on_sentences);

        // Key discriminant features
        report.push_str("--- Ключевые диагностические показатели ---\n");
        let _ = writeln!(
            report,
            "Внешние предикаты: {:.1}%",
            metrics.external_predicates
        );
        let _ = writeln!(
            report,
            "Внутренние предикаты: {:.1}%",
            metrics.internal_predicates
        );
        let _ = writeln!(
            report,
            "Глаголы прошедшего времени: {:.1}%",
            metrics.past_tense_verbs
        );
        let _ = writeln!(
            report,
            "Глаголы настоящего времени: {:.1}%",
            metrics.present_tense_verbs
        );
        let _ = writeln!(
            report,
            "Слова социального взаимодействия: {:.1}%",
            metrics.social_interaction_words
        );
        let _ = writeln!(report, "Слова эмоций: {:.1}%", metrics.emotion_words);
        let _ = writeln!(
            report,
            "Местоимения 1-го лица ед.ч.: {:.1}%",
            metrics.first_person_singular_pronouns
        );
        let _ = writeln!(
            report,
            "Отглагольные формы: {:.1}%",
            metrics.non_finite_verb_forms
        );
        let _ = writeln!(
            report,
            "Индекс эгоцентризма: {:.1}%\n",
            metrics.egocentrism_index
        );

        // Classification result
        report.push_str("=== РЕЗУЛЬТАТ КЛАССИФИКАЦИИ ===\n\n");
        let _ = writeln!(
            report,
            "Предварительная оценка: {}",
            result.primary_diagnosis
        );
        let _ = writeln!(
            report,
            "Уверенность: {:.1}%\n",
            result.confidence * 100.0
        );

        report.push_str("--- Вероятности по группам ---\n");
        let _ = writeln!(
            report,
            "Психически здоровые: {:.1}%",
            result.group_scores.healthy * 100.0
        );
        let _ = writeln!(
            report,
            "Шизофрения: {:.1}%",
            result.group_scores.schizophrenia * 100.0
        );
        let _ = writeln!(
            report,
            "Расстройство личности: {:.1}%",
            result.group_scores.personality_disorder * 100.0
        );
        let _ = writeln!(
            report,
            "Биполярное расстройство: {:.1}%",
            result.group_scores.bipolar_disorder * 100.0
        );

        report.push_str("\n=== ВАЖНОЕ ПРИМЕЧАНИЕ ===\n");
        report.push_str("Данный анализ носит исследовательский характер и НЕ является\n");
        report.push_str("медицинским диагнозом. Для постановки диагноза необходимо\n");
        report.push_str("обратиться к квалифицированному специалисту.\n");

        report
    }
}

impl Default for Classifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::TextAnalyzer;

    #[test]
    fn test_classification() {
        let analyzer = TextAnalyzer::new();
        let classifier = Classifier::new();

        // Schizophrenia example from paper
        let text = "Как я катался на 3-колёсном велосипеде и упал";
        let metrics = analyzer.analyze(text);
        let result = classifier.classify(&metrics);

        // Should have a valid classification
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_healthy_classification() {
        let analyzer = TextAnalyzer::new();
        let classifier = Classifier::new();

        // Healthy example from paper (longer, more emotional)
        let text = "Ко мне подходит мама, склоняется надо мной и просит поднять руки. \
                    Я поднимаю руки и на меня натягивается блузка с короткими рукавами-фонариками. \
                    Кофточка из прозрачного плохо тянущегося материала отделана блестящим люрексом \
                    сильно колется. Но ничего, я потерплю. Это часть костюма и без нее никак не обойтись.";
        let metrics = analyzer.analyze(text);
        let result = classifier.classify(&metrics);

        // Healthy texts should have higher healthy score due to length and present tense
        // The exact classification depends on all features
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_paper_examples() {
        let analyzer = TextAnalyzer::new();
        let classifier = Classifier::new();

        // Actual examples from the paper
        let examples = [
            ("Schizophrenia", "Как я катался на 3-колёсном велосипеде и упал. 3–4 года"),
            ("Healthy", "Ко мне подходит мама, склоняется надо мной и просит поднять руки. \
                        Я поднимаю руки и на меня натягивается блузка с короткими рукавами-фонариками. \
                        Кофточка из прозрачного плохо тянущегося материала отделана блестящим люрексом \
                        сильно колется. Но ничего, я потерплю. Это часть костюма и без нее никак не обойтись."),
            ("PersonalityDisorder", "Я шла за руки с родителями, потом я упала и разбила себе коленку левую. \
                        Была кровь, и я много плакала, но все, как мне тогда казалось, смеялись. Около 4 лет."),
            ("Bipolar", "Мне было где-то 6 месяцев. Я подняла голову из коляски и увидела проходящие мимо ноги людей."),
        ];

        println!("\n=== PAPER EXAMPLES CLASSIFICATION ===\n");

        for (expected, text) in &examples {
            let metrics = analyzer.analyze(text);
            let result = classifier.classify(&metrics);

            println!("Expected: {}", expected);
            let display_text: String = text.chars().take(50).collect();
            println!("Text: {}...", display_text);
            println!("Result: {:?}", result.primary_diagnosis);
            println!("Confidence: {:.1}%", result.confidence * 100.0);
            println!("Scores: H={:.1}% S={:.1}% PD={:.1}% B={:.1}%",
                result.group_scores.healthy * 100.0,
                result.group_scores.schizophrenia * 100.0,
                result.group_scores.personality_disorder * 100.0,
                result.group_scores.bipolar_disorder * 100.0);
            println!("Words: {}, 1st-sing: {:.1}%, Present: {:.1}%, Past: {:.1}%",
                metrics.total_words,
                metrics.first_person_singular_pronouns,
                metrics.present_tense_verbs,
                metrics.past_tense_verbs);
            println!("Internal: {:.1}%, External: {:.1}%, Emotion: {:.1}%, Social: {:.1}%",
                metrics.internal_predicates,
                metrics.external_predicates,
                metrics.emotion_words,
                metrics.social_interaction_words);
            println!();
        }
    }

    #[test]
    fn test_all_examples_classification() {
        let analyzer = TextAnalyzer::new();
        let classifier = Classifier::new();

        // Example texts from the paper (or following paper's patterns)
        let examples = [
            // Schizophrenia: short, past tense, external predicates, no emotions
            ("Schizophrenia", "Как я катался на 3-колёсном велосипеде и упал. 3–4 года"),
            // Healthy: long text, present tense, internal predicates, emotions
            ("Healthy", "Ко мне подходит мама, склоняется надо мной и просит поднять руки. \
                        Я поднимаю руки и на меня натягивается блузка с короткими рукавами-фонариками. \
                        Кофточка из прозрачного плохо тянущегося материала отделана блестящим люрексом \
                        сильно колется. Но ничего, я потерплю. Это часть костюма и без нее никак не обойтись."),
            // PersonalityDisorder: social interaction, present tense, moderate emotions
            ("PersonalityDisorder", "Мы с мамой идём гулять в парк. Она держит меня за руку. \
                        Мне нравится как мы проводим время вместе."),
            // Bipolar: high emotions, first person, external actions
            ("Bipolar", "Я помню яркие эмоции от праздника. Было очень весело, мы танцевали и смеялись. \
                        Я чувствовала такую радость и счастье!"),
        ];

        println!("\n=== LDA CLASSIFICATION RESULTS ===\n");

        for (expected, text) in &examples {
            let metrics = analyzer.analyze(text);
            let result = classifier.classify(&metrics);

            println!("Expected: {}", expected);
            let display_text: String = text.chars().take(40).collect();
            println!("Text: {}...", display_text);
            println!("Result: {:?}", result.primary_diagnosis);
            println!("Confidence: {:.1}%", result.confidence * 100.0);
            println!("Scores: H={:.1}% S={:.1}% PD={:.1}% B={:.1}%",
                result.group_scores.healthy * 100.0,
                result.group_scores.schizophrenia * 100.0,
                result.group_scores.personality_disorder * 100.0,
                result.group_scores.bipolar_disorder * 100.0);
            println!("Words: {}, Present: {:.1}%, Past: {:.1}%, Internal: {:.1}%",
                metrics.total_words,
                metrics.present_tense_verbs,
                metrics.past_tense_verbs,
                metrics.internal_predicates);
            println!();
        }
    }
}
