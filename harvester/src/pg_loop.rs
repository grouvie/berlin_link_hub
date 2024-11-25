use tokio::sync::mpsc::channel;

use crate::{
    controller_loop::{ControllerHandle, ToController},
    error::SystemResult,
    model::{uri::MeetupUriUpdate, ModelController},
};

struct PgListener {
    controller_handle: ControllerHandle,
    mc: ModelController,
}

impl PgListener {
    fn new(server_handle: ControllerHandle, mc: ModelController) -> Self {
        Self {
            controller_handle: server_handle,
            mc,
        }
    }
    async fn handle_message(&mut self, msg: MeetupUriUpdate) -> SystemResult<()> {
        tracing::info!("{msg:#?}");
        let to_server = ToController::Update(msg);
        self.controller_handle.send(to_server).await;
        Ok(())
    }
}

async fn run_listener(mut pg_bot_actor: PgListener) -> SystemResult<()> {
    let (sender, mut receiver) = channel(64);

    pg_bot_actor
        .mc
        .listen_to_meetup_uris_changes(sender)
        .await?;

    while let Some(msg) = receiver.recv().await {
        pg_bot_actor.handle_message(msg).await?;
    }
    Ok(())
}

pub(crate) struct PgListenerHandle;

impl PgListenerHandle {
    pub(crate) fn spawn(server_handle: ControllerHandle, mc: ModelController) {
        let actor = PgListener::new(server_handle, mc);

        tokio::spawn(async move {
            if let Err(error) = run_listener(actor).await {
                tracing::error!("{error}");
            }
        });
    }
}
