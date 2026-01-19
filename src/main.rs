//! Mental Health Text Analyzer CLI
//!
//! Command-line interface for analyzing structural characteristics
//! of written speech for mental health research.

use mdpredict_russian::{analyze_and_classify, get_full_report, Classifier, TextAnalyzer};
use std::env;
use std::fs;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => run_interactive_mode(),
        2 => {
            match args[1].as_str() {
                "--help" | "-h" => print_help(),
                "--version" | "-v" => print_version(),
                "--demo" => run_demo(),
                "--json" => run_json_mode(),
                _ => {
                    // Treat as file path
                    analyze_file(&args[1], false);
                }
            }
        }
        3 => {
            if args[1] == "--json" || args[2] == "--json" {
                let file_path = if args[1] == "--json" { &args[2] } else { &args[1] };
                analyze_file(file_path, true);
            } else {
                eprintln!("Unknown arguments. Use --help for usage information.");
            }
        }
        _ => {
            eprintln!("Too many arguments. Use --help for usage information.");
        }
    }
}

fn print_help() {
    println!(
        r#"mdpredict - Mental Disorder Prediction (Russian)

ИСПОЛЬЗОВАНИЕ:
    mdpredict [OPTIONS] [FILE]

ОПИСАНИЕ:
    Предсказание психических расстройств на основе структурных
    характеристик письменной речи на русском языке.

    Основан на методологии исследования:
    "Диагностическое значение структурных характеристик письменной
    речи пациентов с шизофренией" (Смерчинская, Трегубенко, Исаева, 2026)

ОПЦИИ:
    -h, --help      Показать справку
    -v, --version   Показать версию
    --demo          Запустить демонстрацию с примерами из статьи
    --json          Вывести результат в формате JSON

ПРИМЕРЫ:
    mdpredict                   Интерактивный режим
    mdpredict text.txt          Анализ файла
    mdpredict --json text.txt   Анализ с JSON-выводом
    mdpredict --demo            Демонстрация

ВАЖНОЕ ПРИМЕЧАНИЕ:
    Данный инструмент предназначен ТОЛЬКО для исследовательских целей.
    Он НЕ является заменой профессиональной медицинской диагностики.
    Для постановки диагноза обратитесь к квалифицированному специалисту.
"#
    );
}

fn print_version() {
    println!("mdpredict v{} - Mental Disorder Prediction (Russian)", env!("CARGO_PKG_VERSION"));
    println!("Основан на исследовании Смерчинской, Трегубенко, Исаевой (2026)");
}

fn run_interactive_mode() {
    println!("=== Анализатор структурных характеристик письменной речи ===");
    println!();
    println!("Введите текст для анализа (для завершения введите пустую строку):");
    println!();

    let stdin = io::stdin();
    let mut text = String::new();

    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                if line.is_empty() {
                    break;
                }
                text.push_str(&line);
                text.push('\n');
            }
            Err(e) => {
                eprintln!("Ошибка чтения: {}", e);
                return;
            }
        }
    }

    if text.trim().is_empty() {
        println!("Текст не введён.");
        return;
    }

    let report = get_full_report(&text);
    println!("\n{}", report);
}

