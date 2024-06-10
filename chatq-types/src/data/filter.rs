use std::fmt;
use std::fmt::{Display, Formatter};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::message::Message;

pub trait EvaluableFilter {
    fn evaluate(&self, message: &Message) -> bool;
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ContentFilter {
    Like(String),
    ILike(String),
    SimilarTo(String),
    NotSimilarTo(String),
    Regexp(String),
    NotRegexp(String),
}

#[cfg(feature = "proto")]
pub mod query {
    use crate::data::filter::{
        to_player_seq, AudienceFilter, CompositeFilter, ContentFilter, ContextFilter, FilterItem,
        MessageFilter, MessageFilterPattern, SourceFilter, TimestampFilter,
    };
    use crate::data::query::Limit;
    use sqlx::{Database, Postgres, QueryBuilder};
    use uuid::Uuid;

    pub trait Queryable<DB: Database> {
        fn append_query(&self, builder: &mut QueryBuilder<'_, DB>);
    }

    impl Queryable<Postgres> for ContentFilter {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                ContentFilter::Like(pattern) => {
                    builder
                        .push("messages.content LIKE ")
                        .push_bind(pattern.to_string());
                }
                ContentFilter::ILike(pattern) => {
                    builder
                        .push("messages.content ILIKE ")
                        .push_bind(pattern.to_string());
                }
                ContentFilter::SimilarTo(pattern) => {
                    builder
                        .push("messages.content SIMILAR TO ")
                        .push_bind(pattern.to_string());
                }
                ContentFilter::NotSimilarTo(pattern) => {
                    builder
                        .push("messages.content NOT SIMILAR TO ")
                        .push_bind(pattern.to_string());
                }
                ContentFilter::Regexp(pattern) => {
                    builder
                        .push("messages.content ~ ")
                        .push_bind(pattern.to_string());
                }
                ContentFilter::NotRegexp(pattern) => {
                    builder
                        .push("messages.content !~ ")
                        .push_bind(pattern.to_string());
                }
            }
        }
    }

    impl Queryable<Postgres> for TimestampFilter {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                TimestampFilter::Equals(ts) => {
                    builder.push("messages.issued = ").push_bind(*ts);
                }
                TimestampFilter::GreaterThan(ts) => {
                    builder.push("messages.issued > ").push_bind(*ts);
                }
                TimestampFilter::LessThan(ts) => {
                    builder.push("messages.issued < ").push_bind(*ts);
                }
                TimestampFilter::GreaterThanEqual(ts) => {
                    builder.push("messages.issued >= ").push_bind(*ts);
                }
                TimestampFilter::LessThanEqual(ts) => {
                    builder.push("messages.issued <= ").push_bind(*ts);
                }
            }
        }
    }

    impl Queryable<Postgres> for AudienceFilter {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                AudienceFilter::Uuid(uuid) => {
                    builder.push("audience_sources.uuid = ").push_bind(*uuid);
                }
            }
        }
    }

    impl Queryable<Postgres> for SourceFilter {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                SourceFilter::Uuid(uuid) => {
                    builder.push("sources.uuid = ").push_bind(*uuid);
                }
            }
        }
    }

    impl Queryable<Postgres> for ContextFilter {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                ContextFilter::Context(context) => {
                    builder
                        .push("messages.context = ")
                        .push_bind(context.to_string());
                }
            }
        }
    }

    impl Queryable<Postgres> for FilterItem {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                FilterItem::Single(single) => single.append_query(builder),
                FilterItem::Composite(composite) => composite.append_query(builder),
            }
        }
    }

    impl Queryable<Postgres> for MessageFilter {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                MessageFilter::InsertTimestamp(ts) => ts.append_query(builder),
                MessageFilter::Audience(audience) => audience.append_query(builder),
                MessageFilter::Source(source) => source.append_query(builder),
                MessageFilter::Context(context) => context.append_query(builder),
                MessageFilter::Content(content) => content.append_query(builder),
            }
        }
    }

    impl Queryable<Postgres> for CompositeFilter {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                CompositeFilter::Or(items) => {
                    builder.push("( ");
                    for (index, filter) in items.iter().enumerate() {
                        if index > 0 {
                            builder.push(" OR ");
                        }
                        filter.append_query(builder);
                    }
                    builder.push(" )");
                }
                CompositeFilter::And(items) => {
                    let mut others: Vec<&FilterItem> = Vec::new();
                    let mut audiences: Vec<Uuid> = Vec::new();

                    for item in items {
                        match item {
                            FilterItem::Single(single) => match single {
                                MessageFilter::Audience(audience) => match audience {
                                    AudienceFilter::Uuid(uuid) => {
                                        audiences.push(*uuid);
                                    }
                                },
                                _ => {
                                    others.push(item);
                                }
                            },
                            FilterItem::Composite(_) => {
                                others.push(item);
                            }
                        }
                    }

                    builder.push("( ");

                    if !audiences.is_empty() {
                        builder.push("audiences.users_hash = MD5(");
                        builder.push_bind(to_player_seq(&audiences));
                        builder.push(") ");

                        if !others.is_empty() {
                            builder.push("AND ");
                        }
                    }

                    for (index, filter) in others.iter().enumerate() {
                        if index > 0 {
                            builder.push(" AND ");
                        }
                        filter.append_query(builder);
                    }

                    builder.push(" )");
                }
                CompositeFilter::Not(item) => {
                    builder.push("( ");
                    builder.push("NOT ");
                    item.append_query(builder);
                    builder.push(" )");
                }
            }
        }
    }

    impl Queryable<Postgres> for MessageFilterPattern {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                MessageFilterPattern::Single(single) => {
                    single.append_query(builder);
                }
                MessageFilterPattern::Composite(composite) => {
                    composite.append_query(builder);
                }
            }
        }
    }

    impl Queryable<Postgres> for Limit {
        fn append_query(&self, builder: &mut QueryBuilder<'_, Postgres>) {
            match self {
                Limit::All => {}
                Limit::Amount(amount) => {
                    builder.push(" LIMIT ");
                    builder.push_bind(*amount);
                }
            }
        }
    }
}

