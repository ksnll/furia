mod parse_torrent;
mod tracker;
mod peers;
mod messages;
use crate::download::Download;
use std::env;
use parse_torrent::parse_torrent;
use rand::{distributions::Alphanumeric, Rng};
use tracker::request_tracker;
use peers::ConnectionManager;
use anyhow::Result;

pub mod download;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <torrent file>", args[0]);
        return Ok(());
    }
    let torrent = parse_torrent(&args[1]);

    let peer_id = format!(
            "-FU0001-{}",
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(12)
                .map(char::from)
                .collect::<String>()
        );
    let tracker_response = request_tracker(&torrent, &peer_id).await?;
    let download = Download::from(&torrent);

    let mut connection_manager = ConnectionManager::new(torrent, download, &peer_id).await;

    for peer in tracker_response.peers.into_iter().take(5) {
        connection_manager.add_peer(peer)?;
    }
    // connection_manager.connect_to_peers()?;
    let tasks = connection_manager.handle_messages().await?;
    for task in tasks {
        task.await?;
    }

    Ok(())
}
