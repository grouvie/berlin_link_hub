use chrono::NaiveDate;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgListener, prelude::FromRow};
use tokio::sync::mpsc::Sender;

use crate::model::uri::{InsertURIRecord, MeetupUriUpdate};

use super::{DatabaseClient, SystemResult};

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub(crate) struct MeetupURIData {
    pub(crate) id: i32,
    pub(crate) meetup_date: NaiveDate,
    pub(crate) uri: String,
}

impl DatabaseClient {
    pub(crate) async fn listen_to_meetup_uris_changes(
        &self,
        sender: Sender<MeetupUriUpdate>,
    ) -> SystemResult<()> {
        let mut listener = PgListener::connect_with(&self.pool).await?;

        listener.listen("meetup_uris_change").await?;

        let mut stream = listener.into_stream();
        tokio::spawn(async move {
            while let Ok(Some(notification)) = stream.try_next().await {
                let payload = notification.payload();
                match serde_json::from_str::<MeetupUriUpdate>(payload) {
                    Ok(status_update) => {
                        if let Err(error) = sender.send(status_update).await {
                            tracing::error!("Sending failed: {error}");
                        }
                    }
                    Err(error) => {
                        tracing::error!("Failed to parse JSON notification: {error}");
                    }
                }
            }
        });

        Ok(())
    }
    pub(crate) async fn get_meetup_uri_data(&self) -> SystemResult<Vec<MeetupURIData>> {
        let statement = "
            SELECT id, meetup_date, uri 
            FROM public.meetup_uris 
            WHERE handled = false;
        ";

        Ok(sqlx::query_as::<_, MeetupURIData>(statement)
            .fetch_all(&self.pool)
            .await?)
    }
    pub(crate) async fn insert_uri_records(
        &self,
        records: Vec<InsertURIRecord>,
    ) -> SystemResult<()> {
        let mut transaction = self.pool.begin().await?;

        let statement = "
            INSERT INTO public.uri_records (
                meetup_id, url, url_scheme, url_host, url_path, status, title, auto_description
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8);
        ";

        for record in records {
            sqlx::query(statement)
                .bind(record.meetup_id)
                .bind(record.url)
                .bind(record.url_scheme)
                .bind(record.url_host)
                .bind(record.url_path)
                .bind(record.status.unwrap_or(false))
                .bind(record.title)
                .bind(record.auto_description)
                .execute(&mut *transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}
