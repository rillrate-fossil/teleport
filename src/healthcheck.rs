use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::LiteTask;
use reqwest::{Client, Url};
use rillrate::Gauge;
use std::time::{Duration, Instant};

pub struct HealthcheckTask {
    client: Client,
    interval: Duration,
    url: Url,
    roundtrip: Gauge,
}

impl HealthcheckTask {
    pub fn new(url: Url, interval: Duration) -> Self {
        let client = Client::new();
        Self {
            client,
            interval,
            url,
            roundtrip: Gauge::create("endpoint.roundtrip").unwrap(),
        }
    }

    async fn check_endpoint(&mut self) -> Result<(), Error> {
        let when = Instant::now();
        let _text = self
            .client
            .get(self.url.clone())
            .send()
            .await?
            .text()
            .await?;
        let elapsed = when.elapsed().as_millis() as f64 / 1_000f64;
        self.roundtrip.set(elapsed);
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
