use sqlx::{Pool, Postgres};

use crate::{database::create_pool, error::SystemResult};

mod auth;
mod csv;
pub(crate) mod upload;
mod uri;

#[derive(Clone)]
pub(crate) struct ModelController {
    pub(crate) pool: Pool<Postgres>,
}

impl ModelController {
    pub(crate) async fn new() -> SystemResult<Self> {
        let pool = create_pool().await?;
        Ok(Self { pool })
    }
    pub(crate) async fn migrate(&self) {
        match sqlx::migrate!().run(&self.pool).await {
            Ok(()) => tracing::info!("Successfully ran migrations"),
            Err(error) => tracing::error!("Running migrations failed: {error}"),
        }
    }
}
