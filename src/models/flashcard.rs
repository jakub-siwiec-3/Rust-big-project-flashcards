//! Flashcard is a pair <term, definition>. Only text is used in terms and definitions
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub term: String,
    pub definition: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flashcard_creation() {
        let card = Flashcard {
            term: "hello".to_string(),
            definition: "cześć".to_string(),
        };

        assert_eq!(card.term, "hello");
        assert_eq!(card.definition, "cześć");
    }

    #[test]
    fn test_flashcard_clone() {
        let card1 = Flashcard {
            term: "hello".to_string(),
            definition: "cześć".to_string(),
        };

        let card2 = card1.clone();
        assert_eq!(card1.term, card2.term);
        assert_eq!(card1.definition, card2.definition);
    }
}
