use actix::Message;
use serde::Serialize;

#[derive(Message, Debug, Copy, Clone, Serialize)]
#[rtype(result = "()")]
#[repr(u8)]
pub enum ServerError {
    /// The last proivded message was malformed
    MalformedMessage = 0x0,
    /// The provided token didn't match up to any game
    InvalidToken = 0x1,
    /// The provided username is already in use
    UsernameTaken = 0x2,
    /// The game is already started or finish so cannot be joined
    NotJoinable = 0x3,
    /// An action was attempting on a player that wasnt found
    UnknownPlayer = 0x4,
    /// Something unexpected went wrong on the server
    Unexpected = 0x5,
    /// Didn't have permission to complete that action
    InvalidPermission = 0x6,
    /// Message was recieved but wasn't expected at the current
    /// state
    UnexpectedMessage = 0x7,
}
