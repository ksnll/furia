use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

#[derive(Debug, Deserialize, Serialize)]
struct Node(String, i64);

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub name: String,
    /// concatenation of each piece sha1 hash. Size multiple of 20 bytes
    pub pieces: ByteBuf,
    /// number of bytes per piece
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<File>>,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentFile {
    pub info: Info,
    #[serde(default)]
    pub announce: String,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

pub fn parse_torrent(file_path: &str) -> TorrentFile {
    let torrent_file = std::fs::read(file_path).expect("Unable to read file");
    serde_bencode::from_bytes(&torrent_file).expect("Unable to parse torrent file")
}

pub fn bitfield_size(torrent: &TorrentFile) -> u32 {
    let number_of_pieces = ((torrent.info.length.unwrap() + torrent.info.piece_length - 1)
        / torrent.info.piece_length) as usize;
    dbg!(number_of_pieces);
    dbg!(((number_of_pieces + 7) / 8) as u32);
    ((number_of_pieces + 7) / 8) as u32

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_a_torrent_file() {
        let torrent = parse_torrent("./data/ubuntu-22.04.3-live-server-amd64.iso.torrent");
        assert_eq!(
            "https://torrent.ubuntu.com/announce",
            torrent.announce
        );
        assert_eq!(Some(1691692385), torrent.creation_date);
        assert_eq!("ubuntu-22.04.3-live-server-amd64.iso", torrent.info.name);
        assert_eq!(262144, torrent.info.piece_length);
    }
}
