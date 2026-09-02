use serde::{Deserialize, Serialize};
use crate::enums::{Language, Theme};

/// Represents Akinator's character or object prediction proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guess {
    /// Proposition ID in Akinator database.
    pub id: String,

    /// Name of the guessed entity.
    pub name: String,

    /// Short description or role of the entity.
    pub description: String,

    /// Direct URL to the character/item image if available.
    pub photo: Option<String>,

    /// The user who submitted or confirmed this entity.
    pub pseudo: Option<String>,

    /// Photo flag status.
    pub flag_photo: u32,

    /// Base proposition identifier if applicable.
    pub base_proposition_id: Option<String>,
}

impl Guess {
    /// Formats and returns the full absolute URL to the guessed character's photo, if available.
    #[must_use]
    pub fn photo_url(&self) -> Option<String> {
        self.photo.as_ref().map(|p| {
            if p.starts_with("http://") || p.starts_with("https://") {
                p.clone()
            } else if p.starts_with('/') {
                format!("https://en.akinator.com{p}")
            } else {
                format!("https://picture.akinator.com/photos/{p}")
            }
        })
    }
}

/// The result returned after initiating a game, submitting an answer, or backtracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StepResult {
    /// Next question to answer.
    Question {
        /// The question text.
        question: String,
        /// Current question index (starts at 0).
        step: usize,
        /// Akinator's certainty progression percentage (0.0 to 100.0).
        progression: f32,
        /// Current mood/akitude identifier or image filename.
        akitude: String,
    },
    /// Akinator has accumulated enough certainty to propose a guess.
    Guess(Guess),
    /// Akinator was unable to find a matching entity and gave up.
    GiveUp,
}

/// A fully serializable and persistent snapshot of an active Akinator session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionData {
    pub session: String,
    pub signature: String,
    pub url: String,
    pub identifiant: String,
    pub question: String,
    pub step: usize,
    pub progression: f32,
    pub step_last_proposition: String,
    pub akitude: String,
    pub won: bool,
    pub ko: bool,
    pub started: bool,
    pub language: Language,
    pub theme: Theme,
    pub child_mode: bool,
    pub current_guess: Option<Guess>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawStepResponse {
    pub completion: Option<String>,
    pub step: Option<serde_json::Value>,
    pub progression: Option<serde_json::Value>,
    pub question: Option<String>,
    pub akitude: Option<String>,
    pub step_last_proposition: Option<serde_json::Value>,
    pub id_proposition: Option<serde_json::Value>,
    pub id_base_proposition: Option<serde_json::Value>,
    pub name_proposition: Option<String>,
    pub description_proposition: Option<String>,
    pub photo: Option<String>,
    pub pseudo: Option<String>,
    pub flag_photo: Option<serde_json::Value>,
}
