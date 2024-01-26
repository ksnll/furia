use crate::parse_torrent::TorrentFile;

#[derive(Debug, Clone)]
pub enum PieceStatus {
    NotStarted,
    Downloading,
    Downloaded,
    ShaVerified,
    WrittenToDisk,
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub content: Option<Vec<u8>>,
    pub status: PieceStatus,
    pub original_sha1: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Download {
    pub pieces: Vec<Piece>,
}

impl Download {
    pub fn from(torrent: &TorrentFile) -> Self {
        Self {
            pieces: torrent
                .info
                .pieces
                .chunks(20)
                .map(|sha1| Piece {
                    content: None,
                    original_sha1: sha1.to_owned(),
                    status: PieceStatus::NotStarted,
                })
                .collect(),
        }
    }
}
