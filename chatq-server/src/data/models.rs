use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::chatq;

#[derive(Debug, Clone, PartialEq)]
pub struct Server(String);

#[derive(Debug, Clone, PartialEq)]
pub struct Plugin(String);

#[derive(Debug, Clone, PartialEq)]
pub enum MessageAudience {
    Players(Vec<Uuid>),
    Servers(Vec<Server>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageSource {
    Players(Vec<Uuid>),
    Plugins(Vec<Plugin>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    id: u64,
    timestamp: NaiveDateTime,
    source: MessageSource,
    audience: MessageAudience,
    content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageStub {
    timestamp: NaiveDateTime,
    source: MessageSource,
    audience: MessageAudience,
    content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageFilter {
    Before(NaiveDateTime),
    After(NaiveDateTime),
    Uuid(Uuid),
    Servers(Vec<Server>),
}
