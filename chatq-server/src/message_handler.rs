use crate::ChatQDao;
use anyhow::Context;
use sqlx::PgPool;

pub struct MessageHandler {
    pub db: ChatQDao,
}

impl MessageHandler {
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(addr)
            .await
            .context("database connect failure")?;

        let db = ChatQDao::with_pool(pool);

        Ok(Self { db })
    }
}
