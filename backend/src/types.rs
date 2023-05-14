use actix::Message;
use bytes::Bytes;
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Message, Debug, Copy, Clone, Serialize)]
#[rtype(result = "()")]
pub enum ServerError {
    /// The last proivded message was malformed
    MalformedMessage,
    /// The provided token didn't match up to any game
    InvalidToken,
    /// The provided username is already in use
    UsernameTaken,
    /// Provided name was not allowed/inappropriate
    InappropriateName,
    /// The game is already started or finish so cannot be joined
    NotJoinable,
    /// The game is already at max capacity
    CapacityReached,
    /// An action was attempting on a player that wasnt found
    UnknownPlayer,
    /// Something unexpected went wrong on the server
    Unexpected,
    /// Didn't have permission to complete that action
    InvalidPermission,
    /// Message was recieved but wasn't expected at the current
    /// state
    UnexpectedMessage,
    /// Provided answer is not valid for the type of question
    InvalidAnswer,
}

/// Type for the different levels of profanity filtering
/// on player names
#[derive(Debug, Deserialize)]
pub enum NameFiltering {
    /// Don't filter names anything goes
    None,
    /// Only stop the more severe names
    Low,
    /// Stop anything thats above mild
    Medium,
    /// Filter out any names that might be inappropriate
    High,
}

impl NameFiltering {
    /// Returns the rustrict type of filtering for
    /// this level or None if filtering is disabled
    pub fn type_of(&self) -> Option<rustrict::Type> {
        Some(match self {
            NameFiltering::Low => rustrict::Type::SEVERE,
            NameFiltering::Medium => rustrict::Type::MODERATE_OR_HIGHER,
            NameFiltering::High => rustrict::Type::INAPPROPRIATE,
            NameFiltering::None => return None,
        })
    }
}

/// Actions that can be executed by the host
/// session of a game
#[derive(Debug, Copy, Clone, Deserialize)]
pub enum HostAction {
    /// Begin the starting process
    Start,
    /// Cancel the starting process
    Cancel,
    /// Skip the current waiting timer state
    Skip,
    /// Reset the game and all its state back to lobby
    Reset,
}

/// Reasons why a player was removed from the game
#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
pub enum RemoveReason {
    /// Player was manually kicked by the host
    RemovedByHost,
    /// The host diconnected ending the game
    HostDisconnect,
    /// Connection was lost to the player
    LostConnection,
    /// Player disconnected
    Disconnected,
}

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

/// Structure of a quiz question
#[derive(Serialize, Deserialize)]
pub struct Question {
    /// The type of question and the question data
    /// flattened to the question level
    #[serde(flatten)]
    pub data: QuestionData,
    /// The text of the question
    pub text: String,
    /// Optional UUID from created image
    pub image: Option<ImageRef>,
    /// The time given to answer the question
    pub answer_time: u64,
    /// The time that a bonus score will be granted within
    /// bonus score is disabled if none (ms)
    pub bonus_score_time: u32,
    /// The point scoring for the question
    pub scoring: Scoring,
}

#[derive(Deserialize, Serialize)]
pub struct AnswerValue {
    /// The actual message for the answer
    pub value: String,
    /// Whether the answer is a correct one
    /// (Not sent to clients)
    #[serde(skip_serializing)]
    pub correct: bool,
}

/// Alias representing an index within an answers list
pub type AnswerIndex = usize;

/// The different types of questions and their
/// associated question data
#[derive(Serialize, Deserialize)]
#[serde(tag = "ty")]
pub enum QuestionData {
    /// Single choice question
    Single {
        /// Vec of indexes of correct answers
        answers: Vec<AnswerValue>,
    },
    /// Multiple choice question
    Multiple {
        /// Vec of indexes of correct answers
        answers: Vec<AnswerValue>,
        /// The minimum number of required answers
        min: usize,
        /// The maximum number of required answers
        max: usize,
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
        answer: AnswerIndex,
    },
    /// Answers for a multiple choice question
    Multiple {
        /// The list of chosen answers
        answers: Vec<AnswerIndex>,
    },
}

impl Answer {
    /// Validation to ensure that a question answer is the
    /// right type of answer for the specified quesiton type
    pub fn is_valid(&self, qt: &QuestionData) -> bool {
        matches!(
            (self, qt),
            // Both type Single
            (Self::Single { .. }, QuestionData::Single { .. })
            // Both type Multiple
                | (Self::Multiple { .. }, QuestionData::Multiple { .. })
        )
    }
}

/// Represents the different scores that can be
/// gained from an answer
#[derive(Serialize, Clone, Copy)]
#[serde(tag = "ty")]
pub enum Score {
    // Answer was 100% correct
    Correct { value: u32 },
    // Answer was incorrect
    Incorrect,
    // Multiple choice has some asnwers right
    Partial { value: u32, count: u32, total: u32 },
}

impl Score {
    /// Obtains the score value from the answer score
    pub fn value(&self) -> u32 {
        match self {
            Self::Correct { value } => *value,
            Self::Partial { value, .. } => *value,
            Self::Incorrect => 0,
        }
    }
}
