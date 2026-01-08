//! Learning session management for spaced repetition practice.
//! Handles multi-round flashcard review with SM-2 algorithm integration.

use super::{LearningCard, ReviewData};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Manages a learning session with multiple review rounds.
/// Cards that aren't mastered (grade < 3) are repeated in subsequent rounds.
pub struct LearningSession {
    pub deck_name: String,
    pub all_cards: Vec<(i64, LearningCard, ReviewData)>,
    pub current_round_cards: Vec<usize>,
    pub current_index: usize,
    pub show_definition: bool,
    pub conn: Arc<Mutex<Connection>>,
    pub round_number: usize,
}

impl LearningSession {
    /// Creates a new learning session from cards that are due for review.
    pub fn new_from_due_cards(
        deck_name: String,
        cards: Vec<(i64, crate::models::Flashcard, ReviewData)>,
        conn: Arc<Mutex<Connection>>,
    ) -> Self {
        // Wrap flashcards in LearningCard for progress tracking
        let learning_cards: Vec<_> = cards
            .into_iter()
            .map(|(id, fc, rd)| (id, LearningCard::new(fc), rd))
            .collect();

        let indices: Vec<usize> = (0..learning_cards.len()).collect();

        Self {
            deck_name,
            all_cards: learning_cards,
            current_round_cards: indices,
            current_index: 0,
            show_definition: false,
            conn,
            round_number: 1,
        }
    }

    pub fn current_card(&self) -> Option<&LearningCard> {
        self.current_round_cards
            .get(self.current_index)
            .and_then(|&idx| self.all_cards.get(idx).map(|(_, card, _)| card))
    }

    pub fn toggle_definition(&mut self) {
        self.show_definition = !self.show_definition;
    }

    pub fn next_card(&mut self) {
        if self.current_index < self.current_round_cards.len() - 1 {
            self.current_index += 1;
            self.show_definition = false;
        } else {
            // End of round - check if there are cards to review
            self.start_next_round();
        }
    }

    /// Starts a new round with cards that weren't mastered (grade < 3).
    /// If no cards remain, the session is complete.
    fn start_next_round(&mut self) {
        // Collect cards that are NOT learned (grade < 3)
        let failed_indices: Vec<usize> = self
            .current_round_cards
            .iter()
            .copied()
            .filter(|&idx| {
                self.all_cards
                    .get(idx)
                    .map(|(_, card, _)| !card.is_learned)
                    .unwrap_or(false)
            })
            .collect();

        if !failed_indices.is_empty() {
            // There are cards to review - start new round
            self.current_round_cards = failed_indices;
            self.current_index = 0;
            self.show_definition = false;
            self.round_number += 1;

            // Reset is_learned for these cards (they'll be shown again)
            for &idx in &self.current_round_cards {
                if let Some((_, card, _)) = self.all_cards.get_mut(idx) {
                    card.is_learned = false;
                }
            }
        }
        // If failed_indices is empty, session ends (is_completed() = true)
    }

    /// Grades the current card and updates its review data using SM-2 algorithm.
    /// Cards with grade >= 3 are marked as learned for this session.
    pub fn grade_current_card(&mut self, quality: u8) {
        if let Some(&actual_idx) = self.current_round_cards.get(self.current_index) {
            if let Some((_, card, review_data)) = self.all_cards.get_mut(actual_idx) {
                // Mark as learned only if grade >= 3
                if quality >= 3 {
                    card.mark_as_learned();
                } else {
                    card.is_learned = false; // Will be repeated in next round
                }

                // Calculate next review using SM-2
                let conn = self.conn.lock().unwrap();
                let current_date = crate::database::db::get_current_date(&conn).unwrap();

                let new_review =
                    crate::models::sm2::calculate_next_review(review_data, quality, current_date);

                // Update in database
                let _ = crate::database::db::update_review_data(&new_review, &conn);

                // Update in memory
                *review_data = new_review;
            }
        }
    }

    pub fn learned_count(&self) -> usize {
        self.current_round_cards
            .iter()
            .filter(|&&idx| {
                self.all_cards
                    .get(idx)
                    .map(|(_, card, _)| card.is_learned)
                    .unwrap_or(false)
            })
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.current_round_cards.len()
    }

    pub fn remaining_count(&self) -> usize {
        self.total_count() - self.learned_count()
    }

    /// Returns true when all cards have been mastered or current round is empty.
    pub fn is_completed(&self) -> bool {
        // Completed when current round is empty (all cards passed)
        self.current_round_cards.is_empty() ||
        // or all cards in current round are learned
        self.learned_count() == self.total_count()
    }

    pub fn phase_message(&self) -> String {
        if self.round_number == 1 {
            format!("Round {}: {} cards", self.round_number, self.total_count())
        } else {
            format!(
                "Round {} (Review): {} cards to retry",
                self.round_number,
                self.total_count()
            )
        }
    }
}
