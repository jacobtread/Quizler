use serde::{ser::SerializeStruct, Serialize};

#[derive(Clone)]
#[repr(u8)]
pub enum ServerError {
    /// The last proivded message was malformed
    MalformedMessage,
    /// The provided token didn't match up to any game
    InvalidToken,
    /// The provided username is already in use
    UsernameTaken,
    /// The game is already started or finish so cannot be joined
    NotJoinable,
    /// An action was attempting on a player that wasnt found
    UnknownPlayer,
}

impl ServerError {
    pub fn code(&self) -> u8 {
        match self {
            Self::MalformedMessage => 0x0,
            Self::InvalidToken => 0x1,
            Self::UsernameTaken => 0x2,
            Self::NotJoinable => 0x3,
            Self::UnknownPlayer => 0x4,
        }
    }
}

impl Serialize for ServerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut stru = serializer.serialize_struct("ServerError", 2)?;
        // Message type field to match up with server messages
        stru.serialize_field("ty", "Error")?;
        // The error code field
        stru.serialize_field("error", &self.code())?;

        stru.end()
    }
}
