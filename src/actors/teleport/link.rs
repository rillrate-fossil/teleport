use super::Teleport;
use crate::loggers::LogFormat;
use anyhow::Error;
use derive_more::{Deref, DerefMut, From};
use meio::prelude::{Action, Address, LiteTask};
use reqwest::Url;
use std::path::{Path, PathBuf};
use tokio::time::Duration;

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

pub(super) struct BindStdin {
    pub format: LogFormat,
}

impl Action for BindStdin {}

impl TeleportLink {
    pub async fn bind_stdin(&mut self, format: LogFormat) -> Result<(), Error> {
        let msg = BindStdin { format };
        self.address.act(msg).await
    }
}

pub(super) struct BindFile {
    pub path: PathBuf,
    pub format: LogFormat,
}

impl Action for BindFile {}

impl TeleportLink {
    pub async fn bind_file(
        &mut self,
        path: impl AsRef<Path>,
        format: LogFormat,
    ) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf();
        let msg = BindFile { path, format };
        self.address.act(msg).await
    }
}

pub(super) struct BindPrometheus {
    pub url: Url,
    pub interval: Duration,
}

impl Action for BindPrometheus {}

impl TeleportLink {
    pub async fn bind_prometheus(&mut self, url: &str, interval_ms: u64) -> Result<(), Error> {
        let url = Url::parse(url)?;
        let interval = Duration::from_millis(interval_ms);
        let msg = BindPrometheus { url, interval };
        self.address.act(msg).await
    }
}

pub(super) struct BindHealthcheck {
    pub name: String,
    pub url: Url,
    pub interval: Duration,
}

impl Action for BindHealthcheck {}

impl TeleportLink {
    pub async fn bind_healthcheck(
        &mut self,
        name: &str,
        url: &str,
        interval_ms: u64,
    ) -> Result<(), Error> {
        let name = name.into();
        let url = Url::parse(url)?;
        let interval = Duration::from_millis(interval_ms);
        let msg = BindHealthcheck {
            name,
            url,
            interval,
        };
        self.address.act(msg).await
    }
}

pub(super) struct BindDockerStats {
    pub name: String,
}

impl Action for BindDockerStats {}

impl TeleportLink {
    pub async fn bind_docker_stats(&mut self, name: &str) -> Result<(), Error> {
        let name = name.into();
        let msg = BindDockerStats { name };
        self.address.act(msg).await
    }
}
