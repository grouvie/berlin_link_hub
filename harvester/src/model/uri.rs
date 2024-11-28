use chrono::DateTime;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use url::Url;

use crate::{
    database::{uri::MeetupURIData, DatabaseClient},
    error::{SystemError, SystemResult},
    http::HttpClient,
};

use super::ModelController;

#[derive(Debug, Deserialize)]
pub(crate) struct MeetupUriUpdate {
    pub(crate) id: i32,
    pub(crate) timestamp: DateTime<chrono::Utc>,
}

pub(crate) struct InsertURIRecord {
    pub(crate) meetup_id: i32,
    pub(crate) url: String,
    pub(crate) url_scheme: String,
    pub(crate) url_host: String,
    pub(crate) url_path: Option<String>,
    pub(crate) status: Option<bool>,
    pub(crate) title: Option<String>,
    pub(crate) auto_description: Option<String>,
}

pub(crate) struct ParsedMeetupURIData {
    pub(crate) id: i32,
    pub(crate) uri_str: String,
    pub(crate) uri: Url,
}

impl TryFrom<MeetupURIData> for ParsedMeetupURIData {
    type Error = SystemError;

    fn try_from(data: MeetupURIData) -> SystemResult<Self, Self::Error> {
        let parsed_url = Url::parse(&data.uri)?;

        Ok(ParsedMeetupURIData {
            id: data.id,
            uri_str: data.uri,
            uri: parsed_url,
        })
    }
}

impl ModelController {
    pub(crate) async fn listen_to_meetup_uris_changes(
        &self,
        sender: Sender<MeetupUriUpdate>,
    ) -> SystemResult<()> {
        let client = DatabaseClient::new(self.pool.clone());

        client.listen_to_meetup_uris_changes(sender).await?;

        Ok(())
    }
    pub(crate) async fn process_new_uris(&self) -> SystemResult<()> {
        tracing::info!("Starting to process new URIs");

        let database_client = DatabaseClient::new(self.pool.clone());

        let meetup_uri_data = match database_client.get_meetup_uri_data().await {
            Ok(data) => {
                tracing::info!(count = data.len(), "Fetched URIs from the database");
                data
            }
            Err(error) => {
                tracing::error!(error = ?error, "Failed to fetch URIs from the database");
                return Err(error);
            }
        };

        let parsed_meetup_uri_data = match meetup_uri_data
            .into_iter()
            .map(TryInto::try_into)
            .collect::<SystemResult<Vec<ParsedMeetupURIData>>>()
        {
            Ok(data) => data,
            Err(error) => {
                tracing::error!(error = ?error, "Failed to parse meetup URI data");
                return Err(error);
            }
        };

        let http_client = HttpClient::new();

        let uri_records = match http_client
            .get_insert_links_from_links(parsed_meetup_uri_data)
            .await
        {
            Ok(records) => {
                tracing::info!(count = records.len(), "Fetched metadata for URIs");
                records
            }
            Err(error) => {
                tracing::error!(error = ?error, "Failed to fetch metadata for URIs");
                return Err(error);
            }
        };

        if let Err(error) = database_client.insert_uri_records(&uri_records).await {
            tracing::error!(error = ?error, "Failed to insert URI records into the database");
            return Err(error);
        }

        tracing::info!(
            count = uri_records.len(),
            "Successfully inserted new URI records"
        );

        Ok(())
    }
}
