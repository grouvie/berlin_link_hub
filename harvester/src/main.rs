use dotenv::dotenv;
use error::SystemResult;
use pg_loop::PgListenerHandle;
use tracing::subscriber;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use controller_loop::ControllerHandle;
use model::ModelController;

mod controller_loop;
mod database;
mod error;
mod http;
mod model;
mod pg_loop;

#[tokio::main]
async fn main() -> SystemResult<()> {
    dotenv().ok();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mc = ModelController::new().await?;

    let (controller_handle, controller_join) = ControllerHandle::new(mc.clone());

    PgListenerHandle::spawn(controller_handle, mc);

    controller_join.await?;

    Ok(())
}
