//! JSON import/export module for flashcard decks.
//! Provides functionality to save and load Deck structures to/from JSON files.

use crate::models::Deck;
use std::fs::File;
use std::io::{Read, Write};

/// Exports a deck to a JSON file at the specified path.
/// Returns an error if file creation or writing fails.
pub fn export_json_to_path(deck: &Deck, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(deck)?;
    let mut file = File::create(path)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}

/// Imports a deck from a JSON file.
/// Prints the deck name upon successful import.
/// Returns an error if the file doesn't exist or contains invalid JSON.
pub fn import_json(filename: &str) -> Result<Deck, Box<dyn std::error::Error>> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize JSON string into Deck structure
    let deck: Deck = serde_json::from_str(&contents)?;

    println!("Deck '{}' imported from '{}'", deck.name, filename);
    Ok(deck)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Deck, Flashcard};
    use std::fs;

    fn create_test_deck() -> Deck {
        Deck {
            name: "Test Deck".to_string(),
            flashcards: vec![
                Flashcard {
                    term: "hello".to_string(),
                    definition: "cześć".to_string(),
                },
                Flashcard {
                    term: "goodbye".to_string(),
                    definition: "do widzenia".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_export_json_to_path() {
        let deck = create_test_deck();
        let test_file = "test_export.json";

        let result = export_json_to_path(&deck, test_file);
        assert!(result.is_ok());

        assert!(fs::metadata(test_file).is_ok(), "File should exist");

        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_import_json() {
        let json_content = r#"{
  "name": "Import Test Deck",
  "flashcards": [
    {
      "term": "test term",
      "definition": "test definition"
    }
  ]
}"#;

        let test_file = "test_import.json";
        fs::write(test_file, json_content).unwrap();

        let result = import_json(test_file);
        assert!(result.is_ok());

        let deck = result.unwrap();
        assert_eq!(deck.name, "Import Test Deck");
        assert_eq!(deck.flashcards.len(), 1);
        assert_eq!(deck.flashcards[0].term, "test term");
        assert_eq!(deck.flashcards[0].definition, "test definition");

        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_export_and_import_roundtrip() {
        let original_deck = create_test_deck();
        let test_file = "test_roundtrip.json";

        let export_result = export_json_to_path(&original_deck, test_file);
        assert!(export_result.is_ok());

        let import_result = import_json(test_file);
        assert!(import_result.is_ok());

        let imported_deck = import_result.unwrap();

        assert_eq!(original_deck.name, imported_deck.name);
        assert_eq!(
            original_deck.flashcards.len(),
            imported_deck.flashcards.len()
        );

        for (orig, imp) in original_deck
            .flashcards
            .iter()
            .zip(imported_deck.flashcards.iter())
        {
            assert_eq!(orig.term, imp.term);
            assert_eq!(orig.definition, imp.definition);
        }

        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_import_nonexistent_file() {
        let result = import_json("nonexistent_file_xyz123.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_invalid_json() {
        let test_file = "test_invalid.json";
        fs::write(test_file, "{ this is not valid json }").unwrap();

        let result = import_json(test_file);
        assert!(result.is_err());

        let _ = fs::remove_file(test_file);
    }
}
