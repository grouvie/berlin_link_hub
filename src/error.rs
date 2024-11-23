use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::result;
use strum_macros::AsRefStr;

pub(crate) type AppResult<T> = result::Result<T, AppError>;

#[derive(Clone, Debug, Serialize, AsRefStr)]
#[allow(clippy::enum_variant_names, reason = "Error variants")]
pub(crate) enum AppError {
    // -- Auth errors
    AuthFailNoAuthTokenCookie,
    AuthFailTokenWrongFormat,
    AuthFailInvalidTimestamp,
    AuthFailExpiredTokenCookie,
    AuthFailCtxNotInRequestExt,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::info!("->> {:<12} - {self:?}", "INTO_RES");

        // Create a placeholder Axum response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response.
        response.extensions_mut().insert(self);

        response
    }
}

#[allow(unreachable_patterns, reason = "Example code")]
impl AppError {
    pub(crate) const fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        match self {
            // -- Auth.
            Self::AuthFailNoAuthTokenCookie
            | Self::AuthFailTokenWrongFormat
            | Self::AuthFailInvalidTimestamp
            | Self::AuthFailExpiredTokenCookie
            | Self::AuthFailCtxNotInRequestExt => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // -- Model.

            // -- Database.

            // -- Other.

            // -- Fallback.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, AsRefStr)]
#[allow(non_camel_case_types, reason = "For easier error recognition")]
pub(crate) enum ClientError {
    NO_AUTH,
    SERVICE_ERROR,
}
