use crate::actors::teleport::TeleportTask;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::LiteTask;
use reqwest::{Client, Url};
use rillrate::rill::prelude::{GaugeTracer, LogTracer, Path};
use std::time::{Duration, Instant};

pub struct HealthcheckTask {
    client: Client,
    interval: Duration,
    url: Url,
    roundtrip: GaugeTracer,
    status: LogTracer,
}

impl TeleportTask for HealthcheckTask {}

impl HealthcheckTask {
    pub fn new(path: Path, url: Url, interval: Duration) -> Self {
        let client = Client::new();
        let roundtrip = GaugeTracer::new(path.concat("roundtrip"), false);
        let status = LogTracer::new(path.concat("status"), false);
        Self {
            client,
            interval,
            url,
            roundtrip,
            status,
        }
    }

    async fn check_endpoint(&mut self) -> Result<(), Error> {
        let when = Instant::now();
        let response = self.client.get(self.url.clone()).send().await?;
        let status = response.status().to_string();
        let _text = response.text().await?;
        let elapsed = when.elapsed().as_millis() as f64 / 1_000f64;
        self.roundtrip.set(elapsed, None);
        self.status.log(status, None);
        Ok(())
    }
}

#[async_trait]
impl LiteTask for HealthcheckTask {
    type Output = ();

    async fn repeatable_routine(&mut self) -> Result<Option<Self::Output>, Error> {
        self.check_endpoint().await?;
        Ok(None)
    }

    fn retry_delay(&self, _last_attempt: Instant) -> Duration {
        self.interval
    }
}
