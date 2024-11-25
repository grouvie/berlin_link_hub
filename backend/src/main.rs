use axum::{
    http::{Method, Uri},
    middleware,
    response::{IntoResponse, Redirect, Response},
    Router,
};
use ctx::Ctx;
use dotenv::dotenv;
use error::{AppError, SystemResult};
use log::log_request;
use model::ModelController;
use serde_json::json;
use tokio::net;
use tower_cookies::CookieManagerLayer;
use tracing::{subscriber::set_global_default, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;
use web::get_origin;

mod ctx;
mod database;
mod error;
mod log;
mod model;
mod web;

#[tokio::main]
async fn main() -> SystemResult<()> {
    dotenv().ok();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    set_global_default(subscriber).expect("setting default subscriber failed");

    let mc = ModelController::new().await?;

    mc.migrate().await;

    let routes_public = Router::new()
        .merge(web::routes_home::routes(mc.clone()))
        .merge(web::routes_auth::routes(mc.clone()))
        .merge(web::routes_user::routes(&mc.clone()));

    let routes_private = Router::new()
        .merge(web::routes_link::routes(mc.clone()))
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(routes_public)
        .merge(routes_private)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new());

    let listener = net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, routes_all).await?;

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    tracing::info!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<AppError>();
    let client_status_error = service_error.map(AppError::client_status_and_error);

    // -- If client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map(|(_status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            tracing::info!("    ->> client_error_body: {client_error_body}");

            let redirect_url = format!("{}/", get_origin());

            // Build the new response from the client_error_body
            Redirect::to(&redirect_url).into_response()
        });

    // Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error)
        .await
        .expect("Logging request failed");

    error_response.unwrap_or(res)
}
