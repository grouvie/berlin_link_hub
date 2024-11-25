use axum::{extract::State, routing::get, Router};
use rinja::Template;

use crate::{database::uri::BasicURIRecord, error::AppResult, model::ModelController};

pub(crate) fn routes(mc: ModelController) -> Router {
    Router::new().route("/", get(home)).with_state(mc)
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    user_id: Option<usize>,
    uris: Vec<BasicURIRecord>,
}

async fn home(State(mc): State<ModelController>) -> AppResult<HomeTemplate> {
    let uris = mc.get_all_uris().await?;

    let home_template = HomeTemplate {
        user_id: None,
        uris,
    };
    Ok(home_template)
}
