mod actor;
mod link;
mod loggers;
mod opts;

use actor::Teleport;
use anyhow::Error;
use clap::Clap;
use link::TeleportLink;
use meio::prelude::Link;
use opts::{Opts, SubCommand};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let opts: Opts = Opts::parse();
    let name = opts.name.unwrap_or_else(|| "teleport".into());
    rill::install(name)?;
    let teleport = Teleport::new();
    let mut link: TeleportLink = meio::spawn(teleport).link();
    match opts.subcmd {
        SubCommand::Stdin(stdin) => {
            link.bind_stdin(stdin.format.into()).await?;
        }
        SubCommand::File(file) => {
            link.bind_file(file.path, file.format.into()).await?;
        }
        SubCommand::Prometheus(prometheus) => {
            todo!();
        }
    }
    link.interrupt()?;
    link.join().await;
    rill::terminate()?;

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
