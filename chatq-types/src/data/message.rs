use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::chatq;
use crate::data::error::ModelConversionError;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MessageAudience {
    players: Vec<Uuid>,
}

impl MessageAudience {
    pub fn new(players: Vec<Uuid>) -> Self {
        Self { players }
    }
    pub fn players(&self) -> &Vec<Uuid> {
        &self.players
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MessageSource {
    player: Uuid,
}

impl MessageSource {
    pub fn new(player: Uuid) -> Self {
        Self { player }
    }
    pub fn player(&self) -> &Uuid {
        &self.player
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MessageStub {
    pub timestamp: NaiveDateTime,
    pub source: MessageSource,
    pub audience: MessageAudience,
    pub content: String,
    pub context: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Message {
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub source: MessageSource,
    pub audience: MessageAudience,
    pub content: String,
    pub context: String,
}

impl TryFrom<chatq::MessageSource> for MessageSource {
    type Error = ModelConversionError;

    fn try_from(f: chatq::MessageSource) -> Result<Self, Self::Error> {
        match f
            .source
            .ok_or_else(|| ModelConversionError::ValueNotProvided("source"))?
        {
            chatq::message_source::Source::Player(player) => {
                Ok(MessageSource::new(Uuid::parse_str(&player.value)?))
            }
        }
    }
}

impl From<MessageSource> for chatq::MessageSource {
    fn from(f: MessageSource) -> Self {
        chatq::MessageSource {
            source: Some(chatq::message_source::Source::Player(chatq::Uuid {
                value: f.player().to_string(),
            })),
        }
    }
}

impl TryFrom<chatq::MessageAudience> for MessageAudience {
    type Error = ModelConversionError;

    fn try_from(f: chatq::MessageAudience) -> Result<Self, Self::Error> {
        match f
            .audience
            .ok_or_else(|| ModelConversionError::ValueNotProvided("audience"))?
        {
            chatq::message_audience::Audience::Players(players) => Ok(MessageAudience::new(
                players
                    .uuids
                    .into_iter()
                    .map(|x| Uuid::from_str(&x.value))
                    .collect::<Result<_, _>>()
                    .map_err(|err| ModelConversionError::UuidConversion(err))?,
            )),
        }
    }
}

impl From<MessageAudience> for chatq::MessageAudience {
    fn from(f: MessageAudience) -> Self {
        Self {
            audience: Some(chatq::message_audience::Audience::Players(chatq::Uuids {
                uuids: f
                    .players()
                    .into_iter()
                    .map(|x| chatq::Uuid {
                        value: x.to_string(),
                    })
                    .collect::<Vec<chatq::Uuid>>(),
            })),
        }
    }
}

impl Message {
    pub fn from_stub(id: i64, stub: MessageStub) -> Self {
        Message {
            id,
            timestamp: stub.timestamp,
            source: stub.source,
            audience: stub.audience,
            content: stub.content,
            context: stub.context,
        }
    }
}

impl From<MessageStub> for chatq::MessageStub {
    fn from(f: MessageStub) -> Self {
        Self {
            timestamp: Some(prost_types::Timestamp {
                seconds: f.timestamp.timestamp(),
                nanos: 0,
            }),
            source: Some(f.source.into()),
            audience: Some(f.audience.into()),
            content: f.content,
            context: f.context
        }
    }
}

impl TryFrom<chatq::MessageStub> for MessageStub {
    type Error = ModelConversionError;

    fn try_from(f: chatq::MessageStub) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: NaiveDateTime::from_timestamp(
                f.timestamp
                    .ok_or_else(|| ModelConversionError::ValueNotProvided("timestamp"))?
                    .seconds,
                0,
            ),
            source: f
                .source
                .ok_or_else(|| ModelConversionError::ValueNotProvided("source"))?
                .try_into()?,
            audience: f
                .audience
                .ok_or_else(|| ModelConversionError::ValueNotProvided("audience"))?
                .try_into()?,
            content: f.content,
            context: f.context
        })
    }
}

impl TryFrom<chatq::Message> for Message {
    type Error = ModelConversionError;

    fn try_from(f: chatq::Message) -> Result<Self, Self::Error> {
        Ok(Self {
            id: f.id,
            timestamp: NaiveDateTime::from_timestamp(
                f.timestamp
                    .ok_or_else(|| ModelConversionError::ValueNotProvided("timestamp"))?
                    .seconds,
                0,
            ),
            source: f
                .source
                .ok_or_else(|| ModelConversionError::ValueNotProvided("source"))?
                .try_into()?,
            audience: f
                .audience
                .ok_or_else(|| ModelConversionError::ValueNotProvided("audience"))?
                .try_into()?,
            content: f.content,
            context: f.context
        })
    }
}

impl From<Message> for chatq::Message {
    fn from(f: Message) -> Self {
        Self {
            id: f.id,
            timestamp: Some(prost_types::Timestamp {
                seconds: f.timestamp.timestamp(),
                nanos: 0,
            }),
            source: Some(f.source.into()),
            audience: Some(f.audience.into()),
            content: f.content,
            context: f.context,
        }
    }
}