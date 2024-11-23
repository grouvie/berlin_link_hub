use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};

use crate::model::ModelController;

pub(crate) fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/greet/:username", get(greet_user))
        .with_state(mc)
}

async fn greet_user(State(mc): State<ModelController>, Path(username): Path<String>) -> String {
    mc.get_greeting(&username)
}
