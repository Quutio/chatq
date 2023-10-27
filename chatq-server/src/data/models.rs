use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::chatq;
use crate::chatq::message_filter_pattern::{message_filter, Operation, PrimaryCondition};
use crate::chatq::message_filter_pattern::message_filter::timestamp_filter::Condition;
use crate::chatq::message_filter_pattern;
use crate::chatq::message_filter_pattern::composite_filter::filter_item::Type;

/*
        match self.filter {
            MessageFilter::Before(before) => {
                self.iter.find(|x| x.timestamp < before)
            },
            MessageFilter::After(after) => {
                self.iter.find(|x| x.timestamp > after)
            },
            MessageFilter::Player(player) => {
                self.iter.find(|x| {
                    match x.audience {
                        MessageAudience::Players(players) => { return players.contains(&player) },
                        MessageAudience::Servers(_) => {}
                    }
                    match x.source {
                        MessageSource::Players(players) => { return players.contains(&player) },
                        MessageSource::Plugins(_) => {},
                    }

                    false
                })
            },
            MessageFilter::Server(server) => {

            }
        }
*/
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

trait EvaluableFilter {
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
        return match self {
            AudienceFilter::Uuid(uuid) => match &message.audience {
                MessageAudience::Players(players) => players.contains(uuid),
                MessageAudience::Servers(_) => false,
            },
            AudienceFilter::Server(server) => match &message.audience {
                MessageAudience::Players(_) => false,
                MessageAudience::Servers(servers) => servers.contains(server),
            },
        };
    }
}

impl EvaluableFilter for SourceFilter {
    fn evaluate(&self, message: &Message) -> bool {
        return match self {
            SourceFilter::Uuid(uuid) => match &message.source {
                MessageSource::Players(players) => players.contains(uuid),
                MessageSource::Plugins(_) => false,
            },
            SourceFilter::Plugin(plugin) => match &message.source {
                MessageSource::Players(_) => false,
                MessageSource::Plugins(plugins) => plugins.contains(plugin),
            },
        };
    }
}

impl EvaluableFilter for TimestampFilter {
    fn evaluate(&self, message: &Message) -> bool {
        return match self {
            TimestampFilter::Equals(ts) => &message.timestamp == ts,
            TimestampFilter::GreaterThan(ts) => &message.timestamp > ts,
            TimestampFilter::LessThan(ts) => &message.timestamp < ts,
            TimestampFilter::GreaterThanEqual(ts) => &message.timestamp >= ts,
            TimestampFilter::LessThanEqual(ts) => &message.timestamp <= ts,
        };
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
        return match self {
            MessageFilter::InsertTimestamp(filter) => filter.evaluate(message),
            MessageFilter::Audience(filter) => filter.evaluate(message),
            MessageFilter::Source(filter) => filter.evaluate(message),
        };
    }
}

