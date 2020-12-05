mod logparser;
mod opts;

use anyhow::Error;
use clap::Clap;
use futures::{select, FutureExt, StreamExt};
use logparser::{LogParser, LogRecord};
use opts::Opts;
use rill::{
    pathfinder::{Pathfinder, Record},
    provider::LogProvider,
};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let opts: Opts = Opts::parse();
    let name = opts.name.unwrap_or_else(|| "teleport".into());
    rill::install(name)?;
    if let Err(err) = routine().await {
        log::error!("Failed: {}", err);
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

async fn routine() -> Result<(), Error> {
    let log_parser = LogParser::build(PATTERN, "::")?;
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines().fuse();
    let mut providers: Pathfinder<LogProvider> = Pathfinder::new();
    let ctrl_c = signal::ctrl_c().fuse();
    tokio::pin!(ctrl_c);
    loop {
        select! {
            line = lines.next() => {
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
                            log::error!("Can't parse line: {}", err);
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

static PATTERN: &str = r"^\[(?P<ts>\S+) (?P<lvl>\S+) (?P<path>\S+)\] (?P<msg>.+)$";
