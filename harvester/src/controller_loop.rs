use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    error::SystemResult,
    model::{uri::MeetupUriUpdate, ModelController},
};

#[derive(Debug)]
pub(crate) enum ToController {
    Update(MeetupUriUpdate),
}

struct Controller {
    receiver: Receiver<ToController>,
    mc: ModelController,
}

impl Controller {
    fn new(receiver: Receiver<ToController>, mc: ModelController) -> Self {
        Self { receiver, mc }
    }
    async fn handle_message(&self, msg: ToController) -> SystemResult<()> {
        let ToController::Update(meetup_uri_update) = msg;
        tracing::info!(
            "id: {}, timestamp: {}",
            meetup_uri_update.id,
            meetup_uri_update.timestamp
        );
        self.mc.process_new_uris().await?;

        Ok(())
    }
}

async fn run_controller(mut controller_actor: Controller) -> SystemResult<()> {
    while let Some(msg) = controller_actor.receiver.recv().await {
        controller_actor.handle_message(msg).await?;
    }
    Ok(())
}

pub(crate) struct ControllerHandle {
    pub(crate) sender: Sender<ToController>,
}

impl ControllerHandle {
    pub(crate) fn new(mc: ModelController) -> (Self, JoinHandle<()>) {
        let (sender, receiver) = channel(64);

        let join = tokio::spawn(async move {
            let actor = Controller::new(receiver, mc);
            if let Err(error) = run_controller(actor).await {
                tracing::error!("{error}");
            };
        });

        (Self { sender }, join)
    }
    pub(crate) async fn send(&mut self, msg: ToController) {
        assert!(
            self.sender.send(msg).await.is_ok(),
            "Controller loop has shut down"
        );
    }
}
