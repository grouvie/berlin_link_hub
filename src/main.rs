use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::{subscriber::set_global_default, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Clone)]
struct ModelController;

impl ModelController {
    fn new() -> Self {
        Self
    }
    pub(crate) fn get_greeting(&self, username: &str) -> String {
        format!("Hello, {username}")
    }
}

#[derive(Deserialize, Debug)]
struct User {
    name: String,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    set_global_default(subscriber).expect("setting default subscriber failed");

    let mc = ModelController::new();

    let app = Router::new()
        .route("/", get(root))
        .route("/:username", get(hello_user))
        .route("/greet", post(greet_post))
        .route("/greet/:username", get(greet_user))
        .route("/delete/:id", delete(delete_user))
        .route("/status", get(status_code))
        .with_state(mc.clone())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn hello_user(Path(username): Path<String>) -> String {
    format!("Hello, {username}")
}

async fn greet_post(State(mc): State<ModelController>, payload: Json<User>) -> String {
    mc.get_greeting(&payload.name)
}

async fn greet_user(State(mc): State<ModelController>, Path(username): Path<String>) -> String {
    mc.get_greeting(&username)
}

async fn delete_user(Path(id): Path<String>) -> String {
    format!("Deleting user with id: {id}")
}

async fn status_code() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Did not find it.")
}
