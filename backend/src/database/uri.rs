use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::error::{AppError, AppResult};

use super::DatabaseClient;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct InsertMeetupURIData {
    pub(crate) meetup_date: NaiveDate,
    pub(crate) uri: String,
    pub(crate) created_by: i32,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
#[allow(clippy::upper_case_acronyms, reason = "is an acronym")]
pub(crate) struct URI {
    pub(crate) id: i32,
    pub(crate) url: String,
    pub(crate) url_scheme: String,
    pub(crate) url_host: String,
    pub(crate) url_path: Option<String>,
    pub(crate) status: bool,
    pub(crate) title: Option<String>,
    pub(crate) auto_description: Option<String>,
    pub(crate) manual_description: Option<String>,
    pub(crate) created_by: Option<i32>,
    pub(crate) created_at: DateTime<chrono::Utc>,
    pub(crate) updated_by: Option<i32>,
    pub(crate) updated_at: Option<DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub(crate) struct BasicURIRecord {
    pub(crate) id: i32,
    pub(crate) url: String,
    pub(crate) url_scheme: String,
    pub(crate) url_host: String,
    pub(crate) url_path: Option<String>,
    pub(crate) status: bool,
    pub(crate) title: Option<String>,
    pub(crate) auto_description: Option<String>,
    pub(crate) manual_description: Option<String>,
}

impl DatabaseClient {
    pub(crate) async fn get_all_uri_records(&self) -> AppResult<Vec<BasicURIRecord>> {
        let statement = "
            SELECT
                ur.id AS id,
                ur.url,
                ur.url_scheme,
                ur.url_host,
                ur.url_path,
                ur.status,
                ur.title,
                ur.auto_description,
                ur.manual_description,
                mu.meetup_date,
                mu.created_at AS meetup_created_at
            FROM
                uri_records ur
            LEFT JOIN
                meetup_uris mu
            ON
                ur.meetup_id = mu.id
            ORDER BY
                mu.meetup_date ASC,
                ur.created_at ASC;
        ";

        let result = sqlx::query_as::<_, BasicURIRecord>(statement)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| AppError::Database {
                error: error.to_string(),
            })?;

        Ok(result)
    }
    pub(crate) async fn insert_uris(&self, insert_uris: Vec<InsertMeetupURIData>) -> AppResult<()> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| AppError::Database {
                error: error.to_string(),
            })?;

        let statement = "
            INSERT INTO public.meetup_uris (meetup_date, uri, created_by)
            SELECT * FROM UNNEST($1::timestamp[], $2::text[], $3::int[]);
        ";

        let meetup_dates: Vec<_> = insert_uris.iter().map(|data| data.meetup_date).collect();
        let uris: Vec<_> = insert_uris.iter().map(|data| data.uri.clone()).collect();
        let created_bys: Vec<_> = insert_uris.iter().map(|data| data.created_by).collect();

        sqlx::query(statement)
            .bind(&meetup_dates)
            .bind(&uris)
            .bind(&created_bys)
            .execute(&mut *transaction)
            .await
            .map_err(|error| AppError::Database {
                error: error.to_string(),
            })?;

        transaction
            .commit()
            .await
            .map_err(|error| AppError::Database {
                error: error.to_string(),
            })?;

        Ok(())
    }
}
