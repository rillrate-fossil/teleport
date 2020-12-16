use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{LiteTask, ShutdownReceiver};
use prometheus_parser::group_metrics as parse;
use reqwest::{Client, Url};
use tokio::time::{delay_for, Duration};

pub struct PrometheusTask {
    client: Client,
    interval: Duration,
    url: Url,
}

impl PrometheusTask {
    pub fn new(url: Url) -> Self {
        let client = Client::new();
        let interval = Duration::from_secs(1);
        Self {
            client,
            interval,
            url,
        }
    }

    async fn get_metrics(&self) -> Result<(), Error> {
        let text = self
            .client
            .get(self.url.clone())
            .send()
            .await?
            .text()
            .await?;
        let metrics = parse(&text)?;
        Ok(())
    }
}

#[async_trait]
impl LiteTask for PrometheusTask {
    async fn routine(mut self, mut signal: ShutdownReceiver) -> Result<(), Error> {
        loop {
            if let Err(err) = signal.or(self.get_metrics()).await? {
                log::error!("Can't fetch metrics from {}: {}", self.url, err);
            }
            signal.or(delay_for(self.interval)).await?;
        }
    }
}
