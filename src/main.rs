mod messages;
mod parse_torrent;
mod peers;
mod tracker;
use crate::download::Download;
use anyhow::Result;
use parse_torrent::parse_torrent;
use peers::ConnectionManager;
use rand::{distributions::Alphanumeric, Rng};
use std::env;
use tracker::request_tracker;
use futures::future;  
use tracing_subscriber;

pub mod download;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <torrent file>", args[0]);
        return Ok(());
    }
    tracing_subscriber::fmt::init();
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

    for peer in tracker_response.peers.into_iter() {
        connection_manager.add_peer(peer)?;
    }

    let tasks = connection_manager.handle_messages().await?;
    future::join_all(tasks).await;

    Ok(())
}
