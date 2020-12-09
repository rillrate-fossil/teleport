mod logparser;
mod opts;
mod supplier;

use anyhow::Error;
use clap::Clap;
use futures::{select, FutureExt, StreamExt};
use logparser::{LogFormat, LogParser, LogRecord};
use opts::{Opts, SubCommand};
use rill::{
    pathfinder::{Pathfinder, Record},
    provider::LogProvider,
};
use supplier::Supplier;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let opts: Opts = Opts::parse();
    let name = opts.name.unwrap_or_else(|| "teleport".into());
    rill::install(name)?;
    match opts.subcmd {
        SubCommand::Stdin(stdin) => {
            let supplier = supplier::stdin();
            // TODO: DRY
            if let Err(err) = routine(supplier, stdin.format.into()).await {
                log::error!("Failed: {}", err);
            }
        }
        SubCommand::File(file) => {
            let supplier = supplier::file(file.path);
            // TODO: DRY
            if let Err(err) = routine(supplier, file.format.into()).await {
                log::error!("Failed: {}", err);
            }
        }
    }
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

async fn routine(mut supplier: impl Supplier, format: LogFormat) -> Result<(), Error> {
    let log_parser = LogParser::build(format)?;
    let mut providers: Pathfinder<LogProvider> = Pathfinder::new();
    let ctrl_c = signal::ctrl_c().fuse();
    tokio::pin!(ctrl_c);
    loop {
        select! {
            line = supplier.next() => {
                if let Some(line) = line.transpose()? {
                    let res = log_parser.parse(&line);
                    match res {
                        Ok(LogRecord { path, timestamp, message }) => {
                            let provider = providers.find(&path).and_then(Record::get_link);
                            if let Some(provider) = provider {
                                if provider.is_active() {
                                    provider.log(timestamp, message);
                                }
                            } else {
                                let provider = LogProvider::new(path.clone());
                                providers.dig(path).set_link(provider);
                            }
                        }
                        Err(err) => {
                            log::error!("Can't parse line \"{}\": {}", line, err);
                        }
                    }
                } else {
                    break;
                }
            }
            _ = ctrl_c => {
                break;
            }
        }
    }
    Ok(())
}
