//! Main application UI and state management.
//! Handles the flashcard app interface, deck management, and learning sessions.

use crate::database::db;
use crate::export::json::{export_json_to_path, import_json};
use crate::models::{Deck, DeckSet, Flashcard, LearningSession};
use chrono::{DateTime, Local};
use eframe::egui;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Application screen states
#[derive(Default)]
enum AppScreen {
    #[default]
    Main,
    LearningSession,
}

/// Main application state
#[derive(Default)]
pub struct MyApp {
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
    all_decks: DeckSet,
    selected_deck_index: Option<usize>,
    current_term: String,
    current_definition: String,
    new_deck_name: String,
    conn: Option<Arc<Mutex<Connection>>>,

    current_screen: AppScreen,
    learning_session: Option<LearningSession>,

    current_date_display: String,

    show_export_dialog: bool,
    show_import_result_dialog: bool,
    import_result_message: String,
}

/// Formats SystemTime as YYYY-MM-DD string
fn format_system_time(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%Y-%m-%d").to_string()
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.current_screen {
            AppScreen::Main => self.render_main_screen(ctx),
            AppScreen::LearningSession => self.render_learning_screen(ctx),
        }

        // Handle window close requests with confirmation dialog
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.allowed_to_close {
                // Allow close
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_confirmation_dialog = true;
            }
        }

        if self.show_confirmation_dialog {
            egui::Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("No").clicked() {
                            self.show_confirmation_dialog = false;
                            self.allowed_to_close = false;
                        }

                        if ui.button("Yes").clicked() {
                            self.show_confirmation_dialog = false;
                            self.allowed_to_close = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
        }
        // exporting a deck
        if self.show_export_dialog {
            let mut export_deck_index: Option<usize> = None;
            let mut should_cancel = false;

            egui::Window::new("Export Deck")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Select a deck to export:");
                    ui.separator();

                    for (i, deck) in self.all_decks.decks.iter().enumerate() {
                        if ui
                            .button(format!("{} ({} cards)", deck.name, deck.flashcards.len()))
                            .clicked()
                        {
                            export_deck_index = Some(i);
                        }
                    }

                    ui.separator();

                    if ui.button("Cancel").clicked() {
                        should_cancel = true;
                    }
                });

            if let Some(i) = export_deck_index {
                self.handle_export(i);
            }
            if should_cancel {
                self.show_export_dialog = false;
            }
        }

        if self.show_import_result_dialog {
            egui::Window::new("Import/Export Result")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&self.import_result_message);
                    ui.add_space(10.0);
                    if ui.button("OK").clicked() {
                        self.show_import_result_dialog = false;
                    }
                });
        }
    }
}

impl MyApp {
    /// Creates a new application instance with decks loaded from database
    pub fn new_with_deckset(deckset: DeckSet, conn: Connection) -> Self {
        let current_date = db::get_current_date(&conn)
            .map(|d| format!("{:?}", d))
            .unwrap_or_else(|_| "Unknown".to_string());
        let has_decks = !deckset.decks.is_empty();
        Self {
            all_decks: deckset,
            selected_deck_index: if has_decks { Some(0) } else { None },
            current_term: String::new(),
            current_definition: String::new(),
            new_deck_name: String::new(),
            show_confirmation_dialog: false,
            allowed_to_close: false,
            conn: Some(Arc::new(Mutex::new(conn))),
            current_screen: AppScreen::Main,
            learning_session: None,
            current_date_display: current_date,
            show_export_dialog: false,
            show_import_result_dialog: false,
            import_result_message: String::new(),
        }
    }

    /// Renders the main screen with deck management interface
    fn render_main_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Fetch and display current date from database
            if let Some(conn) = &self.conn {
                if let Ok(conn_guard) = conn.lock() {
                    if let Ok(current_date) = db::get_current_date(&conn_guard) {
                        self.current_date_display = format_system_time(current_date);
                    }
                }
            }
            ui.label(format!("{}", self.current_date_display));