fn analyze_file(path: &str, json_output: bool) {
    match fs::read_to_string(path) {
        Ok(text) => {
            if json_output {
                let (metrics, result) = analyze_and_classify(&text);
                let output = serde_json::json!({
                    "metrics": metrics,
                    "classification": result
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                let report = get_full_report(&text);
                println!("{}", report);
            }
        }
        Err(e) => {
            eprintln!("Ошибка чтения файла '{}': {}", path, e);
        }
    }
}

fn run_json_mode() {
    println!("Введите текст для анализа (завершите вводом EOF или Ctrl+D):");

    let stdin = io::stdin();
    let mut text = String::new();

    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                text.push_str(&line);
                text.push('\n');
            }
            Err(_) => break,
        }
    }

    if !text.trim().is_empty() {
        let (metrics, result) = analyze_and_classify(&text);
        let output = serde_json::json!({
            "metrics": metrics,
            "classification": result
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}

fn run_demo() {
    println!("=== ДЕМОНСТРАЦИЯ АНАЛИЗАТОРА ===\n");

    let analyzer = TextAnalyzer::new();
    let classifier = Classifier::new();

    // Example from the paper - schizophrenia patient
    println!("--- Пример 1: Текст пациента с шизофренией (из статьи) ---\n");
    let schizo_text = "Как я катался на 3-колёсном велосипеде и упал. 3–4 года";
    println!("Текст: \"{}\"\n", schizo_text);

    let metrics1 = analyzer.analyze(schizo_text);
    let result1 = classifier.classify(&metrics1);
    print_brief_analysis(&metrics1, &result1);

    println!("\n");

    // Example from the paper - healthy participant
    println!("--- Пример 2: Текст психически здорового человека (из статьи) ---\n");
    let healthy_text = "Ко мне подходит мама, склоняется надо мной и просит поднять руки. \
                        Я поднимаю руки и на меня натягивается блузка с короткими рукавами-фонариками. \
                        Кофточка из прозрачного плохо тянущегося материала отделана блестящим люрексом \
                        сильно колется. Но ничего, я потерплю. Это часть костюма и без нее никак не обойтись.";
    println!("Текст: \"{}\"\n", &healthy_text[..100]);
    println!("       ...(продолжение)...\n");

    let metrics2 = analyzer.analyze(healthy_text);
    let result2 = classifier.classify(&metrics2);
    print_brief_analysis(&metrics2, &result2);

    println!("\n");

    // Example from paper - personality disorder
    println!("--- Пример 3: Текст пациента с расстройством личности (из статьи) ---\n");
    let pd_text = "Я шла за руки с родителями, потом я упала и разбила себе коленку левую. \
                   Была кровь, и я много плакала, но все, как мне тогда казалось, смеялись. Около 4 лет.";
    println!("Текст: \"{}\"\n", pd_text);

    let metrics3 = analyzer.analyze(pd_text);
    let result3 = classifier.classify(&metrics3);
    print_brief_analysis(&metrics3, &result3);

    println!("\n");

    // Example from paper - bipolar disorder
    println!("--- Пример 4: Текст пациента с биполярным расстройством (из статьи) ---\n");
    let bipolar_text = "Мне было где-то 6 месяцев. Я подняла голову из коляски и увидела проходящие мимо ноги людей.";
    println!("Текст: \"{}\"\n", bipolar_text);

    let metrics4 = analyzer.analyze(bipolar_text);
    let result4 = classifier.classify(&metrics4);
    print_brief_analysis(&metrics4, &result4);

    println!("\n=== Ключевые различия по статье ===\n");
    println!("Шизофрения vs Здоровые (точность 92%):");
    println!("  - Внешние предикаты (↑ при шизофрении)");
    println!("  - Глаголы прошедшего времени (↑ при шизофрении)");
    println!("  - Глаголы настоящего времени (↓ при шизофрении)");
    println!("  - Слова социального взаимодействия");
    println!("  - Слова эмоций (↓ при шизофрении)");
    println!();
    println!("Шизофрения vs БАР vs РЛ (точность 70%):");
    println!("  - Сложносочинённые предложения (↑ при шизофрении)");
    println!("  - Отглагольные формы (↑ при БАР)");
    println!("  - Местоимения 1-го лица ед.ч. (↑ при БАР и РЛ)");
}

fn print_brief_analysis(
    metrics: &mdpredict_russian::TextMetrics,
    result: &mdpredict_russian::ClassificationResult,
) {
    println!("Объём текста: {} слов", metrics.total_words);
    println!(
        "Лексическое разнообразие: {:.1}%",
        metrics.lexical_diversity_index
    );
    println!("Внешние предикаты: {:.1}%", metrics.external_predicates);
    println!("Внутренние предикаты: {:.1}%", metrics.internal_predicates);
    println!("Глаголы прош. времени: {:.1}%", metrics.past_tense_verbs);
    println!("Глаголы наст. времени: {:.1}%", metrics.present_tense_verbs);
    println!(
        "Местоимения 1л. ед.ч.: {:.1}%",
        metrics.first_person_singular_pronouns
    );
    println!();
    println!("Результат классификации: {}", result.primary_diagnosis);
    println!("Уверенность: {:.1}%", result.confidence * 100.0);
    println!();
    println!("Вероятности:");
    println!(
        "  Здоровые: {:.1}%",
        result.group_scores.healthy * 100.0
    );
    println!(
        "  Шизофрения: {:.1}%",
        result.group_scores.schizophrenia * 100.0
    );
    println!(
        "  Расстройство личности: {:.1}%",
        result.group_scores.personality_disorder * 100.0
    );
    println!(
        "  Биполярное расстройство: {:.1}%",
        result.group_scores.bipolar_disorder * 100.0
    );
}
