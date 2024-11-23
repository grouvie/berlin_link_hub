use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    ctx::Ctx,
    error::{AppError, AppResult, ClientError},
};

pub(crate) async fn log_request(
    uuid: Uuid,
    req_method: Method,
    uri: Uri,
    context: Option<Ctx>,
    service_error: Option<&AppError>,
    client_error: Option<ClientError>,
) -> AppResult<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Calculating duration since unix_epoch failed")
        .as_millis();

    let error_type = service_error.map(|se| se.as_ref().to_owned());
    let error_data = serde_json::to_value(service_error)
        .ok()
        .and_then(|mut value| value.get_mut("data").map(Value::take));

    // Create the RequestLogLine
    let log_line = RequestLogLine {
        uuid: uuid.to_string(),
        timestamp: timestamp.to_string(),

        req_path: uri.to_string(),
        req_method: req_method.to_string(),

        user_id: context.map(|ctx| ctx.user_id()),

        client_error_type: client_error.map(|error| error.as_ref().to_owned()),

        error_type,
        error_data,
    };

    tracing::info!("   ->> log_request: \n{}", json!(log_line));

    // TODO - Send to some logging service.

    Ok(())
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
    uuid: String,      // uuid string formatted
    timestamp: String, // (should be iso8601)

    // -- User and context attributes.
    user_id: Option<usize>,

    // -- http request attributes.
    req_path: String,
    req_method: String,

    // -- Errors attributes.
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,
}
