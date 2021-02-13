use super::parser::{LogFormat, LogParser, LogRecord};
use super::supplier::Supplier;
use crate::actors::teleport::TeleportTask;
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::prelude::LiteTask;
use rillrate::protocol::pathfinder::{Pathfinder, Record};
use rillrate::rill::prelude::LogTracer;

pub struct LogTask<T: Supplier> {
    supplier: T,
    format: LogFormat,
}

impl<T: Supplier> TeleportTask for LogTask<T> {}

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
        let mut tracers: Pathfinder<LogTracer> = Pathfinder::new();
        let supplier = &mut self.supplier;
        loop {
            if let Some(line) = supplier.next().await.transpose()? {
                let res = log_parser.parse(&line);
                match res {
                    Ok(LogRecord { path, message, .. }) => {
                        let tracer = tracers.find(&path).and_then(Record::get_link);
                        if let Some(tracer) = tracer {
                            if tracer.is_active() {
                                // TODO: Convert timestamp to `SystemTime`
                                tracer.log(message, None);
                            }
                        } else {
                            let tracer = LogTracer::new(path.clone(), false);
                            tracers.dig(path).set_link(tracer);
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
