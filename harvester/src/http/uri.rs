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
        //let semaphore = Arc::new(Semaphore::new(50));
        let mut tasks = Vec::new();

        for entry in data {
            //let permit = semaphore.clone().acquire_owned().await.unwrap();

            let builder = self.builder(entry.uri.clone());

            tasks.push(tokio::spawn(async move {
                //drop(permit);
                Ok::<_, SystemError>((entry, HttpClient::get(builder).await?))
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
                _ => None,
            })
            .collect();

        Ok(insert_links)
    }
}
