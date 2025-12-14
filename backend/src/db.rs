use sqlx::{MySql, Pool, pool::PoolOptions};
use std::time::Duration;

pub type MySqlPoolOptions = PoolOptions<MySql>;
pub type DbPool = Pool<MySql>;

pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
}
