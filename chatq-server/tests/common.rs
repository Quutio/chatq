use chatq_types::data::filter::{
    AudienceFilter, ContentFilter, MessageFilter, MessageFilterPattern, SourceFilter,
};
use chatq_types::data::message::{Message, MessageAudience, MessageSource, MessageStub};
use chatq_types::data::query::{
    Limit, MessageQueryPattern, MessageQueryRequest, MessageQueryRequestKind, Sessionless,
};
use chrono::NaiveDateTime;
use lib::ChatQDao;
use sqlx::PgPool;
use uuid::Uuid;

use lib::ports::MessageRepo;

mod insert {
    #[sqlx::test(migrations = "./migrations")]
    async fn insert_message_ok(pool: PgPool) {
        let chatq = ChatQDao::with_pool(pool);

        let ts: NaiveDateTime = NaiveDateTime::from_timestamp(777, 0);
        let source = MessageSource::new(Uuid::new_v4());
        let audience = MessageAudience::new(vec![Uuid::new_v4(), Uuid::new_v4(), *source.player()]);

        let stub = MessageStub {
            timestamp: ts,
            source: source.clone(),
            audience: audience.clone(),
            content: "test".to_string(),
            context: "test".to_string(),
        };

        let result = chatq.insert(stub).await.unwrap();

        assert_eq!(result.content, "test");
        assert_eq!(result.timestamp, ts);
        assert_eq!(result.audience, audience);
        assert_eq!(result.context, "test");
        assert_eq!(result.source, source);
    }
    use super::*;
}

mod query {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn message_source(pool: PgPool) {
        let chatq = ChatQDao::with_pool(pool);

        let needles = vec![
            MessageStub {
                timestamp: NaiveDateTime::from_timestamp(1, 0),
                source: MessageSource::new(Uuid::from_u128(1)),
                audience: MessageAudience::new(vec![
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    Uuid::from_u128(3),
                ]),
                content: "functoriality".to_string(),
                context: "lambdaland".to_string(),
            },
            MessageStub {
                timestamp: NaiveDateTime::from_timestamp(2, 0),
                source: MessageSource::new(Uuid::from_u128(1)),
                audience: MessageAudience::new(vec![
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    Uuid::from_u128(3),
                ]),
                content: "naturality".to_string(),
                context: "lambdaland".to_string(),
            },
        ];

        let mut haystack = Vec::from_iter(needles.clone());
        haystack.push(MessageStub {
            timestamp: NaiveDateTime::from_timestamp(3, 0),
            source: MessageSource::new(Uuid::from_u128(2)),
            audience: MessageAudience::new(vec![
                Uuid::from_u128(1),
                Uuid::from_u128(2),
                Uuid::from_u128(3),
            ]),
            content: "topoi".to_string(),
            context: "lambdaland".to_string(),
        });

        for message in haystack {
            chatq.insert(message).await.unwrap();
        }

        let query = MessageQueryRequest {
            kind: MessageQueryRequestKind::Sessionless(Sessionless {
                page_size: 777,
                row_limit: None,
                pattern: MessageQueryPattern {
                    limit: Limit::All,
                    filter: MessageFilterPattern::Single(MessageFilter::Source(
                        SourceFilter::Uuid(Uuid::from_u128(1)),
                    )),
                },
            }),
        };

        let result = chatq.query(query).await.unwrap();

        let message_to_stub = |message: Message| -> MessageStub {
            MessageStub {
                timestamp: message.timestamp,
                source: message.source,
                audience: message.audience,
                context: message.context,
                content: message.content,
            }
        };

        let mut stubs = result
            .messages
            .into_iter()
            .map(message_to_stub)
            .collect::<Vec<_>>();

