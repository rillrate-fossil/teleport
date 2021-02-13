mod actors;
mod adapters;
mod docker_stats;
mod healthcheck;
mod loggers;
mod opts;

use actors::teleport::{Teleport, TeleportLink};
use anyhow::Error;
use clap::Clap;
use meio::prelude::System;
use opts::{Opts, SubCommand};
use reqwest::Url;
use rillrate::rill::prelude::Path;
use rillrate::RillRate;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let opts: Opts = Opts::parse();
    let rillrate = RillRate::from_env(&opts.name)?;
    let teleport = System::spawn(Teleport::new());
    let mut link: TeleportLink = teleport.link();
    match opts.subcmd {
        SubCommand::Stdin(params) => {
            use crate::loggers::{supplier, LogTask};
            let supplier = supplier::stdin();
            let task = LogTask::new(supplier, params.format.into());
            link.attach_task(task).await?;
        }
        SubCommand::File(params) => {
            use crate::loggers::{supplier, LogTask};
            let supplier = supplier::file(params.path);
            let task = LogTask::new(supplier, params.format.into());
            link.attach_task(task).await?;
        }
        SubCommand::Prometheus(params) => {
            use crate::adapters::prometheus::PrometheusTask;
            let url = Url::parse(&params.url)?;
            let interval = Duration::from_millis(params.interval);
            let task = PrometheusTask::new(url, interval);
            link.attach_task(task).await?;
        }
        SubCommand::Healthcheck(params) => {
            use crate::healthcheck::HealthcheckTask;
            let url = Url::parse(&params.url)?;
            let interval = Duration::from_millis(params.interval);
            let path = Path::single(params.name);
            let task = HealthcheckTask::new(path, url, interval);
            link.attach_task(task).await?;
        }
        SubCommand::DockerStats(params) => {
            use crate::docker_stats::DockerStatsTask;
            let path = Path::single(params.name);
            let task = DockerStatsTask::new(path);
            link.attach_task(task).await?;
        }
    }
    System::wait_or_interrupt(teleport).await?;
    drop(rillrate);

    // I have to use `exit` call here, because:
    //
    // Source: https://docs.rs/tokio/0.3.1/tokio/io/fn.stdin.html
    // > This handle is best used for non-interactive uses, such as when a file is piped
    // into the application. For technical reasons, stdin is implemented by using
    // an ordinary blocking read on a separate thread, and it is impossible to cancel
    // that read. This can make shutdown of the runtime hang until the user presses enter.
    //
    // Since we are here than any incoming data is useless. We can just exit the process.
    std::process::exit(0);
}
