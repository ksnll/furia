use anyhow::{anyhow, Result};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{
    parse_torrent::TorrentFile,
    tracker::{get_info_hash, Peer},
};

pub enum PeerStatus {
    Chocked,
    Interested,
}

pub enum PeerConnectionState {
    Handshake,
    Data,
    Closed,
    Error,
    Uninitialized,
    Connected,
}

pub struct PeerConnection {
    peer: Peer,
    am_status: Option<PeerStatus>,
    peer_status: Option<PeerStatus>,
    connection: TcpStream,
    peer_connection_state: PeerConnectionState,
    bitfield: Vec<u8>,
}

pub struct ConnectionManager<'a> {
    connections: Vec<PeerConnection>,
    torrent: &'a TorrentFile,
}

impl<'a> ConnectionManager<'a> {
    pub fn new(torrent: &'a TorrentFile) -> Self {
        Self {
            connections: Vec::new(),
            torrent,
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
            connection.bitfield(&self.torrent)?;
            // connection.interested(0)?;
        }
        Ok(())
    }
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
            peer_connection_state: PeerConnectionState::Uninitialized,
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
            self.peer_connection_state = PeerConnectionState::Error;
            return Err(anyhow!("Invalid protocol"));
        }
        if &response[27..47] != info_hash.as_slice() {
            self.peer_connection_state = PeerConnectionState::Error;
            return Err(anyhow!(
                "Invalid info hash {} {}",
                hex::encode(&response[27..47]),
                hex::encode(info_hash.as_slice())
            ));
        }
        self.peer_connection_state = PeerConnectionState::Connected;
        Ok(())
    }

    fn bitfield(&mut self, torrent: &TorrentFile) -> Result<()> {
        let mut concatenated_bytes = Vec::new();
        let number_of_pieces = ((torrent.info.length.unwrap() + torrent.info.piece_length - 1)
            / torrent.info.piece_length) as usize;
        let bitfield_size = ((number_of_pieces + 7) / 8) as usize;

        concatenated_bytes
            .write_all(&((bitfield_size as u8+ 1)).to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes
            .write_all(&5_u32.to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes
            .write_all(&vec![0; bitfield_size])
            .expect("Failed to write number of bytes");
        self.connection.write_all(&concatenated_bytes)?;
        let mut response = vec![0; bitfield_size as usize];
        self.connection.read_exact(&mut response)?;
        dbg!(hex::encode(&response[0..20]));
        // self.bitfield = response;
        Ok(())
    }

    fn interested(&mut self, index: u32) -> Result<()> {
        let mut concatenated_bytes = Vec::new();
        concatenated_bytes
            .write_all(&1_u32.to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes
            .write_all(&2_u8.to_be_bytes())
            .expect("Failed to write number of bytes");
        self.connection.write_all(&concatenated_bytes)?;

        let mut response = vec![0; 16];
        self.connection.read_exact(&mut response)?;

        dbg!(&response);
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
