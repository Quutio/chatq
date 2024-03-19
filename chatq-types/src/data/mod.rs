use crate::data::message::Message;
use crate::data::query::MessageQueryPattern;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod error;
pub mod filter;
pub mod message;
pub mod query;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub target: Uuid,
    pub query: MessageQueryPattern,
    pub taken: NaiveDateTime,
    pub messages: Vec<Message>,
}

#[cfg(feature = "proto")]
mod from_proto {
    use crate::chatq;
    use crate::data::error::ModelConversionError;
    use crate::data::error::ModelConversionError::ValueNotProvided;
    use crate::data::Snapshot;
    use chrono::{DateTime, NaiveDateTime};
    use prost_types::Timestamp;
    use uuid::Uuid;

    pub(crate) fn naive_from_proto(
        ts: prost_types::Timestamp,
    ) -> Result<NaiveDateTime, ModelConversionError> {
        Ok(DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
            .ok_or(ValueNotProvided("taken"))?
            .naive_utc())
    }

    pub(crate) fn proto_from_naive(ts: NaiveDateTime) -> prost_types::Timestamp {
        Timestamp {
            seconds: ts.and_utc().timestamp(),
            nanos: ts.and_utc().timestamp_subsec_nanos() as i32,
        }
    }

    impl TryFrom<chatq::Snapshot> for Snapshot {
        type Error = ModelConversionError;

        fn try_from(value: chatq::Snapshot) -> Result<Self, Self::Error> {
            let taken = value.taken.ok_or(ValueNotProvided("taken"))?;

            Ok(Self {
                id: value.id.ok_or(ValueNotProvided("id"))?.try_into()?,
                target: value.target.ok_or(ValueNotProvided("target"))?.try_into()?,
                query: value.query.ok_or(ValueNotProvided("query"))?.try_into()?,
                taken: naive_from_proto(taken)?,
                messages: value
                    .messages
                    .into_iter()
                    .map(|op| op.try_into())
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
    }

    impl From<Snapshot> for chatq::Snapshot {
        fn from(value: Snapshot) -> Self {
            Self {
                id: Some(value.id.into()),
                target: Some(value.target.into()),
                query: Some(value.query.into()),
                taken: Some(proto_from_naive(value.taken)),
                messages: value.messages.into_iter().map(|op| op.into()).collect(),
            }
        }
    }

    impl TryFrom<chatq::Uuid> for Uuid {
        type Error = sqlx::types::uuid::Error;

        fn try_from(value: chatq::Uuid) -> Result<Self, Self::Error> {
            Uuid::parse_str(&value.value)
        }
    }

    impl From<Uuid> for chatq::Uuid {
        fn from(value: Uuid) -> Self {
            Self {
                value: value.to_string(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::chatq::SnapshotGenerateRequest;
        use crate::data::filter::{
            AudienceFilter, CompositeFilter, EvaluableFilter, FilterItem, MessageFilter,
            MessageFilterPattern,
        };
        use crate::data::message::{Message, MessageAudience, MessageSource};
        use crate::data::query::{Limit, MessageQueryPattern};
        use chrono::Utc;
        use sqlx::types::Uuid;
        use std::str::FromStr;

        #[test]
        fn filters() {
            let data = vec![Message {
                id: 0,
                timestamp: Utc::now().naive_utc(),
                source: MessageSource::new(Uuid::from_u128(1)),
                audience: MessageAudience::new(vec![
                    Uuid::from_u128(4),
                    Uuid::from_u128(5),
                    Uuid::from_u128(6),
                ]),
                content: "test2".to_string(),
                context: "msg".to_string(),
            }];

            let filter =
                MessageFilterPattern::Composite(CompositeFilter::Or(vec![FilterItem::Composite(
                    CompositeFilter::And(vec![
                        FilterItem::Single(MessageFilter::Audience(AudienceFilter::Uuid(
                            Uuid::from_u128(4),
                        ))),
                        FilterItem::Single(MessageFilter::Audience(AudienceFilter::Uuid(
                            Uuid::from_u128(10),
                        ))),
                    ]),
                )]));

            println!("{:#?}", filter.clone());
            println!("{}", filter.clone());

            let pattern = MessageQueryPattern {
                limit: Limit::Amount(32),
                filter: filter.clone(),
            };

            println!("{}", pattern.limit);

            let res = data
                .iter()
                .filter(|op| filter.evaluate(op))
                .collect::<Vec<_>>();

            println!("{:#?}", res)
        }

        #[test]
        fn grpc_filter() {
            let target = Uuid::from_str("4a997b33-3c67-4204-97e0-d21d50d6dda0").unwrap();

            let filter = MessageFilterPattern::Single(MessageFilter::Audience(
                AudienceFilter::Uuid(target.clone()),
            ));

            let query = MessageQueryPattern {
                limit: Limit::All,
                filter,
            };

            println!("{:#?}", query);

            let grpcd: crate::chatq::MessageQueryPattern = query.into();

            println!("{:#?}", grpcd);

            let req = SnapshotGenerateRequest {
                target: Some(target.clone().into()),
                query: Some(grpcd),
            };
        }
    }
}
