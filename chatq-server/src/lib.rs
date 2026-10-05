#![feature(int_roundings)]

use anyhow::{anyhow, Context};
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;

use chatq_types::data::filter::query::Queryable;
use chatq_types::data::filter::{
    to_player_seq, CompositeFilter, FilterItem, MessageFilter, MessageFilterPattern,
    TimestampFilter,
};
use chrono::{NaiveDateTime, Utc};
use num::integer::div_ceil;
use sqlx::postgres::PgRow;
use sqlx::{query, Executor, PgPool, Postgres, QueryBuilder, Row, Transaction};
use tokio::sync::RwLock;
use tonic::async_trait;
use uuid::Uuid;

use chatq_types::data::message::{Message, MessageAudience, MessageSource, MessageStub};
use chatq_types::data::query::{
    Limit, MessageQueryPattern, MessageQueryRequest, MessageQueryRequestKind, QueryMessageResponse,
};
use chatq_types::data::Snapshot;
use crate::ports::{MessageRepo, MessageRepoResult, RepoError};

pub mod grpc;
pub mod message_handler;
pub mod ports;
pub mod logic;
pub mod event_channel;

#[derive(Clone, Debug)]
pub struct SessionData {
    session_id: Uuid,
    issued: NaiveDateTime,
    page_size: i32,
    #[allow(dead_code)]
    page_number: i32,
    pattern: MessageQueryPattern,
    cursors: HashMap<i32, Cursor>,
    total_pages: i32,
    total_messages_count: i32
}

#[derive(Copy, Clone, Debug)]
pub enum Cursor {
    First,
    Other(CursorData)
}

#[derive(Copy, Clone, Debug)]
pub struct CursorData {
    last_issued: NaiveDateTime,
    last_id: i64
}

pub struct ChatQDao {
    pub pool: PgPool,
    pub session_cache: Arc<RwLock<HashMap<Uuid, SessionData>>>,
}

// Rust 1.75+ natively supports async traits, no #[async_trait] needed
#[async_trait]
impl MessageRepo for ChatQDao {
    async fn insert(&self, message: MessageStub) -> MessageRepoResult<Message> {
        self.insert_message(&message).await
            .map_err(|err| RepoError::Arbitrary(err.to_string()))
    }

    async fn query(&self, query: MessageQueryRequest) -> MessageRepoResult<QueryMessageResponse> {
        self.query_messages_raw(query).await
            .map_err(|err| RepoError::Arbitrary(err.to_string()))
    }
}

impl ChatQDao {
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

    pub(crate) async fn insert_message(&self, stub: &MessageStub) -> anyhow::Result<Message> {
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
        SELECT
            messages.id,
            messages.issued as timestamp,
            messages.content,
            messages.context,
            sources.uuid as source_uuid,
            audiences.users as audience_users
        FROM messages
        INNER JOIN sources ON sources.id = messages.source_id
        INNER JOIN audiences ON audiences.id = messages.audience_id
        "#,
        );

        builder.push(" WHERE ");
        query.filter.append_query(&mut builder);
        builder.push(" ORDER BY messages.id DESC ");
        query.limit.append_query(&mut builder);

        let all_messages = builder
            .build()
            .map(|row: PgRow| {
                Message {
                    id: row.get("id"),
                    timestamp: row.get("timestamp"),
                    source: MessageSource::new(row.get("source_uuid")),
                    audience: MessageAudience::new(row.get("audience_users")),
                    content: row.get("content"),
                    context: row.get("context"),
                }
            })
            .fetch_all(&mut *conn)
            .await?;

