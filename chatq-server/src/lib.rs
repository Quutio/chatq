use anyhow::Context;
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;

use chatq_types::data::filter::query::Queryable;
use chatq_types::data::filter::{
    to_player_seq, CompositeFilter, FilterItem, MessageFilter, MessageFilterPattern,
    TimestampFilter,
};
use chrono::{NaiveDateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{query, Executor, PgPool, Postgres, QueryBuilder, Row, Transaction};
use tokio::sync::RwLock;
use uuid::Uuid;

use chatq_types::data::message::{Message, MessageAudience, MessageSource, MessageStub};
use chatq_types::data::query::{
    Limit, MessageQueryPattern, MessageQueryRequest, MessageQueryRequestKind, QueryMessageResponse,
};
use chatq_types::data::Snapshot;
use num::integer::div_ceil;

pub mod grpc;
pub mod message_handler;

#[derive(Clone)]
pub struct SessionData {
    session_id: Uuid,
    issued: NaiveDateTime,
    page_size: i32,
    #[allow(dead_code)]
    page_number: i32,
    pattern: MessageQueryPattern,
}

pub struct ChatQDao {
    pub pool: PgPool,
    pub session_cache: Arc<RwLock<HashMap<Uuid, SessionData>>>,
}

struct MessageDetails {
    id: i64,
    issued: chrono::NaiveDateTime,
    content: String,
    audience_id: i64,
    source_id: i64,
    context: String,
}

impl<'a> ChatQDao {
    pub fn with_pool(pool: PgPool) -> Self {
        ChatQDao {
            pool,
            session_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn upsert_audience<T>(conn: &mut T, players: &[Uuid]) -> anyhow::Result<i64>
    where
        for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
        let padded = to_player_seq(&players);
        let audience_id = sqlx::query!(
            r#"
INSERT INTO audiences (users,users_hash)
VALUES ($1,MD5($2)) ON CONFLICT (users) DO UPDATE SET users = EXCLUDED.users
RETURNING id
            "#,
            &players,
            padded
        )
        .fetch_one(&mut *conn)
        .await?
        .id;

        Ok(audience_id)
    }

    async fn upsert_source<T>(conn: &mut T, player: &Uuid) -> anyhow::Result<i64>
    where
        for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
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

        Ok(source_id)
    }

    async fn insert_message(&self, stub: &MessageStub) -> anyhow::Result<Message> {
        let mut txn = self.pool.begin().await?;

        let audience_id = Self::upsert_audience(&mut *txn, stub.audience.players()).await?;
        let source_id = Self::upsert_source(&mut *txn, stub.source.player()).await?;

        sqlx::query!(
            r#"
INSERT INTO source_audiences (source_id,audience_id)
VALUES ($1,$2) ON CONFLICT DO NOTHING"#,
            source_id,
            audience_id
        )
        .execute(&mut *txn)
        .await?;

        let message_id = sqlx::query!(
            r#"
INSERT INTO messages (issued, content, audience_id, source_id, context)
VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING
RETURNING id
            "#,
            stub.timestamp,
            stub.content,
            audience_id,
            source_id,
            stub.context
        )
        .fetch_one(&mut *txn)
        .await?
        .id;

        txn.commit().await?;

        Ok(Message {
            id: message_id,
            timestamp: stub.timestamp,
            source: stub.source.clone(),
            audience: stub.audience.clone(),
            content: stub.content.clone(),
            context: stub.context.clone(),
        })
    }

    async fn _generate_snapshot<T>(
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

    async fn _fetch_snapshot<T>(conn: &mut T, id: Uuid) -> anyhow::Result<Option<Snapshot>>
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
        let mut builder = QueryBuilder::new(
            r#"
        SELECT messages.*
            FROM messages
            LEFT JOIN source_audiences ON source_audiences.audience_id = messages.audience_id
            LEFT JOIN sources as audience_sources ON audience_sources.id = source_audiences.source_id
            LEFT JOIN sources ON sources.id = messages.source_id
            LEFT JOIN audiences ON audiences.id = source_audiences.audience_id
        "#,
        );

        builder.push(" WHERE ");
        query.filter.append_query(&mut builder);
        builder.push(" GROUP BY messages.id ORDER BY messages.issued DESC");
        query.limit.append_query(&mut builder);

        dbg!("{}", builder.sql());

        let all_messages = builder
            .build()
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

    async fn _message_details_query<T>(
        conn: &mut T,
        all_messages: &[MessageDetails],
    ) -> anyhow::Result<Vec<Message>>
    where
        for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
        let mut audiences = HashMap::new();
        let mut sources = HashMap::new();

        for message in all_messages {
            let bar = sqlx::query!(
                r#"SELECT * FROM audiences WHERE id = $1"#,
                message.audience_id as i32
            )
            .fetch_one(&mut *conn)
            .await?;

            audiences.insert(message.id, (bar.users, message.audience_id));
        }

        for message in all_messages {
            let bar = sqlx::query!(
                r#"
                SELECT * FROM sources WHERE id = $1
                "#,
                message.source_id as i32
            )
            .fetch_one(&mut *conn)
            .await?;

            sources.insert(message.id, (bar.uuid, message.source_id));
        }

        let mut res: Vec<Message> = Vec::new();

        for message in all_messages {
            let audiences = audiences.get(&message.id);
            let sources = sources.get(&message.id);

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
                id: message.id,
                timestamp: message.issued,
                source: src,
                audience: aud,
                content: message.content.to_string(),
                context: message.context.to_string(),
            })
        }

        Ok(res)
    }

    pub async fn _query_messages_raw<T>(
        &self,
        conn: &mut T,
        query: MessageQueryRequest,
    ) -> anyhow::Result<QueryMessageResponse>
    where
        for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
        let count_base_sql = r#"SELECT COUNT(*) as count FROM ("#;

        let base_sql = r#"
        SELECT messages.*
            FROM messages
            LEFT JOIN source_audiences ON source_audiences.audience_id = messages.audience_id
            LEFT JOIN sources as audience_sources ON audience_sources.id = source_audiences.source_id
            LEFT JOIN sources ON sources.id = messages.source_id
            LEFT JOIN audiences ON audiences.id = source_audiences.audience_id
        "#;

        match query.kind {
            MessageQueryRequestKind::Sessionless(sessionless) => {
                let mut count_builder = QueryBuilder::new(count_base_sql);

                let mut builder = QueryBuilder::new(base_sql);

                let mut query = sessionless.pattern.clone();
                let page_size = sessionless.page_size;

                match &mut query.limit {
                    Limit::All => query.limit = Limit::Amount(page_size),
                    Limit::Amount(amount) => query.limit = Limit::Amount(min(page_size, *amount)),
                }

                builder.push(" WHERE ");
                query.filter.append_query(&mut builder);
                builder.push(" GROUP BY messages.id ORDER BY messages.issued DESC");

                count_builder.push(base_sql);
                count_builder.push(" WHERE ");
                query.filter.append_query(&mut count_builder);
                count_builder.push(" GROUP BY messages.id ORDER BY messages.issued DESC");
                count_builder.push(") as derivedQuery");

                query.limit.append_query(&mut builder);

                dbg!("{:#?}", &query);
                dbg!("{}", &builder.sql());

                dbg!("{}", &builder.sql());
                dbg!("{}", &count_builder.sql());

                let count_messages = count_builder
                    .build()
                    .map(|row: PgRow| {
                        let count: i64 = row.get("count");
                        count
                    })
                    .fetch_one(&mut *conn)
                    .await?;

                let all_messages = builder
                    .build()
                    .map(|row: PgRow| {
                        let message_id: i64 = row.get("id");
                        let issued: NaiveDateTime = row.get("issued");
                        let content: String = row.get("content");
                        let audience_id: i64 = row.get("audience_id");
                        let source_id: i64 = row.get("source_id");
                        let context: String = row.get("context");

                        MessageDetails {
                            id: message_id,
                            issued,
                            content,
                            audience_id,
                            source_id,
                            context,
                        }
                    })
                    .fetch_all(&mut *conn)
                    .await?;

                dbg!("{:#?}", &query.filter);

                let res = Self::_message_details_query(&mut *conn, &all_messages).await?;

                let mut write = self.session_cache.write().await;
                let session = SessionData {
                    session_id: Uuid::new_v4(),
                    issued: Utc::now().naive_local(),
                    page_size: sessionless.page_size,
                    page_number: 1,
                    pattern: sessionless.pattern,
                };

                write.insert(session.session_id, session.clone());

                let resp = QueryMessageResponse {
                    session_key: session.session_id,
                    total_count: count_messages as i32,
                    current_page: 1,
                    total_pages: div_ceil(count_messages as i32, sessionless.page_size),
                    messages: res,
                };
                Ok(resp)
            }
            MessageQueryRequestKind::WithSession(with_session) => {
                let session: SessionData;

                {
                    let write = self.session_cache.write().await;
                    session = write
                        .get(&with_session.session_id)
                        .context("session not present")?
                        .clone();
                }

                let mut query = session.pattern.clone();
                let page_number = with_session.page_number;
                let page_size = session.page_size;

                let offset = (page_number - 1) * page_size;

                let mut count_builder = QueryBuilder::new(count_base_sql);

                let mut builder = QueryBuilder::new(base_sql);

                match query.limit {
                    Limit::All => query.limit = Limit::Amount(page_size),
                    Limit::Amount(amount) => query.limit = Limit::Amount(min(page_size, amount)),
                }

                let new_filter: MessageFilterPattern;

                let timestamp_filter =
                    MessageFilter::InsertTimestamp(TimestampFilter::LessThanEqual(session.issued));

                dbg!("{:#?}", &query.filter);

                match query.filter {
                    MessageFilterPattern::Single(filter) => {
                        new_filter = MessageFilterPattern::Composite(CompositeFilter::And(vec![
                            FilterItem::Single(filter),
                            FilterItem::Single(timestamp_filter),
                        ]))
                    }
                    MessageFilterPattern::Composite(filter) => {
                        new_filter = MessageFilterPattern::Composite(CompositeFilter::And(vec![
                            FilterItem::Single(timestamp_filter),
                            FilterItem::Composite(filter),
                        ]))
                    }
                }

                query.filter = new_filter.clone();

                builder.push(" WHERE ");
                query.filter.append_query(&mut builder);
                builder.push(" GROUP BY messages.id ORDER BY messages.issued DESC");

                count_builder.push(base_sql);
                count_builder.push(" WHERE ");
                query.filter.append_query(&mut count_builder);
                count_builder.push(" GROUP BY messages.id ORDER BY messages.issued DESC");
                count_builder.push(") as derivedQuery");

                query.limit.append_query(&mut builder);
                builder.push(" OFFSET ");
                builder.push_bind(offset);

                dbg!("{:?}", &new_filter);
                dbg!("{}", &builder.sql());

                let count_messages = count_builder
                    .build()
                    .map(|row| {
                        let count: i64 = row.get("count");
                        count
                    })
                    .fetch_one(&mut *conn)
                    .await?;

                let all_messages = builder
                    .build()
                    .map(|row: PgRow| {
                        let message_id: i64 = row.get("id");
                        let issued: NaiveDateTime = row.get("issued");
                        let content: String = row.get("content");
                        let audience_id: i64 = row.get("audience_id");
                        let source_id: i64 = row.get("source_id");
                        let context: String = row.get("context");

                        MessageDetails {
                            id: message_id,
                            issued,
                            content,
                            audience_id,
                            source_id,
                            context,
                        }
                    })
                    .fetch_all(&mut *conn)
                    .await?;

                let res = Self::_message_details_query(&mut *conn, &all_messages).await?;

                let resp = QueryMessageResponse {
                    session_key: session.session_id,
                    total_count: count_messages as i32,
                    current_page: page_number,
                    total_pages: div_ceil(count_messages as i32, page_size),
                    messages: res,
                };

                {
                    let mut write = self.session_cache.write().await;
                    let session = SessionData {
                        session_id: session.session_id,
                        issued: session.issued,
                        page_size,
                        page_number,
                        pattern: session.pattern.clone(),
                    };

                    write.insert(session.session_id, session);
                }

                Ok(resp)
            }
        }
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

    pub async fn query_messages_raw(
        &self,
        query: MessageQueryRequest,
    ) -> anyhow::Result<QueryMessageResponse> {
        let mut txn: Transaction<Postgres> = self.pool.begin().await?;
        let res = self._query_messages_raw(&mut *txn, query).await;
        txn.commit().await?;
        res
    }
}
