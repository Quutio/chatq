use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::NaiveDateTime;
use thiserror::Error;
use uuid::Uuid;

use crate::chatq;
use crate::chatq::message_filter_pattern;
use crate::chatq::message_filter_pattern::composite_filter::filter_item::Type;
use crate::chatq::message_filter_pattern::message_filter::timestamp_filter::Condition;
use crate::chatq::message_filter_pattern::{message_filter, Operation, PrimaryCondition};

#[derive(Debug, Clone, PartialEq)]
pub struct Server {
    pub value: String,
}

impl Server {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plugin {
    pub value: String,
}

impl Plugin {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
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
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub source: MessageSource,
    pub audience: MessageAudience,
    pub content: String,
}

impl Message {
    pub fn from_stub(id: i64, stub: MessageStub) -> Self {
        Message {
            id,
            timestamp: stub.timestamp,
            source: stub.source,
            audience: stub.audience,
            content: stub.content,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageStub {
    pub timestamp: NaiveDateTime,
    pub source: MessageSource,
    pub audience: MessageAudience,
    pub content: String,
}

pub trait EvaluableFilter {
    fn evaluate(&self, message: &Message) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimestampFilter {
    Equals(NaiveDateTime),
    GreaterThan(NaiveDateTime),
    LessThan(NaiveDateTime),
    GreaterThanEqual(NaiveDateTime),
    LessThanEqual(NaiveDateTime),
}

impl Display for TimestampFilter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TimestampFilter::Equals(ts) => {
                write!(f, "= '{}'", ts)
            }
            TimestampFilter::GreaterThan(ts) => {
                write!(f, "> '{}'", ts)
            }
            TimestampFilter::LessThan(ts) => {
                write!(f, "< '{}'", ts)
            }
            TimestampFilter::GreaterThanEqual(ts) => {
                write!(f, ">= '{}'", ts)
            }
            TimestampFilter::LessThanEqual(ts) => {
                write!(f, "<= '{}'", ts)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudienceFilter {
    Uuid(Uuid),
    Server(Server),
}

impl Display for AudienceFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AudienceFilter::Uuid(uuid) => {
                write!(f, "audiences_player.player = \'{}\'", uuid)
            }
            AudienceFilter::Server(server) => {
                write!(f, "audiences_server.server = \'{}\'", server.value)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceFilter {
    Uuid(Uuid),
    Plugin(Plugin),
}

impl Display for SourceFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SourceFilter::Uuid(uuid) => {
                write!(f, "sources_player.player = \'{}\'", uuid)
            }
            SourceFilter::Plugin(plugin) => {
                write!(f, "sources_plugin.plugin = \'{}\'", plugin.value)
            }
        }
    }
}

impl EvaluableFilter for AudienceFilter {
    fn evaluate(&self, message: &Message) -> bool {
        match self {
            AudienceFilter::Uuid(uuid) => match &message.audience {
                MessageAudience::Players(players) => players.contains(uuid),
                MessageAudience::Servers(_) => false,
            },
            AudienceFilter::Server(server) => match &message.audience {
                MessageAudience::Players(_) => false,
                MessageAudience::Servers(servers) => servers.contains(server),
            },
        }
    }
}

impl EvaluableFilter for SourceFilter {
    fn evaluate(&self, message: &Message) -> bool {
        match self {
            SourceFilter::Uuid(uuid) => match &message.source {
                MessageSource::Players(players) => players.contains(uuid),
                MessageSource::Plugins(_) => false,
            },
            SourceFilter::Plugin(plugin) => match &message.source {
                MessageSource::Players(_) => false,
                MessageSource::Plugins(plugins) => plugins.contains(plugin),
            },
        }
    }
}

impl EvaluableFilter for TimestampFilter {
    fn evaluate(&self, message: &Message) -> bool {
        match self {
            TimestampFilter::Equals(ts) => &message.timestamp == ts,
            TimestampFilter::GreaterThan(ts) => &message.timestamp > ts,
            TimestampFilter::LessThan(ts) => &message.timestamp < ts,
            TimestampFilter::GreaterThanEqual(ts) => &message.timestamp >= ts,
            TimestampFilter::LessThanEqual(ts) => &message.timestamp <= ts,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageFilter {
    InsertTimestamp(TimestampFilter),
    Audience(AudienceFilter),
    Source(SourceFilter),
}

impl Display for MessageFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MessageFilter::InsertTimestamp(ts) => {
                write!(f, "{}", ts)
            }
            MessageFilter::Audience(audience) => {
                write!(f, "{}", audience)
            }
            MessageFilter::Source(source) => {
                write!(f, "{}", source)
            }
        }
    }
}

impl EvaluableFilter for MessageFilter {
    fn evaluate(&self, message: &Message) -> bool {
        match self {
            MessageFilter::InsertTimestamp(filter) => filter.evaluate(message),
            MessageFilter::Audience(filter) => filter.evaluate(message),
            MessageFilter::Source(filter) => filter.evaluate(message),
        }
    }
}

impl EvaluableFilter for FilterItem {
    fn evaluate(&self, message: &Message) -> bool {
        match &self {
            FilterItem::Single(single) => single.evaluate(message),
            FilterItem::Composite(composite) => composite.evaluate(message),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositeFilter {
    Or(Vec<FilterItem>),
    And(Vec<FilterItem>),
    Not(Box<FilterItem>),
}

impl Display for CompositeFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompositeFilter::Or(items) => {
                let joined = items
                    .iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>()
                    .join(" OR ");
                write!(f, "{}", joined)
            }
            CompositeFilter::And(items) => {
                let joined = items
                    .iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>()
                    .join(" AND ");
                write!(f, "{}", joined)
            }
            CompositeFilter::Not(item) => write!(f, "NOT {}", item),
        }
    }
}

impl EvaluableFilter for CompositeFilter {
    fn evaluate(&self, message: &Message) -> bool {
        return match self {
            CompositeFilter::Or(filters) => {
                let n_matched = filters.iter().filter(|op| op.evaluate(message)).count();
                n_matched > 0
            }
            CompositeFilter::And(filters) => {
                let n_matched = filters.iter().filter(|op| op.evaluate(message)).count();
                n_matched == filters.len()
            }
            CompositeFilter::Not(filter) => !filter.evaluate(message),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterItem {
    Single(MessageFilter),
    Composite(CompositeFilter),
}

impl Display for FilterItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FilterItem::Single(report_filter) => write!(f, "({})", report_filter),
            FilterItem::Composite(composite_filter) => write!(f, "({})", composite_filter),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageFilterPattern {
    Single(MessageFilter),
    Composite(CompositeFilter),
}

impl Display for MessageFilterPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MessageFilterPattern::Single(single) => write!(f, "{}", single),
            MessageFilterPattern::Composite(composite) => write!(f, "{}", composite),
        }
    }
}

impl EvaluableFilter for MessageFilterPattern {
    fn evaluate(&self, message: &Message) -> bool {
        match self {
            MessageFilterPattern::Single(single) => single.evaluate(message),
            MessageFilterPattern::Composite(composite) => composite.evaluate(message),
        }
    }
}

pub mod query {
    use std::fmt::Display;

    use crate::chatq;

    use super::{MessageFilterPattern, ModelConversionError};

    #[derive(Debug, Clone)]
    pub enum Limit {
        All,
        Amount(i32),
    }

    impl Display for Limit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Limit::All => write!(f, ""),
                Limit::Amount(amount) => write!(f, "LIMIT {}", amount),
            }
        }
    }

