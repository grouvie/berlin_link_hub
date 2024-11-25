use axum::{
    extract::{Multipart, State},
    response::Redirect,
    routing::{get, post},
    Router,
};
use rinja::Template;

use crate::{ctx::Ctx, database::uri::BasicURIRecord, error::AppResult, model::ModelController};

use super::get_origin;

pub(crate) fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/links", get(links))
        .route("/upload", get(upload))
        .route("/api/upload", post(api_upload))
        .with_state(mc)
}

#[derive(Template)]
#[template(path = "preview.html")]
struct PreviewTemplate {
    user_id: Option<usize>,
    uris: Vec<BasicURIRecord>,
}

async fn links(State(mc): State<ModelController>, ctx: Ctx) -> AppResult<PreviewTemplate> {
    let user_id = ctx.user_id();

    let uris = mc.get_all_uris().await?;

    let preview_template = PreviewTemplate {
        user_id: Some(user_id),
        uris,
    };

    Ok(preview_template)
}

#[derive(Template)]
#[template(path = "upload.html")]
struct UploadTemplate {
    user_id: Option<usize>,
}

async fn upload(ctx: Ctx) -> AppResult<UploadTemplate> {
    let user_id = ctx.user_id();

    let upload_template = UploadTemplate {
        user_id: Some(user_id),
    };

    Ok(upload_template)
}

async fn api_upload(
    State(mc): State<ModelController>,
    ctx: Ctx,
    multipart: Multipart,
) -> AppResult<Redirect> {
    let user_id = ctx.user_id();

    mc.process_uri_csv_upload(user_id, multipart).await?;

    Ok(Redirect::to(&format!("{}/links", get_origin())))
}
