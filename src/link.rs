use crate::actor::Teleport;
use crate::loggers::LogFormat;
use anyhow::Error;
use derive_more::{Deref, DerefMut, From};
use meio::prelude::{Action, Address};
use std::path::{Path, PathBuf};

#[derive(Debug, From, Deref, DerefMut)]
pub struct TeleportLink {
    address: Address<Teleport>,
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