impl Display for ContentFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ContentFilter::Like(left) => {
                write!(f, "messages.content LIKE '{}'", left)
            }
            ContentFilter::ILike(left) => {
                write!(f, "messages.content ILIKE '{}'", left)
            }
            ContentFilter::SimilarTo(left) => {
                write!(f, "messages.content SIMILAR TO '{}'", left)
            }
            ContentFilter::NotSimilarTo(left) => {
                write!(f, "messages.content NOT SIMILAR TO '{}'", left)
            }
            ContentFilter::Regexp(left) => {
                write!(f, "messages.content ~ '{}'", left)
            }
            ContentFilter::NotRegexp(left) => {
                write!(f, "messages.content !~ '{}'", left)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
                write!(f, "messages.issued = '{}'", ts)
            }
            TimestampFilter::GreaterThan(ts) => {
                write!(f, "messages.issued > '{}'", ts)
            }
            TimestampFilter::LessThan(ts) => {
                write!(f, "messages.issued < '{}'", ts)
            }
            TimestampFilter::GreaterThanEqual(ts) => {
                write!(f, "messages.issued >= '{}'", ts)
            }
            TimestampFilter::LessThanEqual(ts) => {
                write!(f, "messages.issued <= '{}'", ts)
            }
        }
    }
}

pub fn to_player_seq(uuids: &[Uuid]) -> String {
    let mut players = uuids.to_owned();
    players.sort();

    players
        .into_iter()
        .map(|op| op.to_string())
        .collect::<Vec<_>>()
        .join("|")
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum AudienceFilter {
    Uuid(uuid::Uuid),
}

impl Display for AudienceFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AudienceFilter::Uuid(uuid) => {
                write!(f, "audience_sources.uuid = \'{}\'", uuid)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum SourceFilter {
    Uuid(Uuid),
}

impl Display for SourceFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SourceFilter::Uuid(uuid) => {
                write!(f, "sources.uuid = \'{}\'", uuid)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ContextFilter {
    Context(String),
}

impl Display for ContextFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ContextFilter::Context(context) => {
                write!(f, "messages.context = \'{}\'", context)
            }
        }
    }
}

impl EvaluableFilter for AudienceFilter {
    fn evaluate(&self, message: &Message) -> bool {
        match self {
            AudienceFilter::Uuid(uuid) => message.audience.players().contains(uuid),
        }
    }
}

