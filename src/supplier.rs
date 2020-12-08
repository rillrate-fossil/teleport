use anyhow::Error;
use async_trait::async_trait;
use futures::{stream::Fuse, StreamExt};
use tokio::io::{self, AsyncBufReadExt, BufReader, Lines, Stdin};

#[async_trait]
pub trait Supplier {
    async fn next_line(&mut self) -> Result<Option<String>, Error>;
}

pub struct StdinSupplier {
    stdin: Fuse<Lines<BufReader<Stdin>>>,
}

impl StdinSupplier {
    pub fn new() -> Self {
        let stdin = BufReader::new(io::stdin()).lines().fuse();
        Self { stdin }
    }
}

#[async_trait]
impl Supplier for StdinSupplier {
    async fn next_line(&mut self) -> Result<Option<String>, Error> {
        self.stdin.next().await.transpose().map_err(Error::from)
    }
}
