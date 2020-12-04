use anyhow::Error;
use futures::{select, FutureExt, StreamExt};
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Error> {
    rill::install("teleport")?;
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines().fuse();
    loop {
        select! {
            line = lines.next() => {
                if let Some(line) = line {
                    // TODO: Send line by rill
                } else {
                    break;
                }
            }
        }
    }
    rill::terminate()?;
    Ok(())
}
