use bytes::Bytes;
use rand_core::{OsRng, RngCore};
use serde::{ser::SerializeMap, Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, hash::Hash, str::FromStr, time::Duration};
use uuid::Uuid;

use crate::{game::GameRef, session::SessionId};

/// Immutable string type
pub type ImStr = Box<str>;

#[derive(Debug, Copy, Clone, Serialize)]
pub enum ServerError {
    /// Provided message was malformed
    MalformedMessage,
    /// The provided token didn't match up to any game
    InvalidToken,
    /// The provided username doesn't meet the requirements
    InvalidNameLength,
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
    /// Progress to the next state
    Next,
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
    pub mime: Box<str>,
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
    pub text: ImStr,
    /// Optional image
    pub image: Option<QuestionImage>,
    /// The time given to answer the question
    pub answer_time: u64,
    /// The time that a bonus score will be granted within
    /// bonus score is disabled if none (ms)
    pub bonus_score_time: u32,
    /// The point scoring for the question
    pub scoring: Scoring,
}

impl Question {
    const MAX_QUESTION_LENGTH: usize = 400;

    /// Validates that the game configuration is valid
    /// and can be used for a game
    pub fn validate(&self) -> bool {
        let text_length = self.text.len();

        // Question text cannot be empty
        if text_length == 0 || text_length > Self::MAX_QUESTION_LENGTH {
            return false;
        }

        self.data.validate()
    }
}

/// Structure of a question image, contains the
/// UUID of the image aswell as its fit mode
#[derive(Serialize, Deserialize)]
pub struct QuestionImage {
    /// UUID from created image  
    pub uuid: ImageRef,
    /// Client side image fit mode
    pub fit: ImageFit,
}

#[derive(Serialize, Deserialize)]
pub enum ImageFit {
    Contain,
    Cover,
    Width,
    Height,
}

#[derive(Deserialize, Serialize)]
pub struct AnswerValue {
    /// The actual message for the answer
    pub value: ImStr,
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
        answers: Box<[AnswerValue]>,
    },
    /// Multiple choice question
    Multiple {
        /// Vec of indexes of correct answers
        answers: Box<[AnswerValue]>,
        /// The number of correct answers
        correct_answers: usize,
    },
    /// True / False questions
    TrueFalse {
        /// Answer value
        /// (Not sent to clients)
        #[serde(skip_serializing)]
        answer: bool,
    },
    /// Typing question
    Typer {
        /// Collection of valid answers
        /// (Not sent to clients)
        #[serde(skip_serializing)]
        answers: Box<[ImStr]>,
        /// Whether to ignore case when marking
        #[serde(skip_serializing)]
        ignore_case: bool,
    },
}

impl QuestionData {
    const MAX_ANSWERS: usize = 8;
    const MAX_ANSWER_LENGTH: usize = 150;

    /// Validates that the game configuration is valid
    /// and can be used for a game
    pub fn validate(&self) -> bool {
        match self {
            QuestionData::Single { answers } | QuestionData::Multiple { answers, .. } => {
                let answers_length = answers.len();
                if answers_length == 0 || answers_length > Self::MAX_ANSWERS {
                    return false;
                }

                answers.iter().all(|answer| {
                    let length = answer.value.len();
                    length > 0 && length < Self::MAX_ANSWER_LENGTH
                })
            }

            QuestionData::TrueFalse { .. } => true,
            QuestionData::Typer { answers, .. } => {
                let answers_length = answers.len();
                if answers_length == 0 || answers_length > Self::MAX_ANSWERS {
                    return false;
                }

                answers.iter().all(|answer| {
                    let length = answer.len();
                    length > 0 && length < Self::MAX_ANSWER_LENGTH
                })
            }
        }
    }
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
pub struct AnswerData {
    /// The current elapsed timer duration at the
    /// time of answering
    pub elapsed: Duration,
    /// The answer to the question
    pub answer: Answer,
}

/// The different types of answers for the different
/// question types along with the associated answer values
#[derive(Deserialize)]
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
        answers: Box<[AnswerIndex]>,
    },
    /// Answer for true false questions
    TrueFalse {
        /// The boolean answer
        answer: bool,
    },
    /// Answer for typing questions
    Typer {
        /// The string answer
        answer: ImStr,
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
                | (Self::TrueFalse { .. }, QuestionData::TrueFalse { .. })
                | (Self::Typer { .. }, QuestionData::Typer { .. })
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

/// More efficient collection for storing the scores of
/// each player that will be sent to the client
pub struct ScoreCollection(pub Vec<(SessionId, u32)>);

impl Serialize for ScoreCollection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;

        for (key, value) in &self.0 {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

/// Token abstraction to store tokens as fixed length byte
/// slices rather than strings. This makes them easier to
/// compare,generate, and serialize
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct GameToken([u8; GameToken::LENGTH]);

impl Hash for GameToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl GameToken {
    /// Length of tokens that will be created
    const LENGTH: usize = 5;
    /// Set of chars that can be used as game tokens
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    /// Creates a unique random token that isn't present in the
    /// provided collect of games
    pub fn unique_token(map: &HashMap<GameToken, GameRef>) -> GameToken {
        /// Length of the charset
        const RANGE: usize = GameToken::CHARSET.len();

        let mut rand = OsRng;
        let mut token = Self([0u8; Self::LENGTH]);

        loop {
            for at in token.0.iter_mut() {
                loop {
                    // Obtain a random number
                    let var = (rand.next_u32() >> (32 - 6)) as usize;

                    // If the value is in the charset break the loop
                    if var < RANGE {
                        *at = Self::CHARSET[var];
                        break;
                    }
                }
            }

            // Check that the token isn't already taken
            if !map.contains_key(&token) {
                return token;
            }
        }
    }
}

/// Game tokens can be created from strings as long as they only
/// contain characters from the token charset
impl FromStr for GameToken {
    type Err = ServerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != GameToken::LENGTH {
            return Err(ServerError::InvalidToken);
        }

        let bytes = s.as_bytes();

        // Handle invalid characters
        if bytes
            .iter()
            .any(|value| !GameToken::CHARSET.contains(value))
        {
            return Err(ServerError::InvalidToken);
        }

        let mut output = [0u8; GameToken::LENGTH];
        output.copy_from_slice(bytes);
        Ok(Self(output))
    }
}

impl AsRef<str> for GameToken {
    fn as_ref(&self) -> &str {
        // Safety: Game tokens are always valid utf8 so can be freely represented as &str
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl Display for GameToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token: &str = self.as_ref();
        f.write_str(token)
    }
}

impl Serialize for GameToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Game tokens are simply serialized as strings by casting the type
        let token: &str = self.as_ref();
        serializer.serialize_str(token)
    }
}
