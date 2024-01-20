mod parse_torrent;
mod tracker;
use parse_torrent::parse_torrent;
use tracker::request_tracker;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let torrent = parse_torrent("./data/centos-6.5.torrent");
    let res = request_tracker(&torrent).await?;
    Ok(())
}
