use std::io::{self, Write};

use kabtangan_core::dictionary::StaticDictionary;
use kabtangan_core::prediction::{PredictionConfig, PredictionEngine};
use kabtangan_core::spell::SpellChecker;
use kabtangan_core::storage::Storage;
use kabtangan_core::transliteration::latin_to_sulat;

fn main() {
    println!("========================================");
    println!("  Kabtangan Keyboard - Core Engine CLI  ");
    println!("========================================");
    
    // 1. Load the bundled static dictionary
    print!("Loading dictionary... ");
    io::stdout().flush().unwrap();
    let dict_bytes = include_bytes!("../../data/dictionary/bahasa_sug_v1.json");
    let dict = StaticDictionary::load(dict_bytes).expect("Failed to load dictionary JSON");
    println!("Loaded {} words.", dict.len());

    // 2. Initialize the local SQLite storage (in-memory for the CLI test)
    print!("Initializing storage... ");
    io::stdout().flush().unwrap();
    let storage = Storage::open_in_memory().expect("Failed to open SQLite storage");
    println!("Done.\n");

    // 3. Setup engines
    let spell_checker = SpellChecker::new(&dict, &storage);
    let predictor = PredictionEngine::new(&dict, &storage, PredictionConfig::default());

    println!("Type a Bahasa Sūg word to test the engine (or type 'quit' to exit):");
    
    loop {
        print!("\n> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let text = input.trim();

        if text.eq_ignore_ascii_case("quit") || text.eq_ignore_ascii_case("exit") {
            println!("Magsukul! Goodbye.");
            break;
        }

        if text.is_empty() {
            continue;
        }

        // --- Transliteration ---
        let sulat = latin_to_sulat(text);
        println!("  Sulat Sūg   : {}", sulat);

        // --- Spell Check ---
        let spell_res = spell_checker.check(text).unwrap();
        if spell_res.is_correct {
            println!("  Spell check : ✅ Correct");
        } else {
            println!("  Spell check : ❌ Unknown");
            if !spell_res.suggestions.is_empty() {
                println!("  Did you mean: {}", spell_res.suggestions.join(", "));
            }
        }

        // --- Word Prediction ---
        let preds = predictor.predict(text);
        if !preds.is_empty() {
            let pred_strs: Vec<String> = preds.into_iter().map(|p| p.word).collect();
            println!("  Predictions : {}", pred_strs.join(", "));
        } else {
            println!("  Predictions : (None)");
        }
    }
}
