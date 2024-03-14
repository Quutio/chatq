use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::chatq;
use crate::data::message::Message;
use crate::data::query::MessageQueryPattern;

pub mod filter;
pub mod message;
pub mod query;
pub mod error;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Snapshot {
    pub id: String,
    pub target: Uuid,
    pub query: MessageQueryPattern,
    pub messages: Vec<Message>,
}
impl TryFrom<chatq::Uuid> for Uuid {
    type Error = sqlx::types::uuid::Error;

    fn try_from(value: chatq::Uuid) -> Result<Self, Self::Error> {
        Uuid::parse_str(&value.value)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sqlx::types::Uuid;
    use crate::data::filter::{AudienceFilter, CompositeFilter, EvaluableFilter, FilterItem, MessageFilter, MessageFilterPattern};
    use crate::data::message::{Message, MessageAudience, MessageSource};

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

        println!("{:#?}", filter);
        println!("{}", filter);

        let res = data
            .iter()
            .filter(|op| filter.evaluate(op))
            .collect::<Vec<_>>();

        println!("{:#?}", res)
    }
}
