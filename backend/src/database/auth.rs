use crate::error::{AppError, AppResult};

use super::DatabaseClient;

impl DatabaseClient {
    pub(crate) async fn get_auth_data(&self, mail_address: String) -> AppResult<(i32, String)> {
        let statement = "
            SELECT id, password FROM users WHERE email = $1;
        ";

        sqlx::query_as::<_, (i32, String)>(statement)
            .bind(mail_address)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| AppError::Database {
                error: error.to_string(),
            })
    }
}
