//! Database operations for flashcard application
//!
//! Handles SQLite database initialization, CRUD operations for decks and flashcards,
//! and SM-2 spaced repetition data management.

use crate::models::{Deck, DeckSet, Flashcard, ReviewData};
use rusqlite::{Connection, Result, params};
use std::time::{Duration, SystemTime};

/// Initializes SQLite database with required tables
///
/// Creates tables for decks, flashcards, SM-2 review data, and app state.
/// Sets current date to now if not already initialized.
pub fn init_database() -> Result<Connection> {
    let conn = Connection::open("db.sqlite3")?;

    // Create decks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS decks (
            name TEXT PRIMARY KEY
        )",
        (),
    )?;

    // Create flashcards table with auto-increment ID
    conn.execute(
        "CREATE TABLE IF NOT EXISTS flashcards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            deck_name TEXT NOT NULL,
            term TEXT NOT NULL,
            definition TEXT NOT NULL,
            FOREIGN KEY (deck_name) REFERENCES decks(name),
            UNIQUE(deck_name, term)
        )",
        (),
    )?;

    // Create review_data table for SM-2 algorithm
    conn.execute(
        "CREATE TABLE IF NOT EXISTS review_data (
            flashcard_id INTEGER PRIMARY KEY,
            easiness_factor REAL NOT NULL DEFAULT 2.5,
            interval_days INTEGER NOT NULL DEFAULT 0,
            repetitions INTEGER NOT NULL DEFAULT 0,
            next_review_date INTEGER NOT NULL,
            FOREIGN KEY (flashcard_id) REFERENCES flashcards(id) ON DELETE CASCADE
        )",
        (),
    )?;

    // Create app_state table for storing current date
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        (),
    )?;

    // Initialize current_date if not exists
    let current_timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    conn.execute(
        "INSERT OR IGNORE INTO app_state (key, value) VALUES ('current_date', ?1)",
        params![current_timestamp.to_string()],
    )?;

    Ok(conn)
}

/// Retrieves current simulated date from database
pub fn get_current_date(conn: &Connection) -> Result<SystemTime> {
    let timestamp: String = conn.query_row(
        "SELECT value FROM app_state WHERE key = 'current_date'",
        [],
        |row| row.get(0),
    )?;

    let secs = timestamp.parse::<u64>().unwrap_or(0);
    Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
}

/// Advances current date by 24 hours (for testing spaced repetition)
pub fn advance_day(conn: &Connection) -> Result<()> {
    let current = get_current_date(conn)?;
    let next_day = current + Duration::from_secs(24 * 60 * 60);
    let timestamp = next_day
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    conn.execute(
        "UPDATE app_state SET value = ?1 WHERE key = 'current_date'",
        params![timestamp.to_string()],
    )?;

    Ok(())
}

/// Creates a new deck in the database
pub fn new_deck(name: &str, conn: &Connection) -> Result<()> {
    conn.execute("INSERT INTO decks (name) VALUES (?1)", params![name])?;
    println!("Deck '{}' created successfully.", name);
    Ok(())
}

/// Adds a flashcard to a deck and initializes its SM-2 review data
///
/// Returns the flashcard ID. If flashcard already exists (same deck + term),
/// it's ignored due to UNIQUE constraint.
pub fn add_flashcard(
    deck_name: &str,
    term: &str,
    definition: &str,
    conn: &Connection,
) -> Result<i64> {
    // Insert flashcard (or ignore if duplicate)
    conn.execute(
        "INSERT OR IGNORE INTO flashcards (deck_name, term, definition) VALUES (?1, ?2, ?3)",
        params![deck_name, term, definition],
    )?;

    // Get flashcard ID
    let flashcard_id: i64 = conn.query_row(
        "SELECT id FROM flashcards WHERE deck_name = ?1 AND term = ?2",
        params![deck_name, term],
        |row| row.get(0),
    )?;

    // Initialize review_data with default SM-2 values
    let current_date = get_current_date(conn)?;
    let timestamp = current_date
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    conn.execute(
        "INSERT OR IGNORE INTO review_data (flashcard_id, easiness_factor, interval_days, repetitions, next_review_date)
         VALUES (?1, 2.5, 0, 0, ?2)",
        params![flashcard_id, timestamp as i64],
    )?;

    Ok(flashcard_id)
}

