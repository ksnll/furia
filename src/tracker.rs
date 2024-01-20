use anyhow::Result;
use percent_encoding::percent_encode_byte;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use url::Url;

use crate::parse_torrent::TorrentFile;

#[derive(Debug, Serialize, Deserialize)]
enum Event {
    Started,
    Stopped,
    Completed,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackerRequest {
    peer_id: String,
    port: u32,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: bool,
    no_peer_id: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Peer {
    #[serde(rename = "peer id")]
    peer_id: String,
    ip: String,
    port: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerResponse {
    #[serde(rename = "failure reason")]
    failure_reason: bool,
    #[serde(rename = "warning message")]
    warning_message: bool,
    /// Interval in seconds that the client should wait between sending regular requests to the tracker
    interval: u32,
    #[serde(rename = "tracker id")]
    tracker_id: String,
    complete: u32,
    incomplete: u32,
    peers: Vec<Peer>,
}

pub async fn request_tracker(torrent: &TorrentFile) -> Result<TrackerResponse> {
    let mut hasher = Sha1::new();
    let info_hash = serde_bencode::to_bytes(&torrent.info)?;
    hasher.update(info_hash);
    let info_hash = hasher.finalize();
    let info_hash: Vec<u8> = info_hash.as_slice().into();
    let info_hash = info_hash.into_iter().map(percent_encode_byte).collect::<String>();

    let tracker_request = TrackerRequest {
        peer_id: "peer_id".to_string(),
        port: 6881,
        uploaded: 0,
        downloaded: 0,
        left: 0,
        compact: true,
        no_peer_id: true,
    };
    let url = Url::parse(&torrent.announce)?;
    let url = url.join(&format!("?info_hash={}", &info_hash)).unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .query(&tracker_request)
        .send()
        .await?;
    let body = response.text().await?;
    dbg!(body);
    todo!();
}
