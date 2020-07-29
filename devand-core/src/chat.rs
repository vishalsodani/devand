use crate::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i32, // FIXME UUID
    pub created_at: DateTime<Utc>,
    pub author: UserId,
    pub txt: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ChatId(pub i32);

impl std::fmt::Display for ChatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: ChatId,
    pub members: Vec<UserId>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Chats(pub Vec<Chat>);
