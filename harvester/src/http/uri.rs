use futures::future;

use crate::model::uri::{InsertURIRecord, ParsedMeetupURIData};

#[derive(Debug)]
pub(crate) struct PageMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
}

use super::{HttpClient, SystemError, SystemResult};

impl HttpClient {
    pub(crate) async fn get_insert_links_from_links(
        self,
        data: Vec<ParsedMeetupURIData>,
    ) -> SystemResult<Vec<InsertURIRecord>> {
        tracing::debug!(count = data.len(), "Starting to fetch metadata for URIs");

        let mut tasks = Vec::new();

        for entry in data {
            let uri = entry.uri.clone();
            let builder = self.builder(uri.clone());

            tasks.push(tokio::spawn(async move {
                let result = HttpClient::get(builder).await;
                match result {
                    Ok(metadata) => {
                        tracing::debug!(uri = %uri, "Successfully fetched metadata for URI");
                        Ok::<_, SystemError>((entry, metadata))
                    }
                    Err(error) => {
                        tracing::error!(error = ?error, uri = %uri, "Failed to fetch metadata for URI");
                        Ok::<_, SystemError>((entry, None))
                    }
                }
            }));
        }

        let results = future::join_all(tasks).await;

        let insert_links = results
            .into_iter()
            .filter_map(|task_result| match task_result {
                Ok(Ok((link, page_metadata))) => Some(InsertURIRecord {
                    meetup_id: link.id,
                    url: link.uri_str,
                    url_scheme: link.uri.scheme().to_owned(),
                    url_host: link.uri.host_str().unwrap_or("-").to_owned(),
                    url_path: Some(link.uri.path().to_owned()),
                    status: Some(true),
                    title: page_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.title.clone()),
                    auto_description: page_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.description.clone()),
                }),
                Ok(Err(system_error)) => {
                    tracing::error!(error = ?system_error, "Failed to fetch page metadata");
                    None
                }
                Err(join_error) => {
                    tracing::error!(error = ?join_error, "Task failed to complete");
                    None
                }
            })
            .collect::<Vec<_>>();

        tracing::info!(count = insert_links.len(), "Finished processing URIs");

        Ok(insert_links)
    }
}
