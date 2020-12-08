use futures::{
    stream::{Fuse, FusedStream},
    task::{Context, Poll},
    Stream, StreamExt,
};
use pin_project::pin_project;
use std::pin::Pin;
use tokio::io::{self, AsyncBufReadExt, BufReader, Error, Lines, Stdin};

pub trait Supplier: Stream<Item = Result<String, Error>> {}

#[pin_project]
pub struct StdinSupplier {
    #[pin]
    stdin: Fuse<Lines<BufReader<Stdin>>>,
}

impl StdinSupplier {
    pub fn new() -> Self {
        let stdin = BufReader::new(io::stdin()).lines().fuse();
        Self { stdin }
    }
}

impl Stream for StdinSupplier {
    type Item = Result<String, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.stdin.poll_next(cx)
    }
}

impl FusedStream for StdinSupplier {
    fn is_terminated(&self) -> bool {
        self.stdin.is_terminated()
    }
}
