use anyhow::Result;
use percent_encoding::percent_encode_byte;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use url::Url;

use crate::parse_torrent::{Info, TorrentFile};

#[derive(Debug, Serialize, Deserialize)]
enum Event {
    Started,
    Stopped,
    Completed,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackerRequest {
    peer_id: String,
    port: isize,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: bool,
    no_peer_id: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Peer {
    #[serde(rename = "peer id")]
    pub peer_id: Option<String>,
    pub ip: String,
    pub port: i64,
}

#[derive(Debug, Deserialize)]
pub struct TrackerResponse {
    #[serde(rename = "failure reason")]
    failure_reason: Option<bool>,
    #[serde(rename = "warning message")]
    warning_message: Option<bool>,
    /// Interval in seconds that the client should wait between sending regular requests to the tracker
    interval: u32,
    #[serde(rename = "tracker id")]
    tracker_id: Option<String>,
    complete: Option<u32>,
    incomplete: Option<u32>,
    #[serde(with = "peer_list")]
    pub peers: Vec<Peer>,
}

mod peer_list {
    use super::Peer;
    use serde::{Deserialize, Deserializer};
    use serde_bytes::{ByteArray, ByteBuf};
    use tracing::info;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Peer>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: ByteBuf = Deserialize::deserialize(deserializer)?;

        let mut peers = Vec::new();
        for chunk in bytes.chunks(6) {
            if chunk.len() == 6 {
                let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);
                let port = ((chunk[4] as i64) << 8) | chunk[5] as i64;
                peers.push(Peer {
                    peer_id: None,
                    ip,
                    port,
                });
            }
        }
        info!("Found {} peers", peers.len());
        Ok(peers)
    }
}
pub fn get_info_hash(info: &Info) -> Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    let info_hash = serde_bencode::to_bytes(info)?;
    hasher.update(info_hash);
    let info_hash = hasher.finalize();
    let info_hash = info_hash.as_slice().try_into().unwrap();
    Ok(info_hash)
}

pub fn get_encoded_info_hash(info: &Info) -> Result<String> {
    let info_hash = get_info_hash(&info)?; // Vec<u8>
    let info_hash = info_hash
        .into_iter()
        .map(percent_encode_byte)
        .collect::<String>();
    Ok(info_hash)
}

pub async fn request_tracker(torrent: &TorrentFile, peer_id: &str) -> Result<TrackerResponse> {
    let info_hash = get_encoded_info_hash(&torrent.info)?;

    let tracker_request = TrackerRequest {
        peer_id: peer_id.to_owned(),
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
    let response = client.get(url).query(&tracker_request).send().await?;
    let body = response.bytes().await?;
    let response: TrackerResponse = serde_bencode::from_bytes::<TrackerResponse>(&body)?;
    Ok(response)
}

#[cfg(test)]
mod test {
    use super::get_encoded_info_hash;
    use crate::parse_torrent::Info;
    use serde_bytes::ByteBuf;

    #[test]
    fn calculate_info_hash() {
        let info = Info {
            name: "test".to_string(),
            pieces: ByteBuf::from(vec![0; 20]),
            piece_length: 20,
            md5sum: None,
            length: None,
            private: None,
            path: None,
            root_hash: None,
            files: None,
        };
        let info_hash = get_encoded_info_hash(&info);
        assert_eq!(
            info_hash.unwrap(),
            "%D3%FA%63%53%76%EC%A2%AF%67%04%85%08%03%09%59%2A%47%63%2B%66"
        );
    }
}
