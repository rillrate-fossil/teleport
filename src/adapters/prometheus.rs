use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::LiteTask;
use prometheus_parser::group_metrics as parse;
use reqwest::{Client, Url};
use rillrate::protocol::pathfinder::{Pathfinder, Record};
use rillrate::{EntryId, LogProvider, Path};
use std::time::{Duration, Instant};

pub struct PrometheusTask {
    client: Client,
    interval: Duration,
    url: Url,
    providers: Pathfinder<LogProvider>,
}

impl PrometheusTask {
    pub fn new(url: Url, interval: Duration) -> Self {
        let client = Client::new();
        Self {
            client,
            interval,
            url,
            providers: Pathfinder::new(),
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
            let provider = self.providers.find(&path).and_then(Record::get_link);
            if let Some(provider) = provider {
                if provider.is_active() {
                    let message = format!("{:?}", metric.metrics);
                    provider.log(message, None);
                }
            } else {
                log::debug!("Found metric: {}", metric.name);
                let provider = LogProvider::new(path.clone());
                self.providers.dig(path).set_link(provider);
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
