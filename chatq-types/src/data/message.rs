use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
pub enum Tree {
    Empty,
    Node(Box<Node>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Node {
    pub label: String,
    pub tail: Box<Tree>,
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
    pub context: Tree,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Message {
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub source: MessageSource,
    pub audience: MessageAudience,
    pub content: String,
    pub context: Tree,
}

#[cfg(feature = "proto")]
pub mod from_proto {
    use crate::chatq;
    use crate::chatq::tree::Kind;
    use crate::data::error::ModelConversionError;
    use crate::data::from_proto::{naive_from_proto, proto_from_naive};
    use crate::data::message::{Message, MessageAudience, MessageSource, MessageStub, Node, Tree};
    use std::str::FromStr;
    use uuid::Uuid;

    impl TryFrom<chatq::MessageSource> for MessageSource {
        type Error = ModelConversionError;

        fn try_from(f: chatq::MessageSource) -> Result<Self, Self::Error> {
            match f
                .source
                .ok_or(ModelConversionError::ValueNotProvided("source".into()))?
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
                .ok_or(ModelConversionError::ValueNotProvided("audience".into()))?
            {
                chatq::message_audience::Audience::Players(players) => Ok(MessageAudience::new(
                    players
                        .uuids
                        .into_iter()
                        .map(|x| Uuid::from_str(&x.value))
                        .collect::<Result<_, _>>()
                        .map_err(ModelConversionError::UuidConversion)?,
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
                        .iter()
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

    impl From<Tree> for chatq::Tree {
        fn from(value: Tree) -> Self {
            match value {
                Tree::Empty => chatq::Tree {
                    kind: Some(Kind::Empty(Default::default())),
                },
                Tree::Node(node) => chatq::Tree {
                    kind: Some(Kind::Node(Box::new(chatq::Node {
                        label: node.label,
                        tail: Some(Box::new((*node.tail).into())),
                    }))),
                },
            }
        }
    }

    impl TryFrom<chatq::Tree> for Tree {
        type Error = ModelConversionError;

        fn try_from(value: chatq::Tree) -> Result<Self, Self::Error> {
            let kind = value
                .kind
                .ok_or(ModelConversionError::ValueNotProvided("kind".into()))?;

            match kind {
                Kind::Empty(_) => Ok(Tree::Empty),
                Kind::Node(node) => {
                    let tail: Tree = (*node
                        .tail
                        .ok_or(ModelConversionError::ValueNotProvided("tail".into()))?)
                    .try_into()?;
                    Ok(Tree::Node(Box::new(Node {
                        label: node.label,
                        tail: Box::new(tail),
                    })))
                }
            }
        }
    }

    impl From<MessageStub> for chatq::MessageStub {
        fn from(f: MessageStub) -> Self {
            Self {
                timestamp: Some(proto_from_naive(f.timestamp)),
                source: Some(f.source.into()),
                audience: Some(f.audience.into()),
                content: f.content,
                context: Some(f.context.into()),
            }
        }
    }

    impl TryFrom<chatq::MessageStub> for MessageStub {
        type Error = ModelConversionError;

        fn try_from(f: chatq::MessageStub) -> Result<Self, Self::Error> {
            Ok(Self {
                timestamp: naive_from_proto(
                    f.timestamp
                        .ok_or(ModelConversionError::ValueNotProvided("timestamp".into()))?,
                )?,
                source: f
                    .source
                    .ok_or(ModelConversionError::ValueNotProvided("source".into()))?
                    .try_into()?,
                audience: f
                    .audience
                    .ok_or(ModelConversionError::ValueNotProvided("audience".into()))?
                    .try_into()?,
                content: f.content,
                context: f.context
                    .ok_or(ModelConversionError::ValueNotProvided("context".into()))?
                    .try_into()?,
            })
        }
    }

    impl TryFrom<chatq::Message> for Message {
        type Error = ModelConversionError;

        fn try_from(f: chatq::Message) -> Result<Self, Self::Error> {
            Ok(Self {
                id: f.id,
                timestamp: naive_from_proto(
                    f.timestamp
                        .ok_or(ModelConversionError::ValueNotProvided("timestamp".into()))?,
                )?,
                source: f
                    .source
                    .ok_or(ModelConversionError::ValueNotProvided("source".into()))?
                    .try_into()?,
                audience: f
                    .audience
                    .ok_or(ModelConversionError::ValueNotProvided("audience".into()))?
                    .try_into()?,
                content: f.content,
                context: f
                    .context
                    .ok_or(ModelConversionError::ValueNotProvided("context".into()))?
                    .try_into()?,
            })
        }
    }

    impl From<Message> for chatq::Message {
        fn from(f: Message) -> Self {
            Self {
                id: f.id,
                timestamp: Some(proto_from_naive(f.timestamp)),
                source: Some(f.source.into()),
                audience: Some(f.audience.into()),
                content: f.content,
                context: Some(f.context.into()),
            }
        }
    }
}
