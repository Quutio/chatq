
use chrono::NaiveDateTime;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use chatq_types::data::message::{Message, MessageAudience, MessageSource, MessageStub};
use chatq_types::data::query::MessageQueryPattern;

pub mod grpc;
pub mod message_handler;

pub struct ChatQDao {
    pub pool: PgPool,
}

impl ChatQDao {
    pub fn with_pool(pool: PgPool) -> Self {
        ChatQDao { pool }
    }

    pub async fn insert_message(&self, stub: MessageStub) -> anyhow::Result<Message> {
        let mut txn = self.pool.begin().await?;

        let audience = &stub.audience;
        let padded: String = audience
            .players()
            .into_iter()
            .map(|op| op.to_string())
            .collect::<Vec<_>>()
            .join("|")
            .into();
        let players = audience.players();

        let audience_id = sqlx::query!(r#"INSERT INTO audiences (users,users_hash) VALUES ($1,MD5($2)) ON CONFLICT (users) DO UPDATE SET users = EXCLUDED.users RETURNING id"#, players, padded)
            .fetch_one(&mut txn).await?.id;

        let source_id = sqlx::query!(r#"INSERT INTO sources (uuid) VALUES ($1) ON CONFLICT (uuid) DO UPDATE SET uuid = EXCLUDED.uuid RETURNING id"#, stub.source.player())
            .fetch_one(&mut txn).await?.id;

        let source = &stub.audience;
        for player in source.players() {
            let source_id = sqlx::query!(r#"INSERT INTO sources (uuid) VALUES ($1) ON CONFLICT (uuid) DO UPDATE SET uuid = EXCLUDED.uuid RETURNING id"#, player)
                .fetch_one(&mut txn).await?.id;
            sqlx::query!(r#"INSERT INTO source_audiences (source_id,audience_id) VALUES ($1,$2) ON CONFLICT DO NOTHING"#, source_id,audience_id)
                .execute(&mut txn).await?;
        }

        let message_id = sqlx::query!(r#"INSERT INTO messages (issued,content,audience_id,source_id,context) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING RETURNING id"#,
            stub.timestamp, stub.content, audience_id, source_id, stub.context
        ).fetch_one(&mut txn).await?.id;

        txn.commit().await?;

        Ok(Message {
            id: message_id,
            timestamp: stub.timestamp,
            source: stub.source,
            audience: stub.audience,
            content: stub.content,
            context: stub.context,
        })
    }

    pub async fn query_messages(
        &self,
        query: &MessageQueryPattern,
    ) -> anyhow::Result<Vec<Message>> {
        let mut txn = self.pool.begin().await?;

        let restriction = query.filter.to_string();
        let limit = query.limit.to_string();

        let all_messages = sqlx::query(&format!(
            r#"
            SELECT messages.*
            FROM messages
            LEFT JOIN source_audiences ON source_audiences.audience_id = messages.audience_id
            LEFT JOIN sources as audience_sources ON audience_sources.id = source_audiences.source_id
            LEFT JOIN sources ON sources.id = messages.source_id
            WHERE {} GROUP BY messages.id {}
            "#,
            restriction, limit
        ))
        .map(|row: PgRow| {
            let message_id: i64 = row.get("id");
            let issued: NaiveDateTime = row.get("issued");
            let content: String = row.get("content");
            let audience_id: i64 = row.get("audience_id");
            let source_id: i64 = row.get("source_id");
            let context: String = row.get("context");

            (message_id, issued, content, audience_id, source_id, context)
        })
        .fetch_all(&mut txn)
        .await?;

        let mut audiences = HashMap::new();
        let mut sources = HashMap::new();

        for message in &all_messages {
            let bar = sqlx::query!(r#"SELECT * FROM audiences WHERE id = $1"#, message.3 as i32)
                .fetch_one(&mut txn)
                .await?;

            audiences.insert(message.0, (bar.users, message.3));
        }

        for message in &all_messages {
            let bar = sqlx::query!(
                r#"
                SELECT * FROM sources WHERE id = $1
                "#,
                message.4 as i32
            )
            .fetch_one(&mut txn)
            .await?;

            sources.insert(message.0, (bar.uuid, message.4));
        }

        let mut res: Vec<Message> = Vec::new();

        for message in &all_messages {
            let audiences = audiences.get(&message.0);
            let sources = sources.get(&message.0);

            let aud: MessageAudience;
            let src: MessageSource;

            if let Some(audiences) = audiences {
                aud = MessageAudience::new(audiences.0.clone());
            } else {
                continue;
            }

            if let Some(source) = sources {
                src = MessageSource::new(source.0.clone());
            } else {
                continue;
            }

            res.push(Message {
                id: message.0,
                timestamp: message.1,
                source: src,
                audience: aud,
                content: message.2.to_string(),
                context: message.5.to_string()
            })
        }

        txn.commit().await?;

        Ok(res)
    }
}
