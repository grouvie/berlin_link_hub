use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

mod auth;
pub(crate) mod uri;

use crate::error::SystemResult;

pub(crate) struct DatabaseClient {
    pub(crate) pool: Pool<Postgres>,
}

impl DatabaseClient {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

pub(crate) async fn create_pool() -> SystemResult<Pool<Postgres>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var not set");

    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?)
}