            if ui.button("Next Day").clicked() {
                if let Some(conn) = &self.conn {
                    let conn = conn.lock().unwrap();
                    let _ = db::advance_day(&conn);
                    if let Ok(current_date) = db::get_current_date(&conn) {
                        self.current_date_display = format_system_time(current_date);
                    }
                }
            }
        });
        ui.separator();

        // Import/Export buttons
        ui.horizontal(|ui| {
            if ui.button("Export Deck").clicked() {
                self.show_export_dialog = true;
            }
            if ui.button("Import Deck").clicked() {
                self.handle_import();
            }
        });

        ui.separator();

        // Deck creation section
        ui.heading("Create New Deck");
        ui.horizontal(|ui| {
            ui.label("Deck name:");
            ui.text_edit_singleline(&mut self.new_deck_name);
            if ui.button("Create Deck").clicked() {
                if !self.new_deck_name.is_empty() {
                    self.all_decks.decks.push(Deck {
                        name: self.new_deck_name.clone(),
                        flashcards: Vec::new(),
                    });

                    // Save to database
                    if let Some(conn) = &self.conn {
                        let conn = conn.lock().unwrap();
                        let _ = conn.execute(
                            "INSERT INTO decks (name) VALUES (?1)",
                            params![self.new_deck_name],
                        );
                    }

                    self.new_deck_name.clear();
                }
            }
        });

        ui.separator();

        ui.heading(format!("Decks ({})", self.all_decks.decks.len()));

        // We store actions to execute after UI rendering to avoid borrowing conflicts
        let mut action_select: Option<usize> = None;
        let mut action_learn: Option<usize> = None;

        egui::ScrollArea::vertical()
            .id_source("decks_list")
            .max_height(150.0)
            .show(ui, |ui| {
                for (i, deck) in self.all_decks.decks.iter().enumerate() {
                    let is_selected = self.selected_deck_index == Some(i);

                    ui.horizontal(|ui| {
                        if ui.selectable_label(
                            is_selected,
                            format!("{}. {} ({} cards)", i + 1, deck.name, deck.flashcards.len())
                        ).clicked() {
                            action_select = Some(i);
                        }

                        if ui.button("Learn").clicked() {
                            action_learn = Some(i);
                        }
                    });
                }
            });

        // Execute deferred actions
        if let Some(i) = action_select {
            self.selected_deck_index = Some(i);
        }
        if let Some(i) = action_learn {
            self.start_learning_session(i);
        }

        ui.separator();

        // Flashcard management for selected deck
        if let Some(deck_index) = self.selected_deck_index {
            if let Some(current_deck) = self.all_decks.decks.get_mut(deck_index) {
                ui.heading(format!("Selected Deck: {}", current_deck.name));

                ui.horizontal(|ui| {
                    ui.label("Term:");
                    ui.text_edit_singleline(&mut self.current_term);
                });

                ui.horizontal(|ui| {
                    ui.label("Definition:");
                    ui.text_edit_singleline(&mut self.current_definition);
                });
                if ui.button("Add Flashcard").clicked() {
                    if !self.current_term.is_empty() && !self.current_definition.is_empty() {
                        current_deck.flashcards.push(Flashcard {
                            term: self.current_term.clone(),
                            definition: self.current_definition.clone(),
                        });
                        // Save to database
                        if let Some(conn) = &self.conn {
                            let conn = conn.lock().unwrap();
                            let _ = conn.execute(
                                "INSERT OR IGNORE INTO flashcards (deck_name, term, definition) VALUES (?1, ?2, ?3)",
                                params![current_deck.name, self.current_term, self.current_definition],
                            );
                        }
                        self.current_term.clear();
                        self.current_definition.clear();
                    }
                }

                ui.separator();

                ui.heading(format!("Flashcards ({})", current_deck.flashcards.len()));

                egui::ScrollArea::vertical()
                    .id_source("flashcards_list")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (i, flashcard) in current_deck.flashcards.iter().enumerate() {
                            ui.group(|ui| {
                                ui.label(format!("{}. Term: {}", i + 1, flashcard.term));
                                ui.label(format!("   Definition: {}", flashcard.definition));
                            });
                        }
                    });
            }
        } else {
            ui.label("Select a deck to add flashcards");
        }
    });
    }

    /// Renders the learning session screen with flashcard review interface
    fn render_learning_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(session) = &mut self.learning_session {
                ui.heading(format!("Learning: {}", session.deck_name));

                ui.label(session.phase_message());

                ui.label(format!(
                    "Progress: {} / {} learned ({} remaining)",
                    session.learned_count(),
                    session.total_count(),
                    session.remaining_count()
                ));

                ui.add_space(20.0);

                if session.is_completed() {
                    ui.heading("Congratulations!");
                    ui.label("You've learned all cards in this deck!");

                    ui.add_space(20.0);

                    if ui.button("Back to Main Screen").clicked() {
                        self.current_screen = AppScreen::Main;
                        self.learning_session = None;
                    }
                } else if let Some(card) = session.current_card() {
                    // Clone values to avoid borrowing issues
                    let show_def = session.show_definition;
                    let is_learned = card.is_learned;
                    let term = card.flashcard.term.clone();
                    let definition = card.flashcard.definition.clone();

                    ui.group(|ui| {
                        ui.set_min_height(200.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);

                            ui.heading("Term:");
                            ui.label(&term);

                            ui.add_space(20.0);

                            if show_def {
                                ui.heading("Definition:");
                                ui.label(&definition);
                            } else {
                                ui.label("(Click 'Show Definition' to reveal)");
                            }

                            ui.add_space(20.0);
                        });
                    });

                    ui.add_space(20.0);

                    // Store actions to execute after UI rendering
                    let mut action_toggle_def = false;
                    let mut action_grade: Option<u8> = None;
                    let mut action_back = false;

                    if !show_def {
                        if ui.button("Show Definition").clicked() {
                            action_toggle_def = true;
                        }
                    }

                    // Quality rating buttons (0-5) - only show after revealing definition
                    if show_def && !is_learned {
                        ui.label("Rate your response:");
                        ui.horizontal(|ui| {
                            if ui.button("0 - Blackout").clicked() {
                                action_grade = Some(0);
                            }
                            if ui.button("1 - Wrong").clicked() {
                                action_grade = Some(1);
                            }
                            if ui.button("2 - Wrong (familiar)").clicked() {
                                action_grade = Some(2);
                            }
                        });
                        ui.horizontal(|ui| {
                            if ui.button("3 - Difficult").clicked() {
                                action_grade = Some(3);
                            }
                            if ui.button("4 - Correct").clicked() {
                                action_grade = Some(4);
                            }
                            if ui.button("5 - Perfect").clicked() {
                                action_grade = Some(5);
                            }
                        });
                    }

                    ui.add_space(20.0);

                    if ui.button("Back to Main Screen").clicked() {
                        action_back = true;
                    }

                    // Execute deferred actions
                    if action_toggle_def {
                        session.toggle_definition();
                    }
                    if let Some(quality) = action_grade {
                        session.grade_current_card(quality);
                        // After grading, move to next card
                        session.next_card();
                    }
                    if action_back {
                        self.current_screen = AppScreen::Main;
                        self.learning_session = None;
                    }
                }
            }
        });
    }

    /// Starts a learning session with cards due for review
    fn start_learning_session(&mut self, deck_index: usize) {
        if let Some(deck) = self.all_decks.decks.get(deck_index) {
            if let Some(conn) = &self.conn {
                let conn_guard = conn.lock().unwrap();

                // Fetch only flashcards due for review today
                let due_cards =
                    crate::database::db::get_flashcards_due_for_review(&deck.name, &conn_guard)
                        .unwrap_or_default();

                drop(conn_guard);

                if !due_cards.is_empty() {
                    self.learning_session = Some(LearningSession::new_from_due_cards(
                        deck.name.clone(),
                        due_cards,
                        Arc::clone(self.conn.as_ref().unwrap()),
                    ));
                    self.current_screen = AppScreen::LearningSession;
                }
            }
        }
    }

    /// Handles deck export to JSON file
    fn handle_export(&mut self, deck_index: usize) {
        if let Some(deck) = self.all_decks.decks.get(deck_index) {
            // Open file save dialog
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(format!("{}.json", deck.name))
                .add_filter("JSON files", &["json"])
                .save_file()
            {
                match export_json_to_path(deck, path.to_str().unwrap()) {
                    Ok(_) => {
                        self.import_result_message =
                            format!("Deck '{}' exported successfully!", deck.name);
                        self.show_import_result_dialog = true;
                    }
                    Err(e) => {
                        self.import_result_message = format!("Export failed: {}", e);
                        self.show_import_result_dialog = true;
                    }
                }
            }
        }
        self.show_export_dialog = false;
    }

    /// Handles deck import from JSON file
    fn handle_import(&mut self) {
        // Open file selection dialog
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON files", &["json"])
            .pick_file()
        {
            match import_json(path.to_str().unwrap()) {
                Ok(deck) => {
                    // Check if deck with this name already exists
                    if self.all_decks.decks.iter().any(|d| d.name == deck.name) {
                        self.import_result_message = format!(
                            "Deck '{}' already exists! Please rename it in the JSON file.",
                            deck.name
                        );
                        self.show_import_result_dialog = true;
                        return;
                    }

                    // Add deck to database
                    if let Some(conn) = &self.conn {
                        let conn_guard = conn.lock().unwrap();

                        // Create deck
                        if let Err(e) = db::new_deck(&deck.name, &conn_guard) {
                            self.import_result_message = format!("Failed to create deck: {}", e);
                            self.show_import_result_dialog = true;
                            return;
                        }

                        // Add flashcards
                        for flashcard in &deck.flashcards {
                            if let Err(e) = db::add_flashcard(
                                &deck.name,
                                &flashcard.term,
                                &flashcard.definition,
                                &conn_guard,
                            ) {
                                self.import_result_message = format!(
                                    "Failed to import flashcard '{}': {}",
                                    flashcard.term, e
                                );
                                self.show_import_result_dialog = true;
                                return;
                            }
                        }

                        drop(conn_guard);
                    }

                    // Add to in-memory DeckSet
                    self.all_decks.decks.push(deck.clone());

                    self.import_result_message = format!(
                        "Deck '{}' imported successfully with {} cards!",
                        deck.name,
                        deck.flashcards.len()
                    );
                    self.show_import_result_dialog = true;
                }
                Err(e) => {
                    self.import_result_message = format!(
                        "Import failed: {}\n\nPlease check if the file has correct structure:\n{{\n  \"name\": \"Deck Name\",\n  \"flashcards\": [...]\n}}",
                        e
                    );
                    self.show_import_result_dialog = true;
                }
            }
        }
    }
}
