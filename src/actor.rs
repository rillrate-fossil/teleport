use crate::log_task::LogTask;
use crate::{link, supplier};
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Consumer, Context, Eliminated, IdOf, InterruptedBy, LiteTask, StartedBy,
    System, Task,
};
use meio::signal::CtrlC;

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
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.address().attach(CtrlC::stream());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for Teleport {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl Consumer<CtrlC> for Teleport {
    async fn handle(&mut self, _: CtrlC, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Ctrl-C signal. Terminating...");
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::BindStdin> for Teleport {
    async fn handle(&mut self, msg: link::BindStdin, ctx: &mut Context<Self>) -> Result<(), Error> {
        let supplier = supplier::stdin();
        let task = LogTask::new(supplier, msg.format.into());
        ctx.bind_task(task, ());
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::BindFile> for Teleport {
    async fn handle(&mut self, msg: link::BindFile, ctx: &mut Context<Self>) -> Result<(), Error> {
        let supplier = supplier::file(msg.path);
        let task = LogTask::new(supplier, msg.format.into());
        ctx.bind_task(task, ());
        Ok(())
    }
}

#[async_trait]
impl<T: LiteTask> Eliminated<Task<T>> for Teleport {
    async fn handle(&mut self, _id: IdOf<Task<T>>, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
