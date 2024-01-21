use std::{net::TcpStream, io::{Write, Read}};
use anyhow::{Result, anyhow};

use crate::{parse_torrent::TorrentFile, tracker::{Peer, get_encoded_info_hash, get_info_hash}};

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
        })
    }

    fn handshake(&mut self, torrent: &TorrentFile) -> Result<()> {
        let info_hash = get_info_hash(&torrent.info)?;
        let mut concatenated_bytes = Vec::new();
        concatenated_bytes.write_all(&19_u8.to_be_bytes()).expect("Failed to write number of bytes");
        concatenated_bytes.extend_from_slice("BitTorrent protocol00000000".as_bytes());
        concatenated_bytes.extend_from_slice(&info_hash);
        self.connection.write_all(&concatenated_bytes)?;
        let mut len = [0;1];
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
            return Err(anyhow!("Invalid info hash {} {}", hex::encode(&response[27..47]), hex::encode(info_hash.as_slice())));
        }
        self.peer_connection_state = PeerConnectionState::Connected;
        Ok(())
    }

}


