use async_stream::try_stream;
use futures::{
    stream::{Fuse, FusedStream},
    task::{Context, Poll},
    Stream, StreamExt,
};
use pin_project::pin_project;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncSeekExt, BufReader, Error, Lines, SeekFrom, Stdin};

pub trait Supplier: Stream<Item = Result<String, Error>> + FusedStream + Unpin {}

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

impl Supplier for StdinSupplier {}

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

pub struct FileSupplier {}

impl FileSupplier {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {}
    }
}

fn read_file_to_end(path: PathBuf) -> impl Stream<Item = Result<String, Error>> {
    try_stream! {
        let mut position = 0;
        let mut file = File::open(&path).await?;
        let total = file.seek(SeekFrom::End(0)).await?;
        if position < total {
            file.seek(SeekFrom::Start(position)).await?;
        } else {
            log::warn!("Log file completely changes (size reset). Reading from the start.");
            file.seek(SeekFrom::Start(0)).await?;
        }
        {
            let mut lines = BufReader::new(&mut file).lines();
            while let Some(line) = lines.next().await.transpose()? {
                yield line;
            }
        }
        position = total;
        // TODO: Await for the inotify
    }
}
