mod logparser;

use anyhow::Error;
use futures::{select, FutureExt, StreamExt};
use logparser::{LogParser, LogRecord};
use rill::{
    pathfinder::{Pathfinder, Record},
    Provider,
};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    rill::install("teleport")?;
    if let Err(err) = routine().await {
        log::error!("Failed: {}", err);
    }
    rill::terminate()?;
    Ok(())
}

async fn routine() -> Result<(), Error> {
    let log_parser = LogParser::build(PATTERN, "::")?;
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines().fuse();
    let mut providers: Pathfinder<Provider> = Pathfinder::new();
    let ctrl_c = signal::ctrl_c().fuse();
    tokio::pin!(ctrl_c);
    loop {
        select! {
            line = lines.next() => {
                if let Some(line) = line.transpose()? {
                    let res = log_parser.parse(&line);
                    match res {
                        Ok(LogRecord { path, data }) => {
                            let provider = providers.find(&path).and_then(Record::get_link);
                            if let Some(provider) = provider {
                                if provider.is_active() {
                                    provider.send(data);
                                }
                            } else {
                                let provider = Provider::new(path.clone());
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
