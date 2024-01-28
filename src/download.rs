use crate::{parse_torrent::TorrentFile, messages::BLOCK_BYTES};

#[derive(Debug, Clone)]
pub enum PieceStatus {
    NotStarted,
    Downloading,
    Downloaded,
    ShaVerified,
    WrittenToDisk,
}

type Block = [u8; BLOCK_BYTES as usize];
#[derive(Debug, Clone)]
pub struct Piece {
    pub content: Option<Vec<Option<Block>>>,
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

    pub fn find_first_block(&self) -> Option<(usize, usize)> {
        for (piece_index, piece) in self.pieces.iter().enumerate() {
            if let Some(blocks) = &piece.content {
                for (block_index, block) in blocks.iter().enumerate() {
                    if block.is_none() {
                        return Some((piece_index, block_index));
                    }
                }
            } else {
                return Some((piece_index, 0));
            }
        }
        None
    }
}
