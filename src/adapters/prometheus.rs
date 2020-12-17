use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{LiteTask, StopReceiver};
use prometheus_parser::group_metrics as parse;
use reqwest::{Client, Url};
use rill::prelude::{EntryId, LogProvider, Path, Pathfinder, Record};
use tokio::time::{delay_for, Duration};

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
    async fn routine(mut self, mut stop: StopReceiver) -> Result<(), Error> {
        loop {
            if let Err(err) = stop.or(self.get_metrics()).await? {
                log::error!("Can't fetch metrics from {}: {}", self.url, err);
            }
            stop.or(delay_for(self.interval)).await?;
        }
    }
}
