mod logparser;
mod opts;
mod supplier;

use anyhow::Error;
use async_trait::async_trait;
use clap::Clap;
use futures::{select, FutureExt, StreamExt};
use logparser::{LogFormat, LogParser, LogRecord};
use meio::prelude::{LiteTask, ShutdownReceiver};
use opts::{Opts, SubCommand};
use rill::{
    pathfinder::{Pathfinder, Record},
    provider::LogProvider,
};
use supplier::Supplier;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let opts: Opts = Opts::parse();
    let name = opts.name.unwrap_or_else(|| "teleport".into());
    rill::install(name)?;
    match opts.subcmd {
        SubCommand::Stdin(stdin) => {
            let supplier = supplier::stdin();
            let task = LogTask::new(supplier, stdin.format.into());
        }
        SubCommand::File(file) => {
            let supplier = supplier::file(file.path);
            let task = LogTask::new(supplier, file.format.into());
        }
        SubCommand::Prometheus(prometheus) => {
            todo!();
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

struct LogTask<T: Supplier> {
    supplier: T,
    format: LogFormat,
}

impl<T: Supplier> LogTask<T> {
    fn new(supplier: T, format: LogFormat) -> Self {
        Self { supplier, format }
    }
}

#[async_trait]
impl<T: Supplier> LiteTask for LogTask<T> {
    async fn routine(mut self, signal: ShutdownReceiver) -> Result<(), Error> {
        let log_parser = LogParser::build(self.format)?;
        let mut providers: Pathfinder<LogProvider> = Pathfinder::new();
        let done = signal.just_done().fuse();
        tokio::pin!(done);
        let supplier = &mut self.supplier;
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
                _ = done => {
                    break;
                }
            }
        }
        Ok(())
    }
}
