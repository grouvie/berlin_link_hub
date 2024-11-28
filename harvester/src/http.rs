use std::time::Duration;

use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    RequestBuilder,
};
use scraper::{Html, Selector};
use tokio::time::sleep;
use uri::PageMetadata;
use url::Url;

use crate::error::{SystemError, SystemResult};

mod uri;

#[derive(Clone)]
pub(crate) struct HttpClient {
    client: reqwest::Client,
    headers: HeaderMap,
}

impl HttpClient {
    pub(crate) fn new() -> Self {
        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0",
            ),
        );

        Self { client, headers }
    }
    pub(crate) fn builder(&self, url: Url) -> RequestBuilder {
        self.client.get(url).headers(self.headers.clone())
    }
    pub(crate) async fn get(builder: RequestBuilder) -> SystemResult<Option<PageMetadata>> {
        const MAX_RETRIES: usize = 3;
        const RETRY_DELAY_MS: u64 = 500;

        let mut attempt = 0;

        loop {
            match builder
                .try_clone()
                .ok_or(SystemError::RetryError)?
                .send()
                .await
            {
                Ok(response) => {
                    let body = response.text().await?;
                    let html = Html::parse_document(&body);
                    return Self::parse_html(&html);
                }
                Err(error) if attempt < MAX_RETRIES => {
                    tracing::warn!(
                        error = ?error,
                        attempt,
                        "Request failed, retrying..."
                    );
                    attempt += 1;
                    sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                }
                Err(error) => {
                    tracing::error!(error = ?error, "Request failed after retries");
                    return Err(SystemError::RequestError(error.to_string()));
                }
            }
        }
    }

    fn parse_html(html: &Html) -> SystemResult<Option<PageMetadata>> {
        let title_selector = parse_selector("title")?;
        let og_title_selector = parse_selector("meta[property='og:title']")?;
        let og_description_selector = parse_selector("meta[property='og:description']")?;
        let description_selector = parse_selector("meta[name='description']")?;

        let title = html
            .select(&title_selector)
            .next()
            .and_then(|element| element.text().next())
            .map(String::from);

        let og_title = extract_content(&og_title_selector, html);
        let og_description = extract_content(&og_description_selector, html);
        let description = extract_content(&description_selector, html);

        if og_title.is_none()
            && title.is_none()
            && og_description.is_none()
            && description.is_none()
        {
            return Ok(None);
        }

        Ok(Some(PageMetadata {
            title: og_title.or(title),                   // Prefer og:title if available
            description: og_description.or(description), // Prefer og:description if available
        }))
    }
}

fn parse_selector(selector_str: &str) -> Result<Selector, SystemError> {
    Selector::parse(selector_str).map_err(|error| SystemError::Selector(error.to_string()))
}

fn extract_content(selector: &Selector, html: &Html) -> Option<String> {
    html.select(selector)
        .next()
        .and_then(|element| element.value().attr("content"))
        .map(String::from)
}
