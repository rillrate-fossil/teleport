use super::link;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, IdOf, InterruptedBy, StartedBy, System, TaskEliminated,
    TaskError,
};

pub struct Teleport {}

impl Teleport {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for Teleport {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<System> for Teleport {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for Teleport {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Ctrl-C signal. Terminating...");
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: link::TeleportTask> ActionHandler<link::AttachTask<T>> for Teleport {
    async fn handle(
        &mut self,
        msg: link::AttachTask<T>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.spawn_task(msg.task, ());
        Ok(())
    }
}

#[async_trait]
impl<T: link::TeleportTask> TaskEliminated<T> for Teleport {
    async fn handle(
        &mut self,
        _id: IdOf<T>,
        _result: Result<T::Output, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
