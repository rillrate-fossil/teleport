use super::Teleport;
use anyhow::Error;
use derive_more::{Deref, DerefMut, From};
use meio::prelude::{Action, Address, LiteTask};

#[derive(Debug, From, Deref, DerefMut)]
pub struct TeleportLink {
    address: Address<Teleport>,
}

pub trait TeleportTask: LiteTask {
    // TODO: Add description here and some meta like parameters or name
}

pub(super) struct AttachTask<T: TeleportTask> {
    pub task: T,
}

impl<T: TeleportTask> Action for AttachTask<T> {}

impl TeleportLink {
    pub async fn attach_task<T: TeleportTask>(&mut self, task: T) -> Result<(), Error> {
        let msg = AttachTask { task };
        self.address.act(msg).await
    }
}
