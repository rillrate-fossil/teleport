use crate::actors::teleport::TeleportTask;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::LiteTask;
use rillrate::rill::prelude::{GaugeTracer, Path};
use shiplift::Docker;
use std::collections::HashMap;

struct TracingGroup {
    cpu: GaugeTracer,
    // TODO: Add logs here
}

impl TracingGroup {
    fn new(path: Path) -> Self {
        let cpu = GaugeTracer::new(path.concat("cpu"), false);
        Self { cpu }
    }
}

pub struct DockerStatsTask {
    docker: Docker,
    containers: HashMap<String, TracingGroup>,
    path: Path,
}

impl TeleportTask for DockerStatsTask {}

impl DockerStatsTask {
    pub fn new(path: Path) -> Self {
        Self {
            path,
            docker: Docker::new(),
            containers: HashMap::new(),
        }
    }

    async fn update_stats(&mut self) -> Result<(), Error> {
        let containers = self.docker.containers();
        let cont_list = containers.list(&Default::default()).await?;
        // TODO: Register logs provider, but keep it inactive, and
        // spawn the async routine if activated.
        // TODO: Attach tracer activate watcher for logs.
        for cont in cont_list {
            let stats = containers.get(&cont.id);
            let task_path = &self.path;
            self.containers.entry(cont.id).or_insert_with_key(|id| {
                let path = task_path.concat(id.as_ref());
                TracingGroup::new(path)
            });
        }
        Ok(())
    }
}

#[async_trait]
impl LiteTask for DockerStatsTask {
    type Output = ();

    async fn repeatable_routine(&mut self) -> Result<Option<Self::Output>, Error> {
        self.update_stats().await?;
        Ok(None)
    }
}
