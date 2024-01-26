mod messages;
mod parse_torrent;
mod peers;
mod tracker;
use crate::download::Download;
use anyhow::Result;
use parse_torrent::parse_torrent;
use peers::ConnectionManager;
use std::env;
use tracker::request_tracker;

pub mod download;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <torrent file>", args[0]);
        return Ok(());
    }
    let torrent = parse_torrent(&args[1]);
    let tracker_response = request_tracker(&torrent).await?;
    let download = Download::from(&torrent);

    let mut connection_manager = ConnectionManager::new(torrent, download);

    for peer in tracker_response.peers.into_iter().take(5) {
        connection_manager.add_peer(peer)?;
    }
    // connection_manager.connect_to_peers()?;
    connection_manager.handle_messages().await?;

    Ok(())
}