impl EvaluableFilter for FilterItem {
    fn evaluate(&self, message: &Message) -> bool {
        return match &self {
            FilterItem::Single(single) => single.evaluate(message),
            FilterItem::Composite(composite) => composite.evaluate(message),
        };
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
            MessageFilterPattern::Composite(composite) => write!(f, "{}", composite)
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
            timestamp: NaiveDateTime::from_timestamp(f.timestamp.unwrap().seconds, 0),
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

impl From<chatq::MessageStub> for MessageStub {
    fn from(f: chatq::MessageStub) -> Self {
        Self {
            timestamp: NaiveDateTime::from_timestamp(f.timestamp.unwrap().seconds, 0),
            source: f.source.unwrap().into(),
            audience: f.audience.unwrap().into(),
            content: f.content,
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
        }
    }
}

impl From<chatq::Uuid> for Uuid {
    fn from(value: chatq::Uuid) -> Self {
        Uuid::from_str(&value.value).unwrap()
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
        use chatq::message_filter_pattern::message_filter;

        message_filter::TimestampFilter {
            condition: Some(match value {
                TimestampFilter::Equals(ts) => {
                    message_filter::timestamp_filter::Condition::Equals(
                        prost_types::Timestamp {
                            seconds: ts.timestamp(),
                            nanos: 0,
                        }
                    )
                }
                TimestampFilter::GreaterThan(ts) => {
                    message_filter::timestamp_filter::Condition::GreaterThan(
                        prost_types::Timestamp {
                            seconds: ts.timestamp(),
                            nanos: 0,
                        }
                    )
                }
                TimestampFilter::LessThan(ts) => {
                    message_filter::timestamp_filter::Condition::LessThan(
                        prost_types::Timestamp {
                            seconds: ts.timestamp(),
                            nanos: 0,
                        }
                    )
                }
                TimestampFilter::GreaterThanEqual(ts) => {
                    message_filter::timestamp_filter::Condition::GreaterThanEqual(
                        prost_types::Timestamp {
                            seconds: ts.timestamp(),
                            nanos: 0,
                        }
                    )
                }
                TimestampFilter::LessThanEqual(ts) => {
                    message_filter::timestamp_filter::Condition::LessThanEqual(
                        prost_types::Timestamp {
                            seconds: ts.timestamp(),
                            nanos: 0,
                        }
                    )
                }
            }),
        }
    }
}

impl From<chatq::message_filter_pattern::message_filter::TimestampFilter> for TimestampFilter {
    fn from(value: chatq::message_filter_pattern::message_filter::TimestampFilter) -> Self {
        match value.condition.unwrap() {
            Condition::Equals(ts) => {Self::Equals(
                NaiveDateTime::from_timestamp(ts.seconds, 0)
            )}
            Condition::GreaterThan(ts) => {
                Self::GreaterThan(
                    NaiveDateTime::from_timestamp(ts.seconds, 0)
                )
            }
            Condition::LessThan(ts) => {
                Self::LessThan(
                    NaiveDateTime::from_timestamp(ts.seconds, 0)
                )
            }
            Condition::GreaterThanEqual(ts) => {
                Self::GreaterThanEqual(
                    NaiveDateTime::from_timestamp(ts.seconds, 0)
                )
            }
            Condition::LessThanEqual(ts) => {
                Self::LessThanEqual(
                    NaiveDateTime::from_timestamp(ts.seconds, 0)
                )
            }
        }
    }
}

impl From<AudienceFilter> for chatq::message_filter_pattern::message_filter::AudienceFilter {
    fn from(value: AudienceFilter) -> Self {
        use chatq::message_filter_pattern::message_filter;

        Self {
            condition: Some(
                match value {
                    AudienceFilter::Uuid(uuid) => {
                        message_filter::audience_filter::Condition::Player(uuid.into())
                    }
                    AudienceFilter::Server(server) => {
                        message_filter::audience_filter::Condition::Server(server.into())
                    }
                }
            )
        }
    }
}

impl From<chatq::message_filter_pattern::message_filter::AudienceFilter> for AudienceFilter {
    fn from(value: message_filter::AudienceFilter) -> Self {
        use chatq::message_filter_pattern::message_filter;

        match value.condition.unwrap() {
            message_filter::audience_filter::Condition::Player(player) => {
                Self::Uuid(player.into())
            }
            message_filter::audience_filter::Condition::Server(server) => {
                Self::Server(server.into())
            }
        }
    }
}

impl From<SourceFilter> for chatq::message_filter_pattern::message_filter::SourceFilter {
    fn from(value: SourceFilter) -> Self {
        use chatq::message_filter_pattern::message_filter;

        Self {
            condition: Some(
                match value {
                    SourceFilter::Uuid(uuid) => {
                        message_filter::source_filter::Condition::Player(uuid.into())
                    }
                    SourceFilter::Plugin(plugin) => {
                        message_filter::source_filter::Condition::Plugin(plugin.into())
                    }
                }
            )
        }
    }
}

impl From<chatq::message_filter_pattern::message_filter::SourceFilter> for SourceFilter {
    fn from(value: message_filter::SourceFilter) -> Self {
        use chatq::message_filter_pattern::message_filter;

        match value.condition.unwrap() {
            message_filter::source_filter::Condition::Player(player) => {
                Self::Uuid(player.into())
            }
            message_filter::source_filter::Condition::Plugin(plugin) => {
                Self::Plugin(plugin.into())
            }
        }
    }
}

impl From<CompositeFilter> for chatq::message_filter_pattern::CompositeFilter {
    fn from(value: CompositeFilter) -> Self {
        match value {
            CompositeFilter::Or(filter) => {
                return Self {
                    operation: Operation::Or as i32,
                    items: filter.into_iter().map(|op| op.into()).collect(),
                }
            }
            CompositeFilter::And(filter) => {
                return Self {
                    operation: Operation::And as i32,
                    items: filter.into_iter().map(|op| op.into()).collect()
                }
            }
            CompositeFilter::Not(filter) => {
                return Self {
                    operation: Operation::Not as i32,
                    items: vec![(*filter).into()]
                }
            }
        }
    }
}

impl From<MessageFilter> for chatq::message_filter_pattern::MessageFilter {
    fn from(value: MessageFilter) -> Self {
        use chatq::message_filter_pattern;

        match value {
            MessageFilter::InsertTimestamp(ts) => {
                message_filter_pattern::MessageFilter {
                    condition: Some(chatq::message_filter_pattern::message_filter::Condition::InsertTimestamp(ts.into())),
                }
            }
            MessageFilter::Audience(audience) => {
                message_filter_pattern::MessageFilter {
                    condition: Some(chatq::message_filter_pattern::message_filter::Condition::Audience(audience.into()))
                }
            }
            MessageFilter::Source(source) => {
                message_filter_pattern::MessageFilter {
                    condition: Some(chatq::message_filter_pattern::message_filter::Condition::Source(source.into()))
                }
            }
        }
    }
}

impl From<chatq::message_filter_pattern::MessageFilter> for MessageFilter {
    fn from(value: message_filter_pattern::MessageFilter) -> Self {
        match value.condition.unwrap() {
            message_filter_pattern::message_filter::Condition::InsertTimestamp(ts) => {
                Self::InsertTimestamp(ts.into())
            }
            message_filter_pattern::message_filter::Condition::Audience(audience) => {
                Self::Audience(audience.into())
            }
            message_filter_pattern::message_filter::Condition::Source(source) => {
                Self::Source(source.into())
            }
        }
    }
}

impl From<chatq::message_filter_pattern::CompositeFilter> for CompositeFilter {
    fn from(value: chatq::message_filter_pattern::CompositeFilter) -> Self {
        match Operation::from_i32(value.operation).unwrap() {
            Operation::And => {
                Self::And(value.items.into_iter().map(|op| op.into()).collect())
            }
            Operation::Or => {
                Self::Or(value.items.into_iter().map(|op| op.into()).collect())
            }
            Operation::Not => {
                Self::Not(Box::new(value.items.into_iter().next().unwrap().into()))
            }
        }
    }
}

impl From<FilterItem> for chatq::message_filter_pattern::composite_filter::FilterItem {
    fn from(value: FilterItem) -> Self {
        use chatq::message_filter_pattern::composite_filter;


        match value {
            FilterItem::Single(single) => {
                composite_filter::FilterItem {
                    r#type: Some(composite_filter::filter_item::Type::Single(single.into())),
                }
            }
            FilterItem::Composite(composite) => {
                composite_filter::FilterItem {
                    r#type: Some(composite_filter::filter_item::Type::Composite(composite.into())),
                }
            }
        }
    }
}

impl From<chatq::message_filter_pattern::composite_filter::FilterItem> for FilterItem {
    fn from(value: message_filter_pattern::composite_filter::FilterItem) -> Self {

        match value.r#type.unwrap() {
            Type::Single(single) => {
                Self::Single(single.into())
            }
            Type::Composite(composite) => {
                Self::Composite(composite.into())
            }
        }
    }
}

impl From<MessageFilterPattern> for chatq::MessageFilterPattern {
    fn from(value: MessageFilterPattern) -> Self {
        match value {
            MessageFilterPattern::Single(single) => {
                Self {
                    primary_condition: PrimaryCondition::SingleFilter(single.into()).into(),
                }
            }
            MessageFilterPattern::Composite(composite) => {
                Self {
                    primary_condition: PrimaryCondition::CompositeFilter(composite.into()).into()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::models::{AudienceFilter, CompositeFilter, EvaluableFilter, FilterItem, Message, MessageAudience, MessageFilter, MessageFilterPattern, MessageSource, Plugin, SourceFilter};
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
            FilterItem::Single(MessageFilter::Source(SourceFilter::Plugin(
                Plugin { value: "qkernel".to_string() },
            ))),
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