        assert!(
            stubs.iter().all(|x| needles.contains(&x))
                && needles.iter().all(|x| stubs.contains(&x)),
        )
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn message_audience(pool: PgPool) {
        let chatq = ChatQDao::with_pool(pool);

        let needles = vec![
            MessageStub {
                timestamp: NaiveDateTime::from_timestamp(1, 0),
                source: MessageSource::new(Uuid::from_u128(1)),
                audience: MessageAudience::new(vec![
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    Uuid::from_u128(3),
                ]),
                content: "functoriality".to_string(),
                context: "lambdaland".to_string(),
            },
            MessageStub {
                timestamp: NaiveDateTime::from_timestamp(2, 0),
                source: MessageSource::new(Uuid::from_u128(1)),
                audience: MessageAudience::new(vec![
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    Uuid::from_u128(3),
                ]),
                content: "naturality".to_string(),
                context: "lambdaland".to_string(),
            },
        ];

        let mut haystack = Vec::from_iter(needles.clone());
        haystack.push(MessageStub {
            timestamp: NaiveDateTime::from_timestamp(3, 0),
            source: MessageSource::new(Uuid::from_u128(2)),
            audience: MessageAudience::new(vec![
                Uuid::from_u128(1),
                Uuid::from_u128(7),
                Uuid::from_u128(5),
            ]),
            content: "topoi".to_string(),
            context: "lambdaland".to_string(),
        });

        for message in haystack {
            chatq.insert(message).await.unwrap();
        }

        let query = MessageQueryRequest {
            kind: MessageQueryRequestKind::Sessionless(Sessionless {
                page_size: 777,
                row_limit: None,
                pattern: MessageQueryPattern {
                    limit: Limit::All,
                    filter: MessageFilterPattern::Single(MessageFilter::Audience(
                        AudienceFilter::Subset(vec![Uuid::from_u128(2), Uuid::from_u128(3)]),
                    )),
                },
            }),
        };

        let result = chatq.query(query).await.unwrap();

        let message_to_stub = |message: Message| -> MessageStub {
            MessageStub {
                timestamp: message.timestamp,
                source: message.source,
                audience: message.audience,
                context: message.context,
                content: message.content,
            }
        };

        let mut stubs = result
            .messages
            .into_iter()
            .map(message_to_stub)
            .collect::<Vec<_>>();

        assert!(
            stubs.iter().all(|x| needles.contains(&x))
                && needles.iter().all(|x| stubs.contains(&x)),
        )
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn message_content(pool: PgPool) {
        let chatq = ChatQDao::with_pool(pool);

        let needles = vec![
            MessageStub {
                timestamp: NaiveDateTime::from_timestamp(1, 0),
                source: MessageSource::new(Uuid::from_u128(1)),
                audience: MessageAudience::new(vec![
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    Uuid::from_u128(3),
                ]),
                content: "functoriality".to_string(),
                context: "lambdaland".to_string(),
            },
            MessageStub {
                timestamp: NaiveDateTime::from_timestamp(2, 0),
                source: MessageSource::new(Uuid::from_u128(1)),
                audience: MessageAudience::new(vec![
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    Uuid::from_u128(3),
                ]),
                content: "naturality".to_string(),
                context: "lambdaland".to_string(),
            },
        ];

        let mut haystack = Vec::from_iter(needles.clone());
        haystack.push(MessageStub {
            timestamp: NaiveDateTime::from_timestamp(3, 0),
            source: MessageSource::new(Uuid::from_u128(2)),
            audience: MessageAudience::new(vec![
                Uuid::from_u128(1),
                Uuid::from_u128(7),
                Uuid::from_u128(5),
            ]),
            content: "topoi".to_string(),
            context: "lambdaland".to_string(),
        });

        for message in haystack {
            chatq.insert(message).await.unwrap();
        }

        let query = MessageQueryRequest {
            kind: MessageQueryRequestKind::Sessionless(Sessionless {
                page_size: 777,
                row_limit: None,
                pattern: MessageQueryPattern {
                    limit: Limit::All,
                    filter: MessageFilterPattern::Single(MessageFilter::Content(
                        ContentFilter::ILike("%lity".to_string()),
                    )),
                },
            }),
        };

        let result = chatq.query(query).await.unwrap();

        let message_to_stub = |message: Message| -> MessageStub {
            MessageStub {
                timestamp: message.timestamp,
                source: message.source,
                audience: message.audience,
                context: message.context,
                content: message.content,
            }
        };

        let mut stubs = result
            .messages
            .into_iter()
            .map(message_to_stub)
            .collect::<Vec<_>>();

        assert!(
            stubs.iter().all(|x| needles.contains(&x))
                && needles.iter().all(|x| stubs.contains(&x)),
        )
    }
}
