use crate::{
    database::{uri::BasicURIRecord, DatabaseClient},
    error::AppResult,
};

use super::ModelController;

impl ModelController {
    pub(crate) async fn get_all_uris(&self) -> AppResult<Vec<BasicURIRecord>> {
        let database_client = DatabaseClient::new(self.pool.clone());

        database_client.get_all_uri_records().await
    }
}
