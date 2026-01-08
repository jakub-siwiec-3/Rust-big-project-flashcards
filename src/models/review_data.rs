use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct ReviewData {
    pub flashcard_id: i64,
    pub easiness_factor: f64,
    pub interval_days: i32,
    pub repetitions: i32,
    pub next_review_date: SystemTime,
}