        Ok(all_messages)
    }

    pub(crate) async fn _query_messages_raw<T>(
        &self,
        conn: &mut T,
        query: MessageQueryRequest,
    ) -> anyhow::Result<QueryMessageResponse>
    where
            for<'e> &'e mut T: Executor<'e, Database = Postgres>,
    {
        let count_base_sql = r#"SELECT COUNT(*) as count FROM ("#;

        let base_sql = r#"
        SELECT
            messages.id,
            messages.issued as timestamp,
            messages.content,
            messages.context,
            sources.uuid as source_uuid,
            audiences.users as audience_users
        FROM messages
        INNER JOIN sources ON sources.id = messages.source_id
        INNER JOIN audiences ON audiences.id = messages.audience_id
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
                builder.push(" ORDER BY messages.id DESC ");

                count_builder.push(base_sql);
                count_builder.push(" WHERE ");
                query.filter.append_query(&mut count_builder);
                count_builder.push(" ORDER BY messages.id DESC LIMIT ");
                count_builder.push_bind(sessionless.row_limit.unwrap_or(7777777));
                count_builder.push(") as derivedQuery");

                query.limit.append_query(&mut builder);

                let count_messages = count_builder
                    .build()
                    .map(|row: PgRow| {
                        let count: i64 = row.get("count");
                        count
                    })
                    .fetch_one(&mut *conn)
                    .await?;

                let res = builder
                    .build()
                    .map(|row: PgRow| {
                        Message {
                            id: row.get("id"),
                            timestamp: row.get("timestamp"),
                            source: MessageSource::new(row.get("source_uuid")),
                            audience: MessageAudience::new(row.get("audience_users")),
                            content: row.get("content"),
                            context: row.get("context"),
                        }
                    })
                    .fetch_all(&mut *conn)
                    .await?;

                let mut cursor_map = HashMap::new();
                cursor_map.insert(1, Cursor::First);

                if let Some(last) = res.last() {
                    cursor_map.insert(2, Cursor::Other(
                        CursorData {
                            last_id: last.id,
                            last_issued: last.timestamp
                        }
                    ));
                }

                let total_pages = (count_messages as i32).div_ceil(sessionless.page_size);

                let session = SessionData {
                    session_id: Uuid::new_v4(),
                    issued: Utc::now().naive_local(),
                    page_size: sessionless.page_size,
                    page_number: 1,
                    pattern: sessionless.pattern,
                    cursors: cursor_map,
                    total_pages,
                    total_messages_count: count_messages as i32,
                };

                // Acquire write lock only at the end to minimize blocking time
                {
                    let mut write = self.session_cache.write().await;
                    write.insert(session.session_id, session.clone());
                }

                Ok(QueryMessageResponse {
                    session_key: session.session_id,
                    total_count: count_messages as i32,
                    current_page: 1,
                    total_pages,
                    messages: res,
                })
            }
            MessageQueryRequestKind::WithSession(with_session) => {
                let session: SessionData = {
                    // Read lock here avoids blocking concurrent requests!
                    let read = self.session_cache.read().await;
                    read.get(&with_session.session_id)
                        .context("session not present")?
                        .clone()
                };

                let mut query = session.pattern.clone();
                let page_number = with_session.page_number;
                let page_size = session.page_size;

                let mut builder = QueryBuilder::new(base_sql);

                if let Some(cursor) = session.cursors.get(&page_number.into()) {

                    let mut cursors_copied = session.cursors.clone();

                    match query.limit {
                        Limit::All => query.limit = Limit::Amount(page_size),
                        Limit::Amount(amount) => query.limit = Limit::Amount(min(page_size, amount)),
                    }

                    let new_filter: MessageFilterPattern;
                    let timestamp_filter = MessageFilter::InsertTimestamp(TimestampFilter::LessThanEqual(session.issued));

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

                    match cursor {
                        Cursor::First => {
                            builder.push(" WHERE ");
                            query.filter.append_query(&mut builder);
                            builder.push(" ORDER BY messages.id DESC ");
                        }
                        Cursor::Other(cursor_data) => {
                            builder.push(" WHERE ");

                            builder.push("messages.issued < ");
                            builder.push_bind(cursor_data.last_issued);
                            builder.push(" AND messages.id < ");
                            builder.push_bind(cursor_data.last_id);

                            builder.push(" AND ");
                            query.filter.append_query(&mut builder);
                            builder.push(" ORDER BY messages.id DESC ");
                        }
                    }

                    query.limit.append_query(&mut builder);

                    let res = builder
                        .build()
                        .map(|row: PgRow| {
                            Message {
                                id: row.get("id"),
                                timestamp: row.get("timestamp"),
                                source: MessageSource::new(row.get("source_uuid")),
                                audience: MessageAudience::new(row.get("audience_users")),
                                content: row.get("content"),
                                context: row.get("context"),
                            }
                        })
                        .fetch_all(&mut *conn)
                        .await?;

                    let next_page_data = res.last();
                    if !session.cursors.contains_key(&(page_number+1)) {
                        if let Some(next_page_cursor) = next_page_data {
                            cursors_copied.insert(page_number+1, Cursor::Other(
                                CursorData {
                                    last_issued: next_page_cursor.timestamp,
                                    last_id: next_page_cursor.id,
                                }
                            ));
                        }
                    }

                    let resp = QueryMessageResponse {
                        session_key: session.session_id,
                        total_count: session.total_messages_count,
                        current_page: page_number,
                        total_pages: session.total_pages,
                        messages: res,
                    };

                    {
                        let mut write = self.session_cache.write().await;
                        let updated_session = SessionData {
                            session_id: session.session_id,
                            issued: session.issued,
                            page_size,
                            page_number,
                            pattern: session.pattern.clone(),
                            cursors: cursors_copied,
                            total_pages: session.total_pages,
                            total_messages_count: session.total_messages_count,
                        };

                        write.insert(updated_session.session_id, updated_session);
                    }

                    return Ok(resp)

                } else {
                    let offset = (page_number - 2)*page_size;

                    builder.push(" WHERE ");
                    query.filter.append_query(&mut builder);

                    let limit = match query.limit {
                        Limit::All => Limit::Amount(page_size*2+1),
                        Limit::Amount(amount) => Limit::Amount(amount*2+1),
                    };

                    builder.push(" ORDER BY messages.id DESC ");
                    limit.append_query(&mut builder);

                    builder.push(" OFFSET ");
                    builder.push_bind(offset-1);

                    let all_messages = builder
                        .build()
                        .map(|row: PgRow| {
                            Message {
                                id: row.get("id"),
                                timestamp: row.get("timestamp"),
                                source: MessageSource::new(row.get("source_uuid")),
                                audience: MessageAudience::new(row.get("audience_users")),
                                content: row.get("content"),
                                context: row.get("context"),
                            }
                        })
                        .fetch_all(&mut *conn)
                        .await?;

                    let prev_page_page_data = all_messages.get(1);
                    let current_page_data = all_messages.get(page_size as usize);
                    let next_page_data = all_messages.last();

                    {
                        let mut cursors_copied = session.cursors.clone();
                        let mut write = self.session_cache.write().await;

                        if !session.cursors.contains_key(&(page_number+1)) {
                            if let Some(next_page_cursor) = next_page_data {
                                cursors_copied.insert(page_number+1, Cursor::Other(
                                    CursorData {
                                        last_issued: next_page_cursor.timestamp,
                                        last_id: next_page_cursor.id,
                                    }
                                ));
                            }
                        }
                        if !session.cursors.contains_key(&(page_number)) {
                            if let Some(current_page_cursor) = current_page_data {
                                cursors_copied.insert(page_number, Cursor::Other(
                                    CursorData {
                                        last_issued: current_page_cursor.timestamp,
                                        last_id: current_page_cursor.id,
                                    }
                                ));
                            };
                        }
                        if !session.cursors.contains_key(&(page_number-1)) {
                            if let Some(prev_page_cursor) = prev_page_page_data {
                                cursors_copied.insert(page_number-1, Cursor::Other(
                                    CursorData {
                                        last_issued: prev_page_cursor.timestamp,
                                        last_id: prev_page_cursor.id,
                                    }
                                ));
                            };
                        }

                        let updated_session = SessionData {
                            session_id: session.session_id,
                            issued: session.issued,
                            page_size,
                            page_number,
                            pattern: session.pattern.clone(),
                            cursors: cursors_copied,
                            total_pages: session.total_pages,
                            total_messages_count: session.total_messages_count
                        };

                        write.insert(updated_session.session_id, updated_session);
                    }

                    let mut res = all_messages.into_iter().rev().take(page_size as usize).collect::<Vec<_>>();
                    res.reverse();

                    let resp = QueryMessageResponse {
                        session_key: session.session_id,
                        total_count: session.total_messages_count,
                        current_page: page_number,
                        total_pages: session.total_pages,
                        messages: res,
                    };

                    return Ok(resp)
                }
            }
        }
    }

    pub(crate) async fn query_messages_raw(
        &self,
        query: MessageQueryRequest,
    ) -> anyhow::Result<QueryMessageResponse> {
        let mut txn: Transaction<Postgres> = self.pool.begin().await?;
        let res = self._query_messages_raw(&mut *txn, query).await;
        txn.commit().await?;
        res
    }
}