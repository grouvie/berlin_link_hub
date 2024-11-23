use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};

use crate::{ctx::Ctx, error::AppResult, model::ModelController};

pub(crate) fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/links/:username", get(get_links))
        .with_state(mc)
}

async fn get_links(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Path(username): Path<String>,
) -> AppResult<String> {
    Ok(mc.get_links(&username, ctx.user_id()))
}
