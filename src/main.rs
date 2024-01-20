mod parse_torrent;
mod tracker;
use std::env;

use parse_torrent::parse_torrent;
use tracker::request_tracker;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <torrent file>", args[0]);
        return Ok(());
    }
    let torrent = parse_torrent(&args[1]);
    let res = request_tracker(&torrent).await?;
    dbg!(&res);
    Ok(())
}
