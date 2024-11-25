use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use chrono::Utc;
use rinja::Template;
use serde::Deserialize;
use tower_cookies::Cookies;

use crate::{
    error::{AppError, AppResult},
    model::ModelController,
    web::{get_origin, mw_auth::remove_private_cookie},
};

use super::mw_auth::set_private_cookie;

pub(crate) fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/api/login", post(api_login))
        .route("/logout", get(api_logout))
        .with_state(mc)
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    user_id: Option<usize>,
}

async fn login() -> AppResult<LoginTemplate> {
    let login_template = LoginTemplate { user_id: None };
    Ok(login_template)
}

#[derive(Deserialize)]
pub(crate) struct LoginData {
    pub(crate) email: String,
    pub(crate) password: String,
}

async fn api_login(
    State(mc): State<ModelController>,
    cookies: Cookies,
    Form(login_data): Form<LoginData>,
) -> AppResult<impl IntoResponse> {
    match mc.login(login_data).await {
        Ok(user_id) => {
            let timestamp = Utc::now().timestamp();

            let token = format!("user-{user_id}.{timestamp}");

            set_private_cookie(&cookies, token);

            let redirect_url = format!("{}/links", get_origin());

            let mut headers = HeaderMap::new();
            headers.insert(
                "HX-Redirect",
                redirect_url.parse().expect("Parsing redirect_url failed"),
            );

            Ok((StatusCode::OK, headers, "Redirecting...").into_response())
        }
        Err(error) => match error {
            AppError::Database { .. } | AppError::AuthFailVerifyPasswordFail => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    "HX-Trigger",
                    r#"{"showError": {"message": "Invalid email or password."}}"#
                        .parse()
                        .expect("Parsing HeaderValue failed"),
                );
                Ok((
                    StatusCode::BAD_REQUEST,
                    headers,
                    "Invalid credentials.".to_owned(),
                )
                    .into_response())
            }
            _ => Err(error),
        },
    }
}

async fn api_logout(cookies: Cookies) -> Redirect {
    tracing::info!("->> {:<12} - logout", "HANDLER");
    remove_private_cookie(&cookies);

    Redirect::to(get_origin())
}
