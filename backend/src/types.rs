use std::time::Duration;

use bytes::Bytes;
use mime::Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type alias for UUIDs used to represent image references
pub type ImageRef = Uuid;

/// Images stored within games
#[derive(Debug, Clone)]
pub struct Image {
    /// Mime type for the image
    pub mime: Mime,
    /// The image data bytes
    pub data: Bytes,
}

/// Type alias for a number representing an index of
/// a question
pub type QuestionIndex = usize;

/// Structure of a quiz question
#[derive(Serialize, Deserialize)]
pub struct Question {
    /// The type of question and the question data
    /// flattened to the question level
    #[serde(flatten)]
    pub data: QuestionData,
    /// The title of the question
    pub title: String,
    /// The text of the question
    pub text: String,
    /// Optional UUID from created image
    pub image: Option<ImageRef>,
    /// The time given to answer the question
    pub answer_time: u64,
    /// The point scoring for the question
    pub scoring: Scoring,
}

/// The different types of questions and their
/// associated question data
#[derive(Serialize, Deserialize)]
#[serde(tag = "ty")]
pub enum QuestionData {
    /// Single choice question
    Single {
        /// Vec of indexes of correct answers
        #[serde(skip)]
        answers: Vec<QuestionIndex>,
        /// Vec of the possible answers
        values: Vec<String>,
    },
    /// Multiple choice question
    Multiple {
        /// Vec of indexes of correct answers
        #[serde(skip)]
        answers: Vec<QuestionIndex>,
        /// Vec of the possible answers
        values: Vec<String>,
        /// The optional minimum number of required answers
        min: Option<usize>,
        /// The optional maximum number of required answers
        max: Option<usize>,
    },
    /// Image where you must click an area
    ClickableImage {
        /// The image ref to take clicking on
        image: ImageRef,
        /// Top left box coordinate
        #[serde(skip)]
        top: (f32, f32),
        /// Bottom right box coordinate
        #[serde(skip)]
        bottom: (f32, f32),
    },
}

/// Game settings for global min/max scoring along with
/// bonus scoring amount
#[derive(Serialize, Deserialize)]
pub struct Scoring {
    /// Minimum score awarded for the longest time taken
    pub min_score: u32,
    /// Maximum score awarded for the shortest time taken
    pub max_score: u32,
    /// The amount awarded if scored within the bonus time
    pub bonus_score: u32,
}

/// Stored state for answer data including
/// the elapsed time when the answer was made
/// to calculate time scores
#[derive(Clone)]
pub struct AnswerData {
    /// The current elapsed timer duration at the
    /// time of answering
    pub elapsed: Duration,
    /// The answer to the question
    pub answer: Answer,
}

/// The different types of answers for the different
/// question types along with the associated answer values
#[derive(Deserialize, Clone)]
#[serde(tag = "ty")]
pub enum Answer {
    /// Answer for a single choice question
    Single {
        /// The index of the chosen answer
        answer: QuestionIndex,
    },
    /// Answers for a multiple choice question
    Multiple {
        /// The list of chosen answers
        answers: Vec<QuestionIndex>,
    },
    /// Answer for a clickable region image
    ClickableImage {
        /// The X and Y position that was clicked
        answer: (f32, f32),
    },
}

impl Answer {
    /// Validation to ensure that a question answer is the
    /// right type of answer for the specified quesiton type
    pub fn is_valid(&self, qt: &QuestionData) -> bool {
        match (self, qt) {
            (Self::Single { .. }, QuestionData::Single { .. })
            | (Self::Multiple { .. }, QuestionData::Multiple { .. })
            | (Self::ClickableImage { .. }, QuestionData::ClickableImage { .. }) => true,
            _ => false,
        }
    }
}

/// Represents the different scores that can be
/// gained from an answer
#[derive(Serialize, Clone, Copy)]
#[serde(tag = "ty", content = "value")]
pub enum Score {
    // Answer was 100% correct
    Correct(u32),
    // Answer was incorrect
    Incorrect,
    // Multiple choice has some asnwers right
    Partial(u32),
}

impl Score {
    /// Obtains the score value from the answer score
    pub fn value(&self) -> u32 {
        match self {
            Self::Correct(value) | Self::Partial(value) => *value,
            Self::Incorrect => 0,
        }
    }
}
