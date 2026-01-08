//! Container for all available decks
use super::Deck;

#[derive(Clone)]
pub struct DeckSet {
    pub decks: Vec<Deck>,
}

impl Default for DeckSet {
    fn default() -> Self {
        Self { decks: Vec::new() }
    }
}
