use eframe::egui;

use kabtangan_core::dictionary::StaticDictionary;
use kabtangan_core::prediction::{PredictionConfig, PredictionEngine};
use kabtangan_core::storage::Storage;
use kabtangan_core::transliteration::latin_to_sulat;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_min_inner_size([400.0, 400.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Kabtangan Keyboard Simulator",
        options,
        Box::new(|_cc| Ok(Box::new(KabtanganApp::new()))),
    )
}

struct KabtanganApp {
    dict: StaticDictionary,
    storage: Storage,
    input_text: String,
}

impl KabtanganApp {
    fn new() -> Self {
        // Load the same static dictionary used by the mobile/desktop keyboards
        let dict_bytes = include_bytes!("../../data/dictionary/bahasa_sug_v1.json");
        let dict = StaticDictionary::load(dict_bytes).expect("Failed to load dictionary");
        let storage = Storage::open_in_memory().expect("Failed to open local storage");

        Self {
            dict,
            storage,
            input_text: String::new(),
        }
    }

    /// Gets the word the user is currently typing
    fn current_word(&self) -> &str {
        self.input_text.split_whitespace().last().unwrap_or("")
    }
}

impl eframe::App for KabtanganApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // -- Header --
            ui.heading("Kabtangan Keyboard - Live Test UI");
            ui.add_space(10.0);

            // -- Transliteration Preview --
            ui.group(|ui| {
                ui.label(egui::RichText::new("Sulat Sūg Transliteration").color(egui::Color32::from_rgb(11, 122, 92)));
                let sulat = latin_to_sulat(&self.input_text);
                
                // Show placeholders since font isn't bundled in this tester
                ui.label(egui::RichText::new(if sulat.is_empty() { "..." } else { &sulat }).size(24.0));
            });

            ui.add_space(20.0);

            // -- Prediction Bar --
            ui.horizontal(|ui| {
                ui.label("Predictions: ");
                
                let predictor = PredictionEngine::new(&self.dict, &self.storage, PredictionConfig::default());
                let current_word = self.current_word();
                let preds = predictor.predict(current_word);
                
                if preds.is_empty() {
                    ui.label(egui::RichText::new("None").italics());
                } else {
                    for p in preds {
                        if ui.button(egui::RichText::new(&p.word).color(egui::Color32::from_rgb(201, 162, 39))).clicked() {
                            // Replace current typing word with prediction
                            let mut words: Vec<&str> = self.input_text.split_whitespace().collect();
                            if !self.input_text.ends_with(' ') && !words.is_empty() {
                                words.pop(); // remove incomplete word
                            }
                            
                            let mut new_text = words.join(" ");
                            if !new_text.is_empty() {
                                new_text.push(' ');
                            }
                            new_text.push_str(&p.word);
                            new_text.push(' ');
                            
                            self.input_text = new_text;
                        }
                    }
                }
            });

            // -- Main Text Field --
            ui.add_space(10.0);
            ui.add_sized(
                [ui.available_width(), 150.0], 
                egui::TextEdit::multiline(&mut self.input_text)
                    .font(egui::TextStyle::Heading)
                    .hint_text("Type in Bahasa Sūg here...")
            );

            // -- On-Screen Keyboard --
            ui.add_space(20.0);
            
            let rows = vec![
                vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
                vec!["a", "s", "d", "f", "g", "h", "j", "k", "l"],
                vec!["'", "z", "x", "c", "v", "b", "n", "m", "⌫"],
                vec!["🌐", "SPACE", "⏎"]
            ];

            for row in rows {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    
                    // Add some left padding to stagger the rows a bit
                    if row.contains(&"a") { ui.add_space(15.0); }
                    if row.contains(&"'") { ui.add_space(30.0); }
                    
                    for key in row {
                        let button_size = match key {
                            "SPACE" => egui::vec2(250.0, 45.0),
                            "⌫" | "⏎" | "🌐" => egui::vec2(60.0, 45.0),
                            _ => egui::vec2(40.0, 45.0),
                        };

                        if ui.add_sized(button_size, egui::Button::new(key)).clicked() {
                            match key {
                                "⌫" => {
                                    self.input_text.pop();
                                },
                                "SPACE" => {
                                    self.input_text.push(' ');
                                },
                                "⏎" => {
                                    self.input_text.push('\n');
                                },
                                "🌐" => {
                                    // Just a placeholder in this tester
                                },
                                _ => {
                                    self.input_text.push_str(key);
                                }
                            }
                        }
                    }
                });
            }
        });
    }
}
