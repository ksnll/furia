use crate::{messages::BLOCK_BYTES, parse_torrent::TorrentFile};

#[derive(Debug, Clone)]
pub enum PieceStatus {
    NotStarted,
    Downloading,
    Downloaded,
    ShaVerified,
    WrittenToDisk,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    NotStarted,
    Downloaded([u8; BLOCK_BYTES as usize]),
    Downloading,
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub content: Vec<Block>,
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
                    content: vec![
                        Block::NotStarted;
                        (torrent.info.piece_length as u32 / BLOCK_BYTES)
                            as usize
                    ],
                    original_sha1: sha1.to_owned(),
                    status: PieceStatus::NotStarted,
                })
                .collect(),
        }
    }

    pub fn find_first_block(&self) -> Option<(usize, usize)> {
        for (piece_index, piece) in self.pieces.iter().enumerate() {
                for (block_index, block) in piece.content.iter().enumerate() {
                    if *block == Block::NotStarted {
                        return Some((piece_index, block_index));
                    }
            }
        }
        None
    }
}
