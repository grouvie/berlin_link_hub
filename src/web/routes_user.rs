use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};

use crate::{
    //ctx::Ctx,
    error::AppResult,
    model::ModelController,
};

pub(crate) fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/greet/:username", get(greet_user))
        .with_state(mc)
}

async fn greet_user(
    State(mc): State<ModelController>,
    /* accessing ctx requires auth */
    // ctx: Ctx,
    Path(username): Path<String>,
) -> AppResult<String> {
    Ok(mc.get_greeting(&username))
}
