pub mod data;

use std::collections::HashMap;
use sqlx::{Executor, PgPool, Row};
use std::error::Error;
use std::fmt::format;

use anyhow::Context;
use chrono::NaiveDateTime;
use sqlx::postgres::PgRow;
use uuid::Uuid;

use data::models::*;
use crate::chatq::message_audience::Audience;
use crate::data::models::MessageFilter::Source;

pub mod chatq {
    tonic::include_proto!("chatq");
}

pub struct ChatQDao {
    pub pool: PgPool,
}

impl ChatQDao {
    pub fn with_pool(pool: PgPool) -> Self {
        ChatQDao { pool }
    }

    pub async fn insert_message(&self, stub: MessageStub) -> anyhow::Result<Message> {
        let mut txn = self.pool.begin().await?;

        let message_id = sqlx::query!(
            r#"
INSERT INTO messages (issued,content)
VALUES ($1,$2)
ON CONFLICT DO NOTHING
RETURNING message_id
            "#,
            stub.timestamp,
            stub.content,
        )
        .fetch_one(&mut txn)
        .await
        .context("insert message database failure")?
        .message_id;

        match &stub.audience {
            MessageAudience::Players(players) => {
                for player in players {
                    sqlx::query!(
                        r#"
INSERT INTO audiences_player (player,message_id)
VALUES ($1,$2)
ON CONFLICT DO NOTHING
RETURNING id
                    "#,
                        sqlx::types::Uuid::from_bytes(player.as_bytes().clone()),
                        message_id
                    )
                    .fetch_one(&mut txn)
                    .await?;
                }
            }
            MessageAudience::Servers(servers) => {
                for server in servers {
                    sqlx::query!(
                        r#"
INSERT INTO audiences_server (server,message_id)
VALUES ($1,$2)
ON CONFLICT DO NOTHING
RETURNING id
                    "#,
                        server.value,
                        message_id
                    )
                    .fetch_one(&mut txn)
                    .await?;
                }
            }
        }

        match &stub.source {
            MessageSource::Players(players) => {
                for player in players {
                    sqlx::query!(
                        r#"
INSERT INTO sources_player (player,message_id)
VALUES ($1,$2)
ON CONFLICT DO NOTHING
RETURNING id
                    "#,
                        sqlx::types::Uuid::from_bytes(player.as_bytes().clone()),
                        message_id
                    )
                    .fetch_one(&mut txn)
                    .await?;
                }
            }
            MessageSource::Plugins(plugins) => {
                for plugin in plugins {
                    sqlx::query!(
                        r#"
INSERT INTO sources_plugin (plugin,message_id)
VALUES ($1,$2)
ON CONFLICT DO NOTHING
RETURNING id
                    "#,
                        plugin.value,
                        message_id
                    )
                    .fetch_one(&mut txn)
                    .await?;
                }
            }
        }

        txn.commit().await?;

        Ok(Message::from_stub(message_id, stub))
    }

    pub async fn query_messages(&self, filter: &MessageFilterPattern) -> anyhow::Result<Vec<Message>> {
        let mut txn = self.pool.begin().await?;

        let restriction = filter.to_string();

        let foo = sqlx::query(&format!(
            r#"
            SELECT messages.*
            FROM messages
            JOIN audiences_server ON audiences_server.message_id = messages.message_id
            JOIN audiences_player ON audiences_player.message_id = messages.message_id
            JOIN sources_plugin ON sources_plugin.message_id = messages.message_id
            JOIN sources_player ON sources_player.message_id = messages.message_id
            WHERE {}
            "#, restriction
        )).map(|row: PgRow| {
            let message_id: i64 = row.get("message_id");
            let issued: NaiveDateTime = row.get("issued");
            let content: String = row.get("content");

            return (message_id, issued, content);
        }).fetch_all(&mut txn).await?;

        let mut a_server = HashMap::new();
        let mut a_player = HashMap::new();
        let mut s_plugin = HashMap::new();
        let mut s_player = HashMap::new();

        for f in &foo {
            let bar = sqlx::query!(
                r#"
                SELECT * FROM audiences_server WHERE message_id = $1
                "#, f.0
            ).fetch_one(&mut txn).await?;

            a_server.entry(f.0).or_insert(Vec::new()).push(
                (
                    bar.message_id,
                    bar.server
                    )
            )
        }

        for f in &foo {
            let bar = sqlx::query!(
                r#"
                SELECT * FROM audiences_player WHERE message_id = $1
                "#, f.0
            ).fetch_one(&mut txn).await?;
            a_player.entry(f.0).or_insert(Vec::new()).push(
                (
                    bar.message_id,
                    bar.player
                )
            )
        }

        for f in &foo {
            let bar = sqlx::query!(
                r#"
                SELECT * FROM sources_plugin WHERE message_id = $1
                "#, f.0
            ).fetch_one(&mut txn).await?;
            s_plugin.entry(f.0).or_insert(Vec::new()).push(
                (
                    bar.message_id,
                    bar.plugin
                )
            )
        }

        for f in &foo {
            let bar = sqlx::query!(
                r#"
                SELECT * FROM sources_player WHERE message_id = $1
                "#, f.0
            ).fetch_one(&mut txn).await?;
            s_player.entry(f.0).or_insert(Vec::new()).push(
                (
                    bar.message_id,
                    bar.player
                )
            )
        }

        let mut res: Vec<Message> = Vec::new();

        for f in &foo {
            let mut a_server = a_server.get(&f.0);
            let mut a_player = a_player.get(&f.0);
            let mut s_plugin = s_plugin.get(&f.0);
            let mut s_player = s_player.get(&f.0);

            if a_server.is_some() && s_plugin.is_some() {
                let aud = MessageAudience::Servers(a_server.unwrap().iter().map(|op| Server { value: op.1.clone()}).collect());
                let src = MessageSource::Plugins(s_plugin.unwrap().iter().map(|op| Plugin { value: op.1.clone()}).collect());

                res.push(Message {
                    id: f.0,
                    timestamp: f.1,
                    source: src,
                    audience: aud,
                    content: f.2.to_string(),
                })
            } else if a_server.is_some() && s_player.is_some() {
                let aud = MessageAudience::Servers(a_server.unwrap().iter().map(|op| Server { value: op.1.clone()}).collect());
                let src = MessageSource::Players(s_player.unwrap().iter().map(|op| Uuid::from_bytes(op.1.as_bytes().clone())).collect());

                res.push(Message {
                    id: f.0,
                    timestamp: f.1,
                    source: src,
                    audience: aud,
                    content: f.2.to_string(),
                })
            } else if a_player.is_some() && s_plugin.is_some() {
                let aud = MessageAudience::Players(a_player.unwrap().iter().map(|op| Uuid::from_bytes(op.1.as_bytes().clone())).collect());
                let src = MessageSource::Plugins(s_plugin.unwrap().iter().map(|op| Plugin { value: op.1.clone()}).collect());

                res.push(Message {
                    id: f.0,
                    timestamp: f.1,
                    source: src,
                    audience: aud,
                    content: f.2.to_string(),
                })
            } else if a_player.is_some() && s_player.is_some() {
                let aud = MessageAudience::Players(a_player.unwrap().iter().map(|op| Uuid::from_bytes(op.1.as_bytes().clone())).collect());
                let src = MessageSource::Players(s_player.unwrap().iter().map(|op| Uuid::from_bytes(op.1.as_bytes().clone())).collect());

                res.push(Message {
                    id: f.0,
                    timestamp: f.1,
                    source: src,
                    audience: aud,
                    content: f.2.to_string(),
                })
            }
        }

        Ok(res)
    }
}
