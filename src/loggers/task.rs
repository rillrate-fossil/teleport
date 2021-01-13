use super::parser::{LogFormat, LogParser, LogRecord};
use super::supplier::Supplier;
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::prelude::LiteTask;
use rillrate::protocol::pathfinder::{Pathfinder, Record};
use rillrate::LogProvider;

pub struct LogTask<T: Supplier> {
    supplier: T,
    format: LogFormat,
}

impl<T: Supplier> LogTask<T> {
    pub fn new(supplier: T, format: LogFormat) -> Self {
        Self { supplier, format }
    }
}

#[async_trait]
impl<T: Supplier> LiteTask for LogTask<T> {
    type Output = ();

    // TODO: Change to??? interruptable routine.
    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        let log_parser = LogParser::build(self.format)?;
        let mut providers: Pathfinder<LogProvider> = Pathfinder::new();
        let supplier = &mut self.supplier;
        loop {
            if let Some(line) = supplier.next().await.transpose()? {
                let res = log_parser.parse(&line);
                match res {
                    Ok(LogRecord { path, message, .. }) => {
                        let provider = providers.find(&path).and_then(Record::get_link);
                        if let Some(provider) = provider {
                            if provider.is_active() {
                                // TODO: Convert timestamp to `SystemTime`
                                provider.log(message, None);
                            }
                        } else {
                            let provider = LogProvider::new(path.clone());
                            providers.dig(path).set_link(provider);
                        }
                    }
                    Err(err) => {
                        // Skipping the line
                        log::error!("Can't parse line \"{}\": {}", line, err);
                    }
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}
