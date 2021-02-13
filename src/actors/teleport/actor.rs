use super::link;
use crate::adapters::prometheus::PrometheusTask;
use crate::docker_stats::DockerStatsTask;
use crate::healthcheck::HealthcheckTask;
use crate::loggers::{supplier, LogTask};
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, IdOf, InterruptedBy, LiteTask, StartedBy, System,
    TaskEliminated, TaskError,
};
use rillrate::rill::prelude::Path;

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

// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// TODO: REPLACE ALL THAT BELOW TO `link::AttachTask`
// and create tasks deparately in the `main` function

#[async_trait]
impl ActionHandler<link::BindStdin> for Teleport {
    async fn handle(&mut self, msg: link::BindStdin, ctx: &mut Context<Self>) -> Result<(), Error> {
        let supplier = supplier::stdin();
        let task = LogTask::new(supplier, msg.format.into());
        ctx.spawn_task(task, ());
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::BindFile> for Teleport {
    async fn handle(&mut self, msg: link::BindFile, ctx: &mut Context<Self>) -> Result<(), Error> {
        let supplier = supplier::file(msg.path);
        let task = LogTask::new(supplier, msg.format.into());
        ctx.spawn_task(task, ());
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::BindPrometheus> for Teleport {
    async fn handle(
        &mut self,
        msg: link::BindPrometheus,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let task = PrometheusTask::new(msg.url, msg.interval);
        ctx.spawn_task(task, ());
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::BindHealthcheck> for Teleport {
    async fn handle(
        &mut self,
        msg: link::BindHealthcheck,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = Path::single(msg.name);
        let task = HealthcheckTask::new(path, msg.url, msg.interval);
        ctx.spawn_task(task, ());
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::BindDockerStats> for Teleport {
    async fn handle(
        &mut self,
        msg: link::BindDockerStats,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = Path::single(msg.name);
        let task = DockerStatsTask::new(path);
        ctx.spawn_task(task, ());
        Ok(())
    }
}

#[async_trait]
impl<T: LiteTask> TaskEliminated<T> for Teleport {
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
