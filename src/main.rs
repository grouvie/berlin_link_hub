use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing::{subscriber::set_global_default, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    set_global_default(subscriber).expect("setting default subscriber failed");

    let app = Router::new()
        .route("/", get(root))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