/// Retrieves all flashcards for a given deck
///
/// Returns vector of (flashcard_id, Flashcard) tuples
pub fn get_flashcards_for_deck(
    deck_name: &str,
    conn: &Connection,
) -> Result<Vec<(i64, Flashcard)>> {
    let mut stmt =
        conn.prepare("SELECT id, term, definition FROM flashcards WHERE deck_name = ?1")?;

    let flashcards = stmt
        .query_map(params![deck_name], |row| {
            Ok((
                row.get(0)?,
                Flashcard {
                    term: row.get(1)?,
                    definition: row.get(2)?,
                },
            ))
        })?
        .collect::<Result<Vec<(i64, Flashcard)>>>()?;

    Ok(flashcards)
}

/// Updates SM-2 review data for a flashcard after a learning session
pub fn update_review_data(review_data: &ReviewData, conn: &Connection) -> Result<()> {
    let timestamp = review_data
        .next_review_date
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "UPDATE review_data
         SET easiness_factor = ?1, interval_days = ?2, repetitions = ?3, next_review_date = ?4
         WHERE flashcard_id = ?5",
        params![
            review_data.easiness_factor,
            review_data.interval_days,
            review_data.repetitions,
            timestamp,
            review_data.flashcard_id
        ],
    )?;

    Ok(())
}

/// Retrieves flashcards due for review in a deck
///
/// Returns flashcards where next_review_date <= current_date,
/// ordered by next_review_date (oldest first).
pub fn get_flashcards_due_for_review(
    deck_name: &str,
    conn: &Connection,
) -> Result<Vec<(i64, Flashcard, ReviewData)>> {
    let current_date = get_current_date(conn)?;
    let current_timestamp = current_date
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut stmt = conn.prepare(
        "SELECT f.id, f.term, f.definition, r.easiness_factor, r.interval_days, r.repetitions, r.next_review_date
         FROM flashcards f
         JOIN review_data r ON f.id = r.flashcard_id
         WHERE f.deck_name = ?1 AND r.next_review_date <= ?2
         ORDER BY r.next_review_date ASC"
    )?;

    let flashcards = stmt
        .query_map(params![deck_name, current_timestamp], |row| {
            let id: i64 = row.get(0)?;
            Ok((
                id,
                Flashcard {
                    term: row.get(1)?,
                    definition: row.get(2)?,
                },
                ReviewData {
                    flashcard_id: id,
                    easiness_factor: row.get(3)?,
                    interval_days: row.get(4)?,
                    repetitions: row.get(5)?,
                    next_review_date: SystemTime::UNIX_EPOCH
                        + Duration::from_secs(row.get::<_, i64>(6)? as u64),
                },
            ))
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(flashcards)
}

/// Retrieves all deck names from database
pub fn get_all_decks(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM decks")?;
    let decks = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>>>()?;
    Ok(decks)
}

/// Loads all decks with their flashcards into memory
///
/// Does not load SM-2 review data - that's fetched separately when starting a learning session.
pub fn load_all_decks(conn: &Connection) -> Result<DeckSet> {
    let mut stmt = conn.prepare("SELECT name FROM decks")?;
    let deck_names = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<String>>>()?;

    let mut decks = Vec::new();

    for deck_name in deck_names {
        let flashcards_with_ids = get_flashcards_for_deck(&deck_name, conn)?;
        // Strip IDs - we only need them for review sessions
        let flashcards = flashcards_with_ids.into_iter().map(|(_, fc)| fc).collect();

        decks.push(Deck {
            name: deck_name,
            flashcards,
        });
    }

    Ok(DeckSet { decks })
}
