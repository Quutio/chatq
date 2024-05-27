use std::collections::HashMap;

use chrono::NaiveDateTime;
use sqlx::postgres::PgRow;
use sqlx::{query, Executor, PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use chatq_types::data::message::{Message, MessageAudience, MessageSource, MessageStub};
use chatq_types::data::query::MessageQueryPattern;
use chatq_types::data::Snapshot;

pub mod grpc;
pub mod message_handler;

pub struct ChatQDao {
    pub pool: PgPool,
}

impl ChatQDao {
    pub fn with_pool(pool: PgPool) -> Self {
        ChatQDao { pool }
    }

    async fn _insert_message<T>(conn: &mut T, stub: MessageStub) -> anyhow::Result<Message>
    where
        for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
        let audience = &stub.audience;

        let mut players = audience.players().clone();
        players.sort();

        let padded: String = players
            .into_iter()
            .map(|op| op.to_string())
            .collect::<Vec<_>>()
            .join("|");

        let players = audience.players();

        let audience_id = sqlx::query!(
            r#"
INSERT INTO audiences (users,users_hash)
VALUES ($1,MD5($2)) ON CONFLICT (users) DO UPDATE SET users = EXCLUDED.users
RETURNING id
            "#,
            players,
            padded
        )
        .fetch_one(&mut *conn)
        .await?
        .id;

        let source_id = sqlx::query!(
            r#"
INSERT INTO sources (uuid)
VALUES ($1) ON CONFLICT (uuid) DO UPDATE SET uuid = EXCLUDED.uuid
RETURNING id
            "#,
            stub.source.player()
        )
        .fetch_one(&mut *conn)
        .await?
        .id;

        let source = &stub.audience;
        for player in source.players() {
            let source_id = sqlx::query!(
                r#"
INSERT INTO sources (uuid)
VALUES ($1) ON CONFLICT (uuid) DO UPDATE SET uuid = EXCLUDED.uuid
RETURNING id
                "#,
                player
            )
            .fetch_one(&mut *conn)
            .await?
            .id;
            sqlx::query!(
                r#"
INSERT INTO source_audiences (source_id,audience_id)
VALUES ($1,$2) ON CONFLICT DO NOTHING"#,
                source_id,
                audience_id
            )
            .execute(&mut *conn)
            .await?;
        }

        let message_id = sqlx::query!(
            r#"
INSERT INTO messages (issued,content,audience_id,source_id,context)
VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING
RETURNING id
            "#,
            stub.timestamp,
            stub.content,
            audience_id,
            source_id,
            stub.context
        )
        .fetch_one(&mut *conn)
        .await?
        .id;

        Ok(Message {
            id: message_id,
            timestamp: stub.timestamp,
            source: stub.source,
            audience: stub.audience,
            content: stub.content,
            context: stub.context,
        })
    }

    pub async fn insert_message(&self, stub: MessageStub) -> anyhow::Result<Message> {
        let mut txn = self.pool.begin().await?;

        let res = Self::_insert_message(&mut *txn, stub).await;
        txn.commit().await?;
        res
    }

    pub async fn _generate_snapshot<T>(
        conn: &mut T,
        target: Uuid,
        query: &MessageQueryPattern,
    ) -> anyhow::Result<Snapshot>
    where
        for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
        let source_id = sqlx::query!(
            r#"SELECT id FROM sources WHERE uuid = $1"#,
            &sqlx::types::Uuid::parse_str(&target.to_string())?
        )
        .fetch_one(&mut *conn)
        .await?
        .id;

        let snapshot_rows = sqlx::query!(
            r#"
INSERT INTO query_snapshots (id,query_json,target,snapshot_taken)
VALUES (gen_random_uuid (),$1,$2,current_timestamp)
RETURNING id, snapshot_taken
            "#,
            serde_json::to_string(&query).unwrap(),
            source_id
        )
        .fetch_one(&mut *conn)
        .await?;

        let (snapshot_id, timestamp) = (snapshot_rows.id, snapshot_rows.snapshot_taken);

        let queried = Self::_query_messages(&mut *conn, query).await?;

        for message in &queried {
            sqlx::query!(
                r#"INSERT INTO message_snapshots (snapshot_id,message_id) VALUES ($1,$2)"#,
                snapshot_id,
                message.id
            )
            .execute(&mut *conn)
            .await?;
        }

        Ok(Snapshot {
            id: snapshot_id,
            target,
            query: query.clone(),
            taken: timestamp,
            messages: queried,
        })
    }

    pub async fn generate_snapshot(
        &self,
        target: Uuid,
        query: &MessageQueryPattern,
    ) -> anyhow::Result<Snapshot> {
        let mut txn: Transaction<Postgres> = self.pool.begin().await?;
        let res = Self::_generate_snapshot(&mut *txn, target, query).await;
        txn.commit().await?;
        res
    }

    pub async fn _fetch_snapshot<T>(conn: &mut T, id: Uuid) -> anyhow::Result<Option<Snapshot>>
    where
        for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
        let snapshot_query = match query!(
            r#"SELECT query_json,snapshot_taken,target FROM query_snapshots WHERE id = $1"#,
            id
        )
        .fetch_optional(&mut *conn)
        .await?
        {
            None => return Ok(None),
            Some(val) => val,
        };
        let (query, taken, target) = (
            serde_json::from_str::<MessageQueryPattern>(&snapshot_query.query_json)?,
            snapshot_query.snapshot_taken,
            snapshot_query.target,
        );
        let messages = Self::_query_messages(&mut *conn, &query).await?;

        let target = query!(
            r#"SELECT sources.* FROM sources WHERE sources.id = $1 "#,
            target
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(Some(Snapshot {
            id,
            target: target.uuid,
            query,
            taken,
            messages,
        }))
    }

    pub async fn fetch_snapshot(&self, id: Uuid) -> anyhow::Result<Option<Snapshot>> {
        let mut txn: Transaction<Postgres> = self.pool.begin().await?;
        let res = Self::_fetch_snapshot(&mut *txn, id).await;
        txn.commit().await?;
        res
    }

    pub async fn _query_messages<T>(
        conn: &mut T,
        query: &MessageQueryPattern,
    ) -> anyhow::Result<Vec<Message>>
    where
        for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
        let restriction = query.filter.to_string();
        let limit = query.limit.to_string();

        println!("{}", &format!(
            r#"
            SELECT messages.*
            FROM messages
            LEFT JOIN source_audiences ON source_audiences.audience_id = messages.audience_id
            LEFT JOIN sources as audience_sources ON audience_sources.id = source_audiences.source_id
            LEFT JOIN sources ON sources.id = messages.source_id
            WHERE {} GROUP BY messages.id ORDER BY messages.issued DESC {}
            "#,
            restriction, limit
        ));
        
        let all_messages = sqlx::query(&format!(
            r#"
            SELECT messages.*
            FROM messages
            LEFT JOIN source_audiences ON source_audiences.audience_id = messages.audience_id
            LEFT JOIN sources as audience_sources ON audience_sources.id = source_audiences.source_id
            LEFT JOIN sources ON sources.id = messages.source_id
            WHERE {} GROUP BY messages.id ORDER BY messages.issued DESC {}
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
            .fetch_all(&mut *conn)
            .await?;

        let mut audiences = HashMap::new();
        let mut sources = HashMap::new();

        for message in &all_messages {
            let bar = sqlx::query!(r#"SELECT * FROM audiences WHERE id = $1"#, message.3 as i32)
                .fetch_one(&mut *conn)
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
            .fetch_one(&mut *conn)
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
                src = MessageSource::new(source.0);
            } else {
                continue;
            }

            res.push(Message {
                id: message.0,
                timestamp: message.1,
                source: src,
                audience: aud,
                content: message.2.to_string(),
                context: message.5.to_string(),
            })
        }

        Ok(res)
    }

    pub async fn query_messages(
        &self,
        query: &MessageQueryPattern,
    ) -> anyhow::Result<Vec<Message>> {
        let mut txn: Transaction<Postgres> = self.pool.begin().await?;
        let res = Self::_query_messages(&mut *txn, query).await;
        txn.commit().await?;
        res
    }
}
