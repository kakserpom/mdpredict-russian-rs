use rsmorphy::prelude::*;
use rsmorphy_dict_ru::DictRu;

pub fn test_rsmorphy() {
    // Create analyzer with Russian dictionary
    let analyzer = MorphAnalyzer::from_dictionary(DictRu);
    
    // Test words
    let words = ["думаю", "иду", "катался", "подходит", "склоняется", "я", "мне"];
    
    for word in &words {
        let parses = analyzer.parse(word);
        println!("\n=== {} ===", word);
        for (i, parse) in parses.iter().enumerate() {
            if i > 2 { break; } // Limit to first 3 parses
            println!("  Parse {}: {:?}", i, parse.lex.grammemes());
        }
    }
}
