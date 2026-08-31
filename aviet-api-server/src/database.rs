use sqlx::PgPool;
use anyhow::Result;

pub async fn init_db(database_url: &str) -> Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;

    Ok(pool)
}
