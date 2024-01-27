use crate::{
    download::Download,
    parse_torrent::{bitfield_size, TorrentFile},
};

pub struct Message {}

pub const BLOCK_BYTES: u32 = 16384;

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
    KeepAlive
}

impl Message {
    pub fn keep_alive() -> Vec<u8> {
        let len = 0_u32.to_be_bytes();
        Vec::from(len)
    }

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
        let bitfield_size = bitfield_size(&torrent);

        let len = bitfield_size as u32 + 1;
        let mut message = Vec::from(len.to_be_bytes());
        message.push(MessageType::Bitfield as u8);
        message.extend_from_slice(&vec![0_u8; len as usize * 4]);
        message
    }

    pub fn request(piece_index: u32, piece_offset: u32) -> Vec<u8> {
        let len = 13_u32.to_be_bytes();
        let mut message = Vec::from(len);
        message.push(MessageType::Request as u8);
        message.extend_from_slice(&piece_index.to_be_bytes());
        message.extend_from_slice(&(piece_offset * BLOCK_BYTES as u32).to_be_bytes());
        message.extend_from_slice(&BLOCK_BYTES.to_be_bytes());
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

    #[test]
    fn request_message() {
        assert_eq!(
            Message::request(0, 0),
            vec![0x00, 0x00, 0x00, 0x0D, 0x06, 0x00, 0x00, 0x0C]
        );
    }
}
