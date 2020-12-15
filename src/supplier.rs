use anyhow::Error;
use async_stream::try_stream;
use futures::{stream::FusedStream, Stream, StreamExt};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::sync::watch;

pub trait Supplier:
    Stream<Item = Result<String, Error>> + FusedStream + Unpin + Send + 'static
{
}

impl<T> Supplier for T where
    T: Stream<Item = Result<String, Error>> + FusedStream + Unpin + Send + 'static
{
}

pub fn stdin() -> impl Supplier {
    watch_stdin().boxed().fuse()
}

fn watch_stdin() -> impl Stream<Item = Result<String, Error>> {
    try_stream! {
        let mut stdin = BufReader::new(io::stdin()).lines();
        while let Some(line) = stdin.next().await.transpose()? {
            yield line;
        }
    }
}

pub fn file(path: impl AsRef<Path>) -> impl Supplier {
    let path = path.as_ref().to_path_buf();
    watch_file(path).boxed().fuse()
}

fn watch_file(path: PathBuf) -> impl Stream<Item = Result<String, Error>> {
    try_stream! {
        let (tx, mut rx) = watch::channel(None);
        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| {
            if let Err(err) = tx.send(Some(res)) {
                log::error!("Can't send inotify signal: {}", err);
            }
        })?;
        watcher.watch(&path, RecursiveMode::NonRecursive)?;
        let mut position = 0;
        loop {
            {
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
            }
            rx.changed().await?;
        }
    }
}
