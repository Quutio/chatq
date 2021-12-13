use std::{str::FromStr, time::SystemTime};

use uuid::Uuid;

use crate::chatq;

#[derive(Debug, Clone, PartialEq)]
pub struct Server {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plugin {
    pub value: String,
}

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
    id: i64,
    timestamp: SystemTime,
    source: MessageSource,
    audience: MessageAudience,
    content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageStub {
    timestamp: SystemTime,
    source: MessageSource,
    audience: MessageAudience,
    content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageFilter {
    Before(SystemTime),
    After(SystemTime),
    Player(Uuid),
    Server(Server),
}

impl From<chatq::Server> for Server {
    fn from(f: chatq::Server) -> Self {
        Self { value: f.value }
    }
}

impl From<Server> for chatq::Server {
    fn from(f: Server) -> Self {
        Self { value: f.value }
    }
}

impl From<chatq::Plugin> for Plugin {
    fn from(f: chatq::Plugin) -> Self {
        Self { value: f.value }
    }
}

impl From<Plugin> for chatq::Plugin {
    fn from(f: Plugin) -> Self {
        Self { value: f.value }
    }
}

impl From<chatq::MessageAudience> for MessageAudience {
    fn from(f: chatq::MessageAudience) -> Self {
        match f.audience.unwrap() {
            chatq::message_audience::Audience::Players(players) => MessageAudience::Players(
                players
                    .uuids
                    .into_iter()
                    .map(|x| Uuid::from_str(&x.value).unwrap())
                    .collect::<Vec<Uuid>>(),
            ),
            chatq::message_audience::Audience::Server(servers) => MessageAudience::Servers(
                servers
                    .servers
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<Server>>(),
            ),
        }
    }
}

impl From<MessageAudience> for chatq::MessageAudience {
    fn from(f: MessageAudience) -> Self {
        match f {
            MessageAudience::Players(players) => Self {
                audience: Some(chatq::message_audience::Audience::Players(chatq::Uuids {
                    uuids: players
                        .into_iter()
                        .map(|x| chatq::Uuid {
                            value: x.to_string(),
                        })
                        .collect::<Vec<chatq::Uuid>>(),
                })),
            },
            MessageAudience::Servers(servers) => Self {
                audience: Some(chatq::message_audience::Audience::Server(chatq::Servers {
                    servers: servers
                        .into_iter()
                        .map(|x| x.into())
                        .collect::<Vec<chatq::Server>>(),
                })),
            },
        }
    }
}

impl From<chatq::MessageSource> for MessageSource {
    fn from(f: chatq::MessageSource) -> Self {
        match f.source.unwrap() {
            chatq::message_source::Source::Players(players) => MessageSource::Players(
                players
                    .uuids
                    .into_iter()
                    .map(|x| Uuid::from_str(&x.value).unwrap())
                    .collect::<Vec<Uuid>>(),
            ),
            chatq::message_source::Source::Plugins(plugins) => MessageSource::Plugins(
                plugins
                    .plugins
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<Plugin>>(),
            ),
        }
    }
}

impl From<MessageSource> for chatq::MessageSource {
    fn from(f: MessageSource) -> Self {
        match f {
            MessageSource::Players(players) => chatq::MessageSource {
                source: Some(chatq::message_source::Source::Players(chatq::Uuids {
                    uuids: players
                        .into_iter()
                        .map(|x| chatq::Uuid {
                            value: x.to_string(),
                        })
                        .collect::<Vec<chatq::Uuid>>(),
                })),
            },
            MessageSource::Plugins(plugins) => chatq::MessageSource {
                source: Some(chatq::message_source::Source::Plugins(chatq::Plugins {
                    plugins: plugins
                        .into_iter()
                        .map(|x| chatq::Plugin { value: x.value })
                        .collect::<Vec<chatq::Plugin>>(),
                })),
            },
        }
    }
}

impl From<chatq::Message> for Message {
    fn from(f: chatq::Message) -> Self {
        Self {
            id: f.id,
            timestamp: f.timestamp.unwrap().try_into().unwrap(),
            source: f.source.unwrap().into(),
            audience: f.audience.unwrap().into(),
            content: f.content,
        }
    }
}

impl From<Message> for chatq::Message {
    fn from(f: Message) -> Self {
        Self {
            id: f.id,
            timestamp: Some(f.timestamp.into()),
            source: Some(f.source.into()),
            audience: Some(f.audience.into()),
            content: f.content,
        }
    }
}

impl From<chatq::MessageStub> for MessageStub {
    fn from(f: chatq::MessageStub) -> Self {
        Self {
            timestamp: f.timestamp.unwrap().try_into().unwrap(),
            source: f.source.unwrap().into(),
            audience: f.audience.unwrap().into(),
            content: f.content,
        }
    }
}

impl From<MessageStub> for chatq::MessageStub {
    fn from(f: MessageStub) -> Self {
        Self {
            timestamp: Some(f.timestamp.into()),
            source: Some(f.source.into()),
            audience: Some(f.audience.into()),
            content: f.content,
        }
    }
}

impl From<chatq::MessageFilter> for MessageFilter {
    fn from(f: chatq::MessageFilter) -> Self {
        match f.condition.unwrap() {
            chatq::message_filter::Condition::Before(bacon) => {
                MessageFilter::Before(bacon.try_into().unwrap())
            }
            chatq::message_filter::Condition::After(aspargus) => {
                MessageFilter::After(aspargus.try_into().unwrap())
            }
            chatq::message_filter::Condition::Player(udon) => {
                MessageFilter::Player(Uuid::from_str(&udon.value).unwrap())
            }
            chatq::message_filter::Condition::Server(salami) => {
                MessageFilter::Server(salami.into())
            }
        }
    }
}

impl From<MessageFilter> for chatq::MessageFilter {
    fn from(f: MessageFilter) -> Self {
        match f {
            MessageFilter::Before(bacon) => Self {
                condition: Some(chatq::message_filter::Condition::Before(bacon.into())),
            },
            MessageFilter::After(aspargus) => Self {
                condition: Some(chatq::message_filter::Condition::After(aspargus.into())),
            },
            MessageFilter::Player(udon) => Self {
                condition: Some(chatq::message_filter::Condition::Player(chatq::Uuid {
                    value: udon.to_string(),
                })),
            },
            MessageFilter::Server(salami) => Self {
                condition: Some(chatq::message_filter::Condition::Server(salami.into())),
            },
        }
    }
}
