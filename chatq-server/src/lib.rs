pub mod data;
pub mod entity;

use std::{sync::Arc, time::Duration};

use sea_orm::{ConnectOptions, Database, DatabaseConnection, Unset};
use tokio::sync::RwLock;

use entity::prelude::*;

use data::models::*;
use tonic::async_trait;

pub mod chatq {
    tonic::include_proto!("chatq");
}

#[async_trait]
pub trait Db {
    async fn insert_message(stub: MessageStub) -> anyhow::Result<Message>;
}

pub struct ChatQDb {
    db: DatabaseConnection,
}

impl ChatQDb {
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        let mut opt = ConnectOptions::new(addr.to_owned());
        opt.max_connections(100)
            .min_connections(5)
            .idle_timeout(Duration::from_secs(8))
            .sqlx_logging(true);

        let db = Database::connect(opt).await?;

        Ok(ChatQDb { db })
    }
}

#[async_trait]
impl Db for ChatQDb {
    async fn insert_message(stub: MessageStub) -> anyhow::Result<Message> {

        let message = entity::messages::ActiveModel {
            ts: stub.timestamp
        };
    }
}
