use anyhow::{anyhow, Result};
use std::{
    future::Future,
    io::{Read, Write},
    net::TcpStream,
};

use crate::{
    download::Download,
    messages::Message,
    parse_torrent::{bitfield_size, TorrentFile},
    tracker::{get_info_hash, Peer},
};

pub enum PeerStatus {
    Chocked,
    Interested,
}

pub struct ConnectionManager<'a> {
    connections: Vec<PeerConnection>,
    torrent: &'a TorrentFile,
    download: Download,
}

impl<'a> ConnectionManager<'a> {
    pub fn new(torrent: &'a TorrentFile, download: Download) -> Self {
        Self {
            connections: Vec::new(),
            torrent,
            download,
        }
    }

    pub fn add_peer(&mut self, peer: Peer) -> Result<()> {
        let connection = PeerConnection::new(peer)?;
        self.connections.push(connection);
        Ok(())
    }

    pub fn connect_to_peers(&mut self) -> Result<()> {
        for connection in &mut self.connections {
            connection.handshake(&self.torrent)?;
            connection.bitfield(&self.torrent, &self.download)?;
            connection.interested()?;
        }
        Ok(())
    }

    pub async fn handle_messages(self) -> Result<()> {
        for mut connection in self.connections {
            tokio::spawn(async move {
                loop {
                    let mut len = [0; 4];
                    connection.connection.read_exact(&mut len).unwrap();
                    let length = u32::from_be_bytes(len);
                    let mut message = vec![0; length as usize];
                    connection.connection.read_exact(&mut message).unwrap();
                    if length == 0 {
                        dbg!("Keep alive");
                        continue;
                    }
                    let message_id = message[0];
                    match message_id {
                        0 => {
                            dbg!("Choke");
                            connection.am_status = Some(PeerStatus::Chocked);
                        }
                        1 => {
                            dbg!("Unchoke");
                            connection.am_status = Some(PeerStatus::Interested);
                        }
                        4 => {
                            dbg!("Have");
                        }
                        5 => {
                            dbg!("Bitfield");
                            connection.bitfield = message[1..].to_vec();
                        }
                        7 => {
                            dbg!("Piece");
                        }
                        _ => {
                            dbg!("Unknown message");
                        }
                    }
                }
            });
        }
        Ok(())
    }
}

pub struct PeerConnection {
    peer: Peer,
    am_status: Option<PeerStatus>,
    peer_status: Option<PeerStatus>,
    connection: TcpStream,
    bitfield: Vec<u8>,
}

impl PeerConnection {
    fn new(peer: Peer) -> Result<Self> {
        dbg!("Connectiong to peer: {:?}", &peer);
        let connection = TcpStream::connect(format!("{}:{}", peer.ip, peer.port))?;
        Ok(Self {
            peer,
            connection,
            am_status: None,
            peer_status: None,
            bitfield: Vec::new(),
        })
    }

    fn handshake(&mut self, torrent: &TorrentFile) -> Result<()> {
        let info_hash = get_info_hash(&torrent.info)?;
        let mut concatenated_bytes = Vec::new();
        concatenated_bytes
            .write_all(&19_u8.to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes.extend_from_slice("BitTorrent protocol00000000".as_bytes());
        concatenated_bytes.extend_from_slice(&info_hash);
        self.connection.write_all(&concatenated_bytes)?;
        let mut len = [0; 1];
        self.connection.read_exact(&mut len)?;
        let total_length = len[0] + 8 + 20 + 20;
        let mut response = vec![0; total_length as usize];
        self.connection.read_exact(&mut response)?;
        if &response[0..19] != "BitTorrent protocol".as_bytes() {
            return Err(anyhow!("Invalid protocol"));
        }
        if &response[27..47] != info_hash.as_slice() {
            return Err(anyhow!(
                "Invalid info hash {} {}",
                hex::encode(&response[27..47]),
                hex::encode(info_hash.as_slice())
            ));
        }
        self.am_status = Some(PeerStatus::Chocked);
        Ok(())
    }

    fn bitfield(&mut self, torrent: &TorrentFile, download: &Download) -> Result<()> {
        let message = Message::bitfield(&torrent, &download);
        self.connection.write_all(&message)?;
        Ok(())
    }

    fn interested(&mut self) -> Result<()> {
        let message = Message::interested();
        self.connection.write_all(&message)?;
        Ok(())
    }

    fn download_block(&mut self, index: u32) -> Result<()> {
        let mut concatenated_bytes = Vec::new();
        concatenated_bytes
            .write_all(&13_u32.to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes
            .write_all(&6_u8.to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes
            .write_all(&index.to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes
            .write_all(&0_u32.to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes
            .write_all(&16384_u32.to_be_bytes())
            .expect("Failed to write number of bytes");
        self.connection.write_all(&concatenated_bytes)?;

        let mut response = vec![0; 16];
        self.connection.read_exact(&mut response)?;

        dbg!(&response);
        Ok(())
    }
}