impl EvaluableFilter for SourceFilter {
    fn evaluate(&self, message: &Message) -> bool {
        match self {
            SourceFilter::Uuid(uuid) => *message.source.player() == *uuid,
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

impl EvaluableFilter for ContextFilter {
    fn evaluate(&self, message: &Message) -> bool {
        match self {
            ContextFilter::Context(context) => *context == message.content,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum MessageFilter {
    InsertTimestamp(TimestampFilter),
    Audience(AudienceFilter),
    Source(SourceFilter),
    Context(ContextFilter),
    Content(ContentFilter),
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
            MessageFilter::Context(context) => {
                write!(f, "{}", context)
            }
            MessageFilter::Content(content) => {
                write!(f, "{}", content)
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
            MessageFilter::Context(filter) => filter.evaluate(message),
            MessageFilter::Content(_content) => unimplemented!(),
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
                let mut others: Vec<&FilterItem> = Vec::new();
                let mut audiences: Vec<Uuid> = Vec::new();

                for item in items {
                    match item {
                        FilterItem::Single(single_filter) => match single_filter {
                            MessageFilter::Audience(audience) => match audience {
                                AudienceFilter::Uuid(uuid) => {
                                    audiences.push(*uuid);
                                }
                            },
                            _ => {
                                others.push(item);
                            }
                        },
                        FilterItem::Composite(_) => others.push(item),
                    }
                }

                let mut joined = others
                    .iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>();

                if !audiences.is_empty() {
                    joined.push(format!(
                        "audiences.users_hash = MD5('{}')",
                        to_player_seq(&audiences)
                    ))
                }

                write!(f, "{}", joined.join(" AND "))
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[cfg(feature = "proto")]
pub mod from_proto {
    use crate::chatq;
    use crate::chatq::message_filter_pattern;
    use crate::data::error::ModelConversionError;
    use crate::data::filter::{
        AudienceFilter, CompositeFilter, ContentFilter, ContextFilter, FilterItem, MessageFilter,
        MessageFilterPattern, SourceFilter, TimestampFilter,
    };
    use std::str::FromStr;
    use uuid::Uuid;

    use crate::chatq::message_filter_pattern::composite_filter::filter_item::Type;
    use crate::chatq::message_filter_pattern::{message_filter, Operation, PrimaryCondition};
    use crate::data::from_proto::{naive_from_proto, proto_from_naive};

    impl From<ContentFilter> for message_filter::ContentFilter {
        fn from(value: ContentFilter) -> Self {
            message_filter::ContentFilter {
                condition: Some(match value {
                    ContentFilter::Like(left) => {
                        message_filter::content_filter::Condition::Like(left)
                    }
                    ContentFilter::ILike(left) => {
                        message_filter::content_filter::Condition::ILike(left)
                    }
                    ContentFilter::SimilarTo(left) => {
                        message_filter::content_filter::Condition::SimilarTo(left)
                    }
                    ContentFilter::NotSimilarTo(left) => {
                        message_filter::content_filter::Condition::NotSimilarTo(left)
                    }
                    ContentFilter::Regexp(left) => {
                        message_filter::content_filter::Condition::Regexp(left)
                    }
                    ContentFilter::NotRegexp(left) => {
                        message_filter::content_filter::Condition::NotRegexp(left)
                    }
                }),
            }
        }
    }

    impl TryFrom<message_filter::ContentFilter> for ContentFilter {
        type Error = ModelConversionError;

        fn try_from(value: message_filter::ContentFilter) -> Result<Self, Self::Error> {
            use crate::chatq::message_filter_pattern::message_filter::content_filter::Condition;

            match value
                .condition
                .ok_or(ModelConversionError::ValueNotProvided("content filter"))?
            {
                Condition::Like(left) => Ok(Self::Like(left)),
                Condition::ILike(left) => Ok(Self::ILike(left)),
                Condition::SimilarTo(left) => Ok(Self::SimilarTo(left)),
                Condition::NotSimilarTo(left) => Ok(Self::NotSimilarTo(left)),
                Condition::Regexp(left) => Ok(Self::Regexp(left)),
                Condition::NotRegexp(left) => Ok(Self::NotRegexp(left)),
            }
        }
    }

    impl From<TimestampFilter> for message_filter::TimestampFilter {
        fn from(value: TimestampFilter) -> Self {
            use crate::chatq::message_filter_pattern::message_filter::timestamp_filter::Condition;

            message_filter::TimestampFilter {
                condition: Some(match value {
                    TimestampFilter::Equals(ts) => Condition::Equals(proto_from_naive(ts)),
                    TimestampFilter::GreaterThan(ts) => {
                        Condition::GreaterThan(proto_from_naive(ts))
                    }
                    TimestampFilter::LessThan(ts) => Condition::LessThan(proto_from_naive(ts)),
                    TimestampFilter::GreaterThanEqual(ts) => {
                        Condition::GreaterThanEqual(proto_from_naive(ts))
                    }
                    TimestampFilter::LessThanEqual(ts) => {
                        Condition::LessThanEqual(proto_from_naive(ts))
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
            use crate::chatq::message_filter_pattern::message_filter::timestamp_filter::Condition;

            match value
                .condition
                .ok_or(ModelConversionError::ValueNotProvided("condition"))?
            {
                Condition::Equals(ts) => Ok(Self::Equals(naive_from_proto(ts)?)),
                Condition::GreaterThan(ts) => Ok(Self::GreaterThan(naive_from_proto(ts)?)),
                Condition::LessThan(ts) => Ok(Self::LessThan(naive_from_proto(ts)?)),
                Condition::GreaterThanEqual(ts) => {
                    Ok(Self::GreaterThanEqual(naive_from_proto(ts)?))
                }
                Condition::LessThanEqual(ts) => Ok(Self::LessThanEqual(naive_from_proto(ts)?)),
            }
        }
    }

    impl From<AudienceFilter> for chatq::message_filter_pattern::message_filter::AudienceFilter {
        fn from(value: AudienceFilter) -> Self {
            Self {
                player: Some(match value {
                    AudienceFilter::Uuid(uuid) => chatq::Uuid {
                        value: uuid.to_string(),
                    },
                }),
            }
        }
    }

    impl TryFrom<chatq::message_filter_pattern::message_filter::AudienceFilter> for AudienceFilter {
        type Error = ModelConversionError;

        fn try_from(value: message_filter::AudienceFilter) -> Result<Self, Self::Error> {
            let player = value
                .player
                .map(|op| Uuid::from_str(&op.value).map_err(ModelConversionError::UuidConversion))
                .ok_or(ModelConversionError::ValueNotProvided("player"))??;
            Ok(AudienceFilter::Uuid(player))
        }
    }

    impl From<SourceFilter> for chatq::message_filter_pattern::message_filter::SourceFilter {
        fn from(value: SourceFilter) -> Self {
            Self {
                player: Some(match value {
                    SourceFilter::Uuid(uuid) => chatq::Uuid {
                        value: uuid.to_string(),
                    },
                }),
            }
        }
    }

    impl TryFrom<chatq::message_filter_pattern::message_filter::SourceFilter> for SourceFilter {
        type Error = ModelConversionError;

        fn try_from(value: message_filter::SourceFilter) -> Result<Self, Self::Error> {
            let player = value
                .player
                .map(|op| Uuid::from_str(&op.value).map_err(ModelConversionError::UuidConversion))
                .ok_or(ModelConversionError::ValueNotProvided("player"))??;
            Ok(SourceFilter::Uuid(player))
        }
    }

    impl From<ContextFilter> for chatq::message_filter_pattern::message_filter::ContextFilter {
        fn from(value: ContextFilter) -> Self {
            Self {
                context: match value {
                    ContextFilter::Context(context) => context,
                },
            }
        }
    }

    impl TryFrom<chatq::message_filter_pattern::message_filter::ContextFilter> for ContextFilter {
        type Error = ModelConversionError;

        fn try_from(value: message_filter::ContextFilter) -> Result<Self, Self::Error> {
            Ok(ContextFilter::Context(value.context))
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
                        chatq::message_filter_pattern::message_filter::Condition::Source(
                            source.into(),
                        ),
                    ),
                },
                MessageFilter::Context(context) => message_filter_pattern::MessageFilter {
                    condition: Some(
                        chatq::message_filter_pattern::message_filter::Condition::Context(
                            context.into(),
                        ),
                    ),
                },
                MessageFilter::Content(content) => message_filter_pattern::MessageFilter {
                    condition: Some(
                        chatq::message_filter_pattern::message_filter::Condition::Content(
                            content.into(),
                        ),
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
                .ok_or(ModelConversionError::ValueNotProvided("condition"))?
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
                message_filter_pattern::message_filter::Condition::Context(context) => {
                    Ok(Self::Context(context.try_into()?))
                }
                message_filter_pattern::message_filter::Condition::Content(content) => {
                    Ok(Self::Content(content.try_into()?))
                }
            }
        }
    }

    impl TryFrom<chatq::message_filter_pattern::CompositeFilter> for CompositeFilter {
        type Error = ModelConversionError;

        fn try_from(
            value: chatq::message_filter_pattern::CompositeFilter,
        ) -> Result<Self, Self::Error> {
            match value.operation() {
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
                        .ok_or(ModelConversionError::ValueNotProvided("NOT items.next()"))?
                        .try_into()?,
                ))),
            }
        }
    }

    impl From<FilterItem> for chatq::message_filter_pattern::composite_filter::FilterItem {
        fn from(value: FilterItem) -> Self {
            use crate::chatq::message_filter_pattern::composite_filter;

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
                .ok_or(ModelConversionError::ValueNotProvided("type"))?
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
                .ok_or(ModelConversionError::ValueNotProvided("primary_condition"))?
            {
                PrimaryCondition::CompositeFilter(composite) => {
                    Ok(Self::Composite(composite.try_into()?))
                }
                PrimaryCondition::SingleFilter(single) => Ok(Self::Single(single.try_into()?)),
            }
        }
    }
}
