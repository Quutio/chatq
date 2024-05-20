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
        AudienceFilter, CompositeFilter, ContextFilter, FilterItem, MessageFilter,
        MessageFilterPattern, SourceFilter, TimestampFilter,
    };
    use std::str::FromStr;
    use uuid::Uuid;

    use crate::chatq::message_filter_pattern::composite_filter::filter_item::Type;
    use crate::chatq::message_filter_pattern::message_filter::timestamp_filter::Condition;
    use crate::chatq::message_filter_pattern::{message_filter, Operation, PrimaryCondition};
    use crate::data::from_proto::{naive_from_proto, proto_from_naive};

    impl From<TimestampFilter> for message_filter::TimestampFilter {
        fn from(value: TimestampFilter) -> Self {
            message_filter::TimestampFilter {
                condition: Some(match value {
                    TimestampFilter::Equals(ts) => {
                        message_filter::timestamp_filter::Condition::Equals(proto_from_naive(ts))
                    }
                    TimestampFilter::GreaterThan(ts) => {
                        message_filter::timestamp_filter::Condition::GreaterThan(proto_from_naive(
                            ts,
                        ))
                    }
                    TimestampFilter::LessThan(ts) => {
                        message_filter::timestamp_filter::Condition::LessThan(proto_from_naive(ts))
                    }
                    TimestampFilter::GreaterThanEqual(ts) => {
                        message_filter::timestamp_filter::Condition::GreaterThanEqual(
                            proto_from_naive(ts),
                        )
                    }
                    TimestampFilter::LessThanEqual(ts) => {
                        message_filter::timestamp_filter::Condition::LessThanEqual(
                            proto_from_naive(ts),
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
            }
        }
    }

    impl TryFrom<chatq::message_filter_pattern::CompositeFilter> for CompositeFilter {
        type Error = ModelConversionError;

        fn try_from(
            value: chatq::message_filter_pattern::CompositeFilter,
        ) -> Result<Self, Self::Error> {
            match Operation::from_i32(value.operation)
                .ok_or(ModelConversionError::ValueNotProvided("operation"))?
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
