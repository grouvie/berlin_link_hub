use sqlx::{Pool, Postgres};

use crate::{database::create_pool, error::SystemResult};

pub(crate) mod uri;

#[derive(Clone)]
pub(crate) struct ModelController {
    pub(crate) pool: Pool<Postgres>,
}

impl ModelController {
    pub(crate) async fn new() -> SystemResult<Self> {
        let pool = create_pool().await?;
        Ok(Self { pool })
    }
}
