use sqlx::PgPool;
use lib::ChatQDao;

#[sqlx::test(migrations = "./migrations")]
async fn first(pool: PgPool) {

    let chatq = ChatQDao::with_pool(pool);
}