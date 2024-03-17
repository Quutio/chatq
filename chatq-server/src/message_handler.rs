use crate::ChatQDao;
use anyhow::Context;
use sqlx::postgres::PgConnectOptions;
use sqlx::{ConnectOptions, PgPool};
use std::str::FromStr;
use tracing::log::LevelFilter;

pub struct MessageHandler {
    pub db: ChatQDao,
}

impl MessageHandler {
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect_with(
            PgConnectOptions::from_str(addr)?.log_statements(LevelFilter::Info),
        )
        .await
        .context("database connect failure")?;

        let db = ChatQDao::with_pool(pool);

        Ok(Self { db })
    }
}
