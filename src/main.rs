mod logparser;

use anyhow::Error;
use futures::{select, StreamExt};
use logparser::LogParser;
use rill::{pathfinder::Pathfinder, Provider};
use tokio::io::{self, AsyncBufReadExt, BufReader};

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
    //let providers = Pathfinder::new();
    let log_provider = Provider::new("stderr".into());
    loop {
        select! {
            line = lines.next() => {
                if let Some(line) = line.transpose()? {
                    let res = log_parser.parse(&line);
                    match res {
                        Ok(record) => {
                            log_provider.send(record.data);
                        }
                        Err(err) => {
                            log::error!("Can't parse line: {}", err);
                        }
                    }
                } else {
                    break;
                }
            }
            // TODO: Add CtrlC
        }
    }
    Ok(())
}

static PATTERN: &str = r"^\[(?P<ts>\S+) (?P<lvl>\S+) (?P<path>\S+)\] (?P<msg>.+)$";
