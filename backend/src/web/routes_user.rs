use axum::Router;

use crate::model::ModelController;

pub(crate) fn routes(_mc: &ModelController) -> Router {
    Router::new()
}