    impl From<Limit> for chatq::message_query_pattern::Limit {
        fn from(value: Limit) -> Self {
            match value {
                Limit::All => chatq::message_query_pattern::Limit::All(true),
                Limit::Amount(amount) => chatq::message_query_pattern::Limit::Amount(amount),
            }
        }
    }

    impl TryFrom<chatq::message_query_pattern::Limit> for Limit {
        type Error = ModelConversionError;

        fn try_from(value: chatq::message_query_pattern::Limit) -> Result<Self, Self::Error> {
            match value {
                chatq::message_query_pattern::Limit::All(_) => Ok(Limit::All),
                chatq::message_query_pattern::Limit::Amount(amount) => Ok(Limit::Amount(amount)),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct MessageQueryPattern {
        pub limit: Limit,
        pub filter: MessageFilterPattern,
    }

    impl From<MessageQueryPattern> for chatq::MessageQueryPattern {
        fn from(value: MessageQueryPattern) -> Self {
            Self {
                limit: Some(value.limit.into()),
                filter: Some(value.filter.into())
            }
        }
    }

    impl TryFrom<chatq::MessageQueryPattern> for MessageQueryPattern {
        type Error = ModelConversionError;

        fn try_from(value: chatq::MessageQueryPattern) -> Result<Self, Self::Error> {
            Ok(Self {
                limit: value.limit.ok_or(ModelConversionError::ValueNotProvided("limit"))?.try_into()?,
                filter: value.filter.ok_or(ModelConversionError::ValueNotProvided("limit"))?.try_into()?
            })
        }
    }
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

#[derive(Error, Debug)]
pub enum ModelConversionError {
    #[error("Inner value was not provided")]
    ValueNotProvided(&'static str),
    #[error("uuid conversion failure")]
    UuidConversion(#[from] uuid::Error),
}

impl TryFrom<chatq::MessageAudience> for MessageAudience {
    type Error = ModelConversionError;

    fn try_from(f: chatq::MessageAudience) -> Result<Self, Self::Error> {
        match f
            .audience
            .ok_or_else(|| ModelConversionError::ValueNotProvided("audience"))?
        {
            chatq::message_audience::Audience::Players(players) => Ok(MessageAudience::Players(
                players
                    .uuids
                    .into_iter()
                    .map(|x| Uuid::from_str(&x.value))
                    .collect::<Result<_, _>>()
                    .map_err(|err| ModelConversionError::UuidConversion(err))?,
            )),
            chatq::message_audience::Audience::Server(servers) => Ok(MessageAudience::Servers(
                servers
                    .servers
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<Server>>(),
            )),
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

impl TryFrom<chatq::MessageSource> for MessageSource {
    type Error = ModelConversionError;

    fn try_from(f: chatq::MessageSource) -> Result<Self, Self::Error> {
        match f
            .source
            .ok_or_else(|| ModelConversionError::ValueNotProvided("source"))?
        {
            chatq::message_source::Source::Players(players) => Ok(MessageSource::Players(
                players
                    .uuids
                    .into_iter()
                    .map(|x| Uuid::from_str(&x.value))
                    .collect::<Result<_, _>>()?,
            )),
            chatq::message_source::Source::Plugins(plugins) => Ok(MessageSource::Plugins(
                plugins
                    .plugins
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<Plugin>>(),
            )),
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
                .ok_or_else(|| ModelConversionError::ValueNotProvided("audince"))?
                .try_into()?,
            content: f.content,
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
        })
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
        }
    }
}

impl TryFrom<chatq::Uuid> for Uuid {
    type Error = uuid::Error;

    fn try_from(value: chatq::Uuid) -> Result<Self, Self::Error> {
        Uuid::from_str(&value.value)
    }
}

impl From<Uuid> for chatq::Uuid {
    fn from(value: Uuid) -> Self {
        chatq::Uuid {
            value: value.to_string(),
        }
    }
}

impl From<TimestampFilter> for chatq::message_filter_pattern::message_filter::TimestampFilter {
    fn from(value: TimestampFilter) -> Self {
        message_filter::TimestampFilter {
            condition: Some(match value {
                TimestampFilter::Equals(ts) => {
                    message_filter::timestamp_filter::Condition::Equals(prost_types::Timestamp {
                        seconds: ts.timestamp(),
                        nanos: 0,
                    })
                }
                TimestampFilter::GreaterThan(ts) => {
                    message_filter::timestamp_filter::Condition::GreaterThan(
                        prost_types::Timestamp {
                            seconds: ts.timestamp(),
                            nanos: 0,
                        },
                    )
                }
                TimestampFilter::LessThan(ts) => {
                    message_filter::timestamp_filter::Condition::LessThan(prost_types::Timestamp {
                        seconds: ts.timestamp(),
                        nanos: 0,
                    })
                }
                TimestampFilter::GreaterThanEqual(ts) => {
                    message_filter::timestamp_filter::Condition::GreaterThanEqual(
                        prost_types::Timestamp {
                            seconds: ts.timestamp(),
                            nanos: 0,
                        },
                    )
                }
                TimestampFilter::LessThanEqual(ts) => {
                    message_filter::timestamp_filter::Condition::LessThanEqual(
                        prost_types::Timestamp {
                            seconds: ts.timestamp(),
                            nanos: 0,
                        },
                    )
                }
            }),
        }
    }
}

impl TryFrom<chatq::message_filter_pattern::message_filter::TimestampFilter> for TimestampFilter {
    type Error = ModelConversionError;

    fn try_from(
        value: chatq::message_filter_pattern::message_filter::TimestampFilter,
    ) -> Result<Self, Self::Error> {
        match value
            .condition
            .ok_or_else(|| ModelConversionError::ValueNotProvided("condition"))?
        {
            Condition::Equals(ts) => Ok(Self::Equals(NaiveDateTime::from_timestamp(ts.seconds, 0))),
            Condition::GreaterThan(ts) => Ok(Self::GreaterThan(NaiveDateTime::from_timestamp(
                ts.seconds, 0,
            ))),
            Condition::LessThan(ts) => {
                Ok(Self::LessThan(NaiveDateTime::from_timestamp(ts.seconds, 0)))
            }
            Condition::GreaterThanEqual(ts) => Ok(Self::GreaterThanEqual(
                NaiveDateTime::from_timestamp(ts.seconds, 0),
            )),
            Condition::LessThanEqual(ts) => Ok(Self::LessThanEqual(NaiveDateTime::from_timestamp(
                ts.seconds, 0,
            ))),
        }
    }
}

impl From<AudienceFilter> for chatq::message_filter_pattern::message_filter::AudienceFilter {
    fn from(value: AudienceFilter) -> Self {
        Self {
            condition: Some(match value {
                AudienceFilter::Uuid(uuid) => {
                    message_filter::audience_filter::Condition::Player(uuid.into())
                }
                AudienceFilter::Server(server) => {
                    message_filter::audience_filter::Condition::Server(server.into())
                }
            }),
        }
    }
}

impl TryFrom<chatq::message_filter_pattern::message_filter::AudienceFilter> for AudienceFilter {
    type Error = ModelConversionError;

    fn try_from(value: message_filter::AudienceFilter) -> Result<Self, Self::Error> {
        match value
            .condition
            .ok_or_else(|| ModelConversionError::ValueNotProvided("condition"))?
        {
            message_filter::audience_filter::Condition::Player(player) => {
                Ok(Self::Uuid(player.try_into()?))
            }
            message_filter::audience_filter::Condition::Server(server) => {
                Ok(Self::Server(server.into()))
            }
        }
    }
}

impl From<SourceFilter> for chatq::message_filter_pattern::message_filter::SourceFilter {
    fn from(value: SourceFilter) -> Self {
        Self {
            condition: Some(match value {
                SourceFilter::Uuid(uuid) => {
                    message_filter::source_filter::Condition::Player(uuid.into())
                }
                SourceFilter::Plugin(plugin) => {
                    message_filter::source_filter::Condition::Plugin(plugin.into())
                }
            }),
        }
    }
}

impl TryFrom<chatq::message_filter_pattern::message_filter::SourceFilter> for SourceFilter {
    type Error = ModelConversionError;

    fn try_from(value: message_filter::SourceFilter) -> Result<Self, Self::Error> {
        match value
            .condition
            .ok_or_else(|| ModelConversionError::ValueNotProvided("condition"))?
        {
            message_filter::source_filter::Condition::Player(player) => {
                Ok(Self::Uuid(player.try_into()?))
            }
            message_filter::source_filter::Condition::Plugin(plugin) => {
                Ok(Self::Plugin(plugin.into()))
            }
        }
    }
}

impl From<CompositeFilter> for chatq::message_filter_pattern::CompositeFilter {
    fn from(value: CompositeFilter) -> Self {
        match value {
            CompositeFilter::Or(filter) => Self {
                operation: Operation::Or as i32,
                items: filter.into_iter().map(|op| op.into()).collect(),
            },
            CompositeFilter::And(filter) => Self {
                operation: Operation::And as i32,
                items: filter.into_iter().map(|op| op.into()).collect(),
            },
            CompositeFilter::Not(filter) => Self {
                operation: Operation::Not as i32,
                items: vec![(*filter).into()],
            },
        }
    }
}

impl From<MessageFilter> for chatq::message_filter_pattern::MessageFilter {
    fn from(value: MessageFilter) -> Self {
        match value {
            MessageFilter::InsertTimestamp(ts) => message_filter_pattern::MessageFilter {
                condition: Some(
                    chatq::message_filter_pattern::message_filter::Condition::InsertTimestamp(
                        ts.into(),
                    ),
                ),
            },
            MessageFilter::Audience(audience) => message_filter_pattern::MessageFilter {
                condition: Some(
                    chatq::message_filter_pattern::message_filter::Condition::Audience(
                        audience.into(),
                    ),
                ),
            },
            MessageFilter::Source(source) => message_filter_pattern::MessageFilter {
                condition: Some(
                    chatq::message_filter_pattern::message_filter::Condition::Source(source.into()),
                ),
            },
        }
    }
}

impl TryFrom<chatq::message_filter_pattern::MessageFilter> for MessageFilter {
    type Error = ModelConversionError;

    fn try_from(value: message_filter_pattern::MessageFilter) -> Result<Self, Self::Error> {
        match value
            .condition
            .ok_or_else(|| ModelConversionError::ValueNotProvided("condition"))?
        {
            message_filter_pattern::message_filter::Condition::InsertTimestamp(ts) => {
                Ok(Self::InsertTimestamp(ts.try_into()?))
            }
            message_filter_pattern::message_filter::Condition::Audience(audience) => {
                Ok(Self::Audience(audience.try_into()?))
            }
            message_filter_pattern::message_filter::Condition::Source(source) => {
                Ok(Self::Source(source.try_into()?))
            }
        }
    }
}

impl TryFrom<chatq::message_filter_pattern::CompositeFilter> for CompositeFilter {
    type Error = ModelConversionError;

    fn try_from(
        value: chatq::message_filter_pattern::CompositeFilter,
    ) -> Result<Self, Self::Error> {
        match Operation::from_i32(value.operation)
            .ok_or_else(|| ModelConversionError::ValueNotProvided("operation"))?
        {
            Operation::And => Ok(Self::And(
                value
                    .items
                    .into_iter()
                    .map(|op| op.try_into())
                    .collect::<Result<_, _>>()?,
            )),
            Operation::Or => Ok(Self::Or(
                value
                    .items
                    .into_iter()
                    .map(|op| op.try_into())
                    .collect::<Result<_, _>>()?,
            )),
            Operation::Not => Ok(Self::Not(Box::new(
                value
                    .items
                    .into_iter()
                    .next()
                    .ok_or_else(|| ModelConversionError::ValueNotProvided("NOT items.next()"))?
                    .try_into()?,
            ))),
        }
    }
}

impl From<FilterItem> for chatq::message_filter_pattern::composite_filter::FilterItem {
    fn from(value: FilterItem) -> Self {
        use chatq::message_filter_pattern::composite_filter;

        match value {
            FilterItem::Single(single) => composite_filter::FilterItem {
                r#type: Some(Type::Single(single.into())),
            },
            FilterItem::Composite(composite) => composite_filter::FilterItem {
                r#type: Some(Type::Composite(composite.into())),
            },
        }
    }
}

impl TryFrom<chatq::message_filter_pattern::composite_filter::FilterItem> for FilterItem {
    type Error = ModelConversionError;

    fn try_from(
        value: message_filter_pattern::composite_filter::FilterItem,
    ) -> Result<Self, Self::Error> {
        match value
            .r#type
            .ok_or_else(|| ModelConversionError::ValueNotProvided("type"))?
        {
            Type::Single(single) => Ok(Self::Single(single.try_into()?)),
            Type::Composite(composite) => Ok(Self::Composite(composite.try_into()?)),
        }
    }
}

impl From<MessageFilterPattern> for chatq::MessageFilterPattern {
    fn from(value: MessageFilterPattern) -> Self {
        match value {
            MessageFilterPattern::Single(single) => Self {
                primary_condition: PrimaryCondition::SingleFilter(single.into()).into(),
            },
            MessageFilterPattern::Composite(composite) => Self {
                primary_condition: PrimaryCondition::CompositeFilter(composite.into()).into(),
            },
        }
    }
}

impl TryFrom<chatq::MessageFilterPattern> for MessageFilterPattern {
    type Error = ModelConversionError;

    fn try_from(value: chatq::MessageFilterPattern) -> Result<Self, Self::Error> {
        match value
            .primary_condition
            .ok_or_else(|| ModelConversionError::ValueNotProvided("primary_condition"))?
        {
            PrimaryCondition::CompositeFilter(composite) => {
                Ok(Self::Composite(composite.try_into()?))
            }
            PrimaryCondition::SingleFilter(single) => Ok(Self::Single(single.try_into()?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::models::{
        AudienceFilter, CompositeFilter, EvaluableFilter, FilterItem, Message, MessageAudience,
        MessageFilter, MessageFilterPattern, MessageSource, Plugin, SourceFilter,
    };
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn filters() {
        let data = vec![
            Message {
                id: 0,
                timestamp: Utc::now().naive_utc(),
                source: MessageSource::Players(vec![
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    Uuid::from_u128(3),
                ]),
                audience: MessageAudience::Players(vec![
                    Uuid::from_u128(4),
                    Uuid::from_u128(5),
                    Uuid::from_u128(6),
                ]),
                content: "test2".to_string(),
            },
            Message {
                id: 0,
                timestamp: Utc::now().naive_utc(),
                source: MessageSource::Plugins(vec![
                    Plugin {
                        value: "qkernel".to_string(),
                    },
                    Plugin {
                        value: "reportas".to_string(),
                    },
                ]),
                audience: MessageAudience::Players(vec![
                    Uuid::from_u128(4),
                    Uuid::from_u128(7),
                    Uuid::from_u128(8),
                ]),
                content: "test1".to_string(),
            },
        ];

        let filter = MessageFilterPattern::Composite(CompositeFilter::Or(vec![
            FilterItem::Composite(CompositeFilter::And(vec![
                FilterItem::Single(MessageFilter::Audience(AudienceFilter::Uuid(
                    Uuid::from_u128(4),
                ))),
                FilterItem::Single(MessageFilter::Audience(AudienceFilter::Uuid(
                    Uuid::from_u128(10),
                ))),
            ])),
            FilterItem::Single(MessageFilter::Source(SourceFilter::Plugin(Plugin {
                value: "qkernel".to_string(),
            }))),
        ]));

        println!("{:#?}", filter);
        println!("{}", filter);

        let res = data
            .iter()
            .filter(|op| filter.evaluate(op))
            .collect::<Vec<_>>();

        println!("{:#?}", res)
    }
}
