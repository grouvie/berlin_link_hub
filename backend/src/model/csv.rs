use std::collections::HashMap;

use axum::extract::Multipart;

use crate::error::{AppError, AppResult};

use super::{upload::ParsedURIData, ModelController};

impl ModelController {
    pub(crate) fn detect_delimiter(csv_content: &str) -> char {
        let common_delimiters = [',', ';', '\t', '|'];
        let mut delimiter_counts = HashMap::new();

        for line in csv_content.lines().take(5) {
            for &delimiter in &common_delimiters {
                let count = line.matches(delimiter).count();
                *delimiter_counts.entry(delimiter).or_insert(0) += count;
            }
        }

        delimiter_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map_or(',', |(delimiter, _)| delimiter)
    }
    pub(crate) async fn parse_csv_to_uri_data(
        mut multipart: Multipart,
    ) -> AppResult<Vec<ParsedURIData>> {
        let mut links = Vec::new();

        if let Some(csv_field) = multipart
            .next_field()
            .await
            .map_err(|_error| AppError::MultipartError)?
            .filter(|field| field.name() == Some("csv"))
        {
            let file_name = csv_field
                .file_name()
                .map_or("Unknown".to_owned(), ToOwned::to_owned);

            let csv_content = csv_field
                .text()
                .await
                .map_err(|_error| AppError::MultipartError)?;

            tracing::info!("Uploaded file: {file_name}");

            let delimiter = Self::detect_delimiter(&csv_content);
            tracing::info!("Detected delimiter: '{delimiter}'");

            links = csv_content
                .lines()
                .filter_map(|line| {
                    let mut parts = line.split(delimiter);
                    let date = parts.next()?.trim().to_owned();
                    let uri = parts.next()?.trim().to_owned();

                    Some(ParsedURIData { date, uri })
                })
                .collect::<Vec<ParsedURIData>>();

            tracing::info!("Parsed CSV data with {} links", links.len());
        }
        Ok(links)
    }
}
