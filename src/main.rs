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
use rillrate::RillRate;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let opts: Opts = Opts::parse();
    let rillrate = RillRate::from_env(&opts.name)?;
    let teleport = System::spawn(Teleport::new());
    let mut link: TeleportLink = teleport.link();
    match opts.subcmd {
        SubCommand::Stdin(params) => {
            link.bind_stdin(params.format.into()).await?;
        }
        SubCommand::File(params) => {
            link.bind_file(params.path, params.format.into()).await?;
        }
        SubCommand::Prometheus(params) => {
            link.bind_prometheus(&params.url, params.interval).await?;
        }
        SubCommand::Healthcheck(params) => {
            link.bind_healthcheck(&params.name, &params.url, params.interval)
                .await?;
        }
        SubCommand::DockerStats(params) => {
            link.bind_docker_stats(&params.name).await?;
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
