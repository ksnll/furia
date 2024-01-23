use crate::{download::Download, parse_torrent::TorrentFile};

pub struct Message {}

pub const BLOCK_BYTES: u8 = 2 ^ 14;

#[repr(u8)]
pub enum MessageType {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
    Port,
}

impl Message {
    pub fn choke() -> Vec<u8> {
        let len = 0_u32.to_be_bytes();
        let mut message = Vec::from(len);
        message.push(MessageType::Choke as u8);
        message
    }

    pub fn unchoke() -> Vec<u8> {
        let len = 1_u32.to_be_bytes();
        let mut message = Vec::from(len);
        message.push(MessageType::Unchoke as u8);
        message
    }

    pub fn interested() -> Vec<u8> {
        let len = 1_u32.to_be_bytes();
        let mut message = Vec::from(len);
        message.push(MessageType::Interested as u8);
        message
    }

    pub fn not_interested() -> Vec<u8> {
        let len = 1_u32.to_be_bytes();
        let mut message = Vec::from(len);
        message.push(MessageType::NotInterested as u8);
        message
    }

    pub fn bitfield(torrent: &TorrentFile, download: &Download) -> Vec<u8> {
        let number_of_pieces = ((torrent.info.length.unwrap() + torrent.info.piece_length - 1)
            / torrent.info.piece_length) as usize;
        let bitfield_size = ((number_of_pieces + 7) / 8) as u32;

        let len = bitfield_size as usize + 1;
        let mut message = Vec::from(len.to_be_bytes());
        message.push(MessageType::Bitfield as u8);
        message.extend_from_slice(&vec![0_u8; len]);
        message
    }

    pub fn request(piece_index: u8, piece_offset: u8) -> Vec<u8> {
        let len = 13_u32.to_be_bytes();
        let mut message = Vec::from(len);
        message.push(MessageType::Request as u8);
        message.push(piece_index);
        message.push(piece_offset * BLOCK_BYTES);
        message.push(BLOCK_BYTES);
        message
    }

    pub fn piece(piece_index: u8, piece_offset: u8, block: Vec<u8>) {
        todo!();
    }

    pub fn cancel(piece_index: u8, piece_offset: u8) {
        todo!();
    }

    pub fn port(port: u8) {
        let len = 3_u32.to_be_bytes();
        let mut message = Vec::from(len);
        message.push(9_u8);
        todo!();
    }
}

#[cfg(test)]
mod test {
    use super::Message;

    fn request_message() {
        let expected = vec![0x0 0x0, 0x0, 0xC]
        Message::request(0, 0);
    }
}
