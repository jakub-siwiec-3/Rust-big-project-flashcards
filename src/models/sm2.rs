//! SM-2 (SuperMemo 2) spaced repetition algorithm implementation.
//!
//! The SM-2 algorithm calculates optimal review intervals based on recall quality:
//! - Each card has an easiness factor (EF) that adjusts based on performance
//! - Quality grades 0-2: Reset interval and repetitions (card needs relearning)
//! - Quality grades 3-5: Increase interval progressively (1 day → 6 days → EF multiplier)
//! - EF is adjusted after each review and has a minimum value of 1.3
//! - Higher quality responses lead to longer intervals between reviews

use super::ReviewData;
use std::time::{Duration, SystemTime};

/// Calculates new review data according to the SM-2 algorithm.
/// quality: 0-5 (0 = complete blackout, 5 = perfect response)
pub fn calculate_next_review(
    review_data: &ReviewData,
    quality: u8,
    current_date: SystemTime,
) -> ReviewData {
    let quality = quality.min(5); // Clamp to 0-5

    // Calculate new E-Factor (easiness factor)
    let q = quality as f64;
    let mut new_ef = review_data.easiness_factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02));

    // E-Factor should not fall below 1.3
    if new_ef < 1.3 {
        new_ef = 1.3;
    }

    let (new_interval, new_repetitions) = if quality < 3 {
        // If quality < 3, start from beginning (reset progress)
        (0, 0)
    } else {
        // Calculate new interval based on repetition number
        let new_reps = review_data.repetitions + 1;
        let new_int = match new_reps {
            1 => 1,                                                          // First repetition: 1 day
            2 => 6, // Second repetition: 6 days
            _ => (review_data.interval_days as f64 * new_ef).round() as i32, // Subsequent: multiply by EF
        };
        (new_int, new_reps)
    };

    // Calculate next review date
    let next_date = current_date + Duration::from_secs((new_interval as u64) * 24 * 60 * 60);

    ReviewData {
        flashcard_id: review_data.flashcard_id,
        easiness_factor: new_ef,
        interval_days: new_interval,
        repetitions: new_repetitions,
        next_review_date: next_date,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_review() {
        let review = ReviewData {
            flashcard_id: 1,
            easiness_factor: 2.5,
            interval_days: 0,
            repetitions: 0,
            next_review_date: SystemTime::now(),
        };

        let next = calculate_next_review(&review, 4, SystemTime::now());
        assert_eq!(next.interval_days, 1);
        assert_eq!(next.repetitions, 1);
    }

    #[test]
    fn test_second_review() {
        let review = ReviewData {
            flashcard_id: 1,
            easiness_factor: 2.5,
            interval_days: 1,
            repetitions: 1,
            next_review_date: SystemTime::now(),
        };

        let next = calculate_next_review(&review, 4, SystemTime::now());
        assert_eq!(next.interval_days, 6);
        assert_eq!(next.repetitions, 2);
    }

    #[test]
    fn test_quality_below_3_resets() {
        let review = ReviewData {
            flashcard_id: 1,
            easiness_factor: 2.5,
            interval_days: 10,
            repetitions: 5,
            next_review_date: SystemTime::now(),
        };

        let next = calculate_next_review(&review, 2, SystemTime::now());
        assert_eq!(next.interval_days, 0);
        assert_eq!(next.repetitions, 0);
        // EF should still be updated
        assert!(next.easiness_factor < 2.5);
    }

    #[test]
    fn test_ef_floor() {
        let review = ReviewData {
            flashcard_id: 1,
            easiness_factor: 1.3,
            interval_days: 1,
            repetitions: 1,
            next_review_date: SystemTime::now(),
        };

        let next = calculate_next_review(&review, 0, SystemTime::now());
        assert!(next.easiness_factor >= 1.3);
    }
}
