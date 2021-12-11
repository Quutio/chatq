pub mod entity;
pub mod data;

use std::{sync::Arc, time::Duration};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::RwLock;

use entity::prelude::*;

pub mod chatq {
    tonic::include_proto!("chatq");
}

#[tonic::async_trait]
pub trait Db {

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
