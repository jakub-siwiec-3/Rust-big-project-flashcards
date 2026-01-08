//! Deck is a set of flashcards
use super::Flashcard;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Deck {
    pub name: String,
    pub flashcards: Vec<Flashcard>,
}

impl Default for Deck {
    fn default() -> Self {
        Self {
            name: "My Deck".to_string(),
            flashcards: Vec::new(),
        }
    }
}
