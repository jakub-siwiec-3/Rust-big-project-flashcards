//! Wrapper for flashcards that tracks learning progress.
use super::Flashcard;
use std::time::SystemTime;

#[derive(Clone)]
pub struct LearningCard {
    pub flashcard: Flashcard,
    pub is_learned: bool,
    pub last_learned_at: Option<SystemTime>,
}

impl LearningCard {
    pub fn new(flashcard: Flashcard) -> Self {
        Self {
            flashcard,
            is_learned: false,
            last_learned_at: None,
        }
    }

    pub fn mark_as_learned(&mut self) {
        self.is_learned = true;
        self.last_learned_at = Some(SystemTime::now());
    }
}
