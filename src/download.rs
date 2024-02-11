use crate::{messages::BLOCK_BYTES, parse_torrent::TorrentFile};
use sha1::{Digest, Sha1};
use tracing::warn;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
                        (torrent.info.piece_length as u32 / BLOCK_BYTES) as usize
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

    pub fn set_piece(&mut self, data: &[u8], piece_index: usize) -> () {
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let info_hash = hasher.finalize();

        if info_hash.as_slice() == self.pieces[piece_index].original_sha1.as_slice() {
            self.pieces[piece_index].status = PieceStatus::ShaVerified;
            self.pieces[piece_index].content = data
                .chunks(BLOCK_BYTES as usize)
                .map(|block| Block::Downloaded(block.try_into().unwrap()))
                .collect();
        } else {
            warn!("Data not valid for piece {}", piece_index);
            self.pieces[piece_index].status = PieceStatus::NotStarted;
        }
    }

    pub fn set_block(&mut self, data: &[u8], piece_index: usize, piece_offset: usize) -> Option<Vec<u8>> {
        self.pieces[piece_index as usize].content[piece_offset / BLOCK_BYTES as usize] =
            Block::Downloaded(data.try_into().unwrap());

        if self.pieces[piece_index as usize]
            .content
            .iter()
            .all(|block| *block != Block::NotStarted && *block != Block::Downloading)
        {
            self.pieces[piece_index as usize].status = PieceStatus::Downloaded;
            let data = self.pieces[piece_index as usize]
                .content
                .iter()
                .map(|block| match block {
                    Block::Downloaded(data) => data.to_vec(),
                    _ => panic!("Block not downloaded"),
                })
                .flatten()
                .collect::<Vec<u8>>();

            let mut hasher = Sha1::new();
            hasher.update(&data);
            let info_hash = hasher.finalize();
            if info_hash.as_slice() == self.pieces[piece_index as usize].original_sha1.as_slice() {
                self.pieces[piece_index as usize].status = PieceStatus::ShaVerified;
                return Some(data)
            } else {
                warn!("Failed to download piece");
                self.pieces[piece_index as usize].status = PieceStatus::NotStarted;
                return None
            };
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::parse_torrent;
    use super::Download;
    
    #[test]
    fn it_sets_invalid_pieces(){
        let torrent = parse_torrent("./data/ubuntu-22.04.3-live-server-amd64.iso.torrent");
        let mut download = Download::from(&torrent);
        download.set_piece(&vec![0; torrent.info.piece_length as usize], 0);
        assert_eq!(download.pieces[0].status, super::PieceStatus::NotStarted);
    } 
}
