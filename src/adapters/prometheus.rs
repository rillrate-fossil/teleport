use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::LiteTask;
use prometheus_parser::group_metrics as parse;
use reqwest::{Client, Url};
use rillrate::protocol::pathfinder::{Pathfinder, Record};
use rillrate::protocol::provider::{EntryId, Path};
use rillrate::rill::prelude::LogTracer;
use std::time::{Duration, Instant};

pub struct PrometheusTask {
    client: Client,
    interval: Duration,
    url: Url,
    tracers: Pathfinder<LogTracer>,
}

impl PrometheusTask {
    pub fn new(url: Url, interval: Duration) -> Self {
        let client = Client::new();
        Self {
            client,
            interval,
            url,
            tracers: Pathfinder::new(),
        }
    }

    async fn get_metrics(&mut self) -> Result<(), Error> {
        let text = self
            .client
            .get(self.url.clone())
            .send()
            .await?
            .text()
            .await?;
        let metrics = parse(&text)?;
        for metric in metrics {
            let entries: Vec<_> = metric.name.split("_").map(EntryId::from).collect();
            let path = Path::from(entries);
            let tracer = self.tracers.find(&path).and_then(Record::get_link);
            if let Some(tracer) = tracer {
                if tracer.is_active() {
                    let message = format!("{:?}", metric.metrics);
                    tracer.log(message, None);
                }
            } else {
                log::debug!("Found metric: {}", metric.name);
                let tracer = LogTracer::new(path.clone(), false);
                self.tracers.dig(path).set_link(tracer);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl LiteTask for PrometheusTask {
    type Output = ();

    async fn repeatable_routine(&mut self) -> Result<(), Error> {
        self.get_metrics().await
        //log::error!("Can't fetch metrics from {}: {}", self.url, err);
    }

    fn retry_delay(&self, _last_attempt: Instant) -> Duration {
        self.interval
    }
}
