mod app;
use flashcards_app::*;

use app::MyApp;
use database::db::{get_all_decks, init_database, load_all_decks, new_deck};

fn main() -> eframe::Result<()> {
    let conn = init_database().expect("Failed to initialize database");

    if get_all_decks(&conn).unwrap_or_default().is_empty() {
        let _ = new_deck("Polish Vocabulary", &conn);

        let _ = database::db::add_flashcard("Polish Vocabulary", "cześć", "hello", &conn);
        let _ = database::db::add_flashcard("Polish Vocabulary", "dziękuję", "thank you", &conn);
        let _ = database::db::add_flashcard("Polish Vocabulary", "proszę", "please", &conn);

        println!("Sample data created!");
    }

    let deck_set = load_all_decks(&conn).expect("Failed to load decks from database");

    println!("Loaded {} decks from database", deck_set.decks.len());
    for deck in &deck_set.decks {
        println!("  - {} ({} cards)", deck.name, deck.flashcards.len());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Flashcards App",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new_with_deckset(deck_set, conn)))),
    )
}
