use anyhow::Error;
use async_trait::async_trait;
use futures::{select, FutureExt};
use meio::prelude::{LiteTask, ShutdownReceiver};
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
}

#[async_trait]
impl LiteTask for PrometheusTask {
    async fn routine(mut self, signal: ShutdownReceiver) -> Result<(), Error> {
        // TODO: DRY that below
        let done = signal.just_done().fuse();
        tokio::pin!(done);

        let duration = self.interval;
        loop {
            select! {
                _ = delay_for(duration).fuse() => {
                    // TODO: Repeat request
                }
                _ = done => {
                    break;
                }
            }
        }
        Ok(())
    }
}
