mod parse_torrent;
mod tracker;
mod peers;
use std::env;

use parse_torrent::parse_torrent;
use tracker::request_tracker;
use peers::ConnectionManager;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <torrent file>", args[0]);
        return Ok(());
    }
    let torrent = parse_torrent(&args[1]);
    let tracker_response = request_tracker(&torrent).await?;

    let mut connection_manager = ConnectionManager::new(&torrent);
    connection_manager.add_peer(tracker_response.peers[0].clone())?;
    connection_manager.connect_to_peers()?;

    Ok(())
}
