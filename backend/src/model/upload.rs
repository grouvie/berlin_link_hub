use std::str::FromStr;

use axum::extract::Multipart;
use chrono::NaiveDate;

use super::ModelController;
use crate::{
    database::{uri::InsertMeetupURIData, DatabaseClient},
    error::{AppError, AppResult},
};

#[derive(Debug)]
pub(crate) struct ParsedURIData {
    pub(crate) date: String,
    pub(crate) uri: String,
}

impl ModelController {
    pub(crate) async fn process_uri_csv_upload(
        &self,
        user_id: usize,
        multipart: Multipart,
    ) -> AppResult<()> {
        let parsed_uri_data = Self::parse_csv_to_uri_data(multipart).await?;

        let uris_to_insert = Self::convert_to_insert_meetup_uri_data(user_id, parsed_uri_data)?;

        let database_client = DatabaseClient::new(self.pool.clone());

        database_client.insert_uris(uris_to_insert).await?;

        Ok(())
    }

    fn convert_to_insert_meetup_uri_data(
        user_id: usize,
        parsed_uri_data: Vec<ParsedURIData>,
    ) -> AppResult<Vec<InsertMeetupURIData>> {
        let mut insert_uris = Vec::new();
        for uri_data in parsed_uri_data {
            let meetup_date =
                NaiveDate::from_str(&uri_data.date).map_err(|_error| AppError::ParseError)?;

            let user_id: i32 = user_id
                .try_into()
                .map_err(|_error| AppError::AuthFailInvalidId)?;

            let insert_uri = InsertMeetupURIData {
                meetup_date,
                uri: uri_data.uri,
                created_by: user_id,
            };
            insert_uris.push(insert_uri);
        }
        Ok(insert_uris)
    }
}
