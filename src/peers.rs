use anyhow::{anyhow, Result};
use tokio::sync::Mutex;
use std::{
    io::{Read, Write},
    net::TcpStream, sync::Arc,
};

use crate::{
    download::Download,
    messages::Message,
    parse_torrent::TorrentFile,
    tracker::{get_info_hash, Peer},
};

pub enum PeerStatus {
    Chocked,
    Interested,
}

pub struct ConnectionManager<'a> {
    torrent: &'a TorrentFile,
    download: Arc<Mutex<Download>>,
    peer_connections: Vec<PeerConnection>,
}

impl<'a> ConnectionManager<'a> {
    pub fn new(torrent: &'a TorrentFile, download: Download) -> Self {
        Self {
            torrent,
            download: Arc::new(Mutex::new(download)),
            peer_connections: Vec::new(),
        }
    }

    pub fn add_peer(&mut self, peer: Peer) -> Result<()> {
        let peer_connection = PeerConnection::new(peer)?;
        self.peer_connections.push(peer_connection);
        Ok(())
    }

    pub async fn handle_messages(self) -> Result<()> {
        let torrent = self.torrent.clone();
        let download = self.download.clone();
        let info_hash = get_info_hash(&torrent.info)?;
        dbg!("Number of peers:", &self.peer_connections.len());
        for mut peer_connection in self.peer_connections {
            dbg!("One thread spawning");
            let download = download.clone();
            let torrent = torrent.clone();
            let task = tokio::spawn(async move {
                loop {
                    if let Err(e) = peer_connection.connect(){
                        dbg!("Failed to connect to peer: {}", e);
                        break;
                    }
                    if let Err(e) = peer_connection.handshake(&info_hash){
                        dbg!("Failed to handshake to peer: {}", e);
                        break;
                    }

                    let download = download.lock().await;
                    if let Err(e) = peer_connection.bitfield(&torrent, &download){
                        dbg!("Failed to handshake to peer: {}", e);
                        break;
                    }
                    
                    if let Err(e) = peer_connection.interested() {
                        dbg!("Failed to send interest message to peer: {}", e);
                        break;
                    }
                    let mut len = [0; 4];
                    if let Some(connection) = peer_connection.connection.as_mut() {
                        dbg!("Connection established and hadshake successfull");
                        let mut one = [0_u8;1];
                        connection.read_exact(&mut one).unwrap();
                        dbg!("One byte read");
                        dbg!(one);

                        match connection.read_exact(&mut len) {
                            Ok(_) => {
                                let length = u32::from_be_bytes(len);
                                let mut message = vec![0; length as usize];
                                connection.read_exact(&mut message).unwrap();
                                if length == 0 {
                                    dbg!("Keep alive");
                                    continue;
                                }
                                let message_id = message[0];
                                match message_id {
                                    0 => {
                                        dbg!("Choke");
                                        peer_connection.am_status = Some(PeerStatus::Chocked);
                                    }
                                    1 => {
                                        peer_connection.request().unwrap();
                                        peer_connection.am_status = Some(PeerStatus::Interested);
                                    }
                                    4 => {
                                        dbg!("Have");
                                    }
                                    5 => {
                                        dbg!("Bitfield");
                                        peer_connection.bitfield = message[1..].to_vec();
                                    }
                                    7 => {
                                        dbg!("Piece");
                                    }
                                    _ => {
                                        dbg!("Unknown message, {}", &message);
                                    }
                                }
                            }
                            Err(error) => {
                                dbg!("Failed to read data from the peer: {}", error);
                                break;
                            }
                        }
                    }
                }
            });
            task.await?;
        }
        Ok(())
    }
}

pub struct PeerConnection {
    peer: Peer,
    am_status: Option<PeerStatus>,
    peer_status: Option<PeerStatus>,
    connection: Option<TcpStream>,
    bitfield: Vec<u8>,
}

impl PeerConnection {
    fn new(peer: Peer) -> Result<Self> {
        Ok(Self {
            peer,
            connection: None,
            am_status: None,
            peer_status: None,
            bitfield: Vec::new(),
        })
    }

    fn connect(&mut self) -> Result<()> {
        let connection = TcpStream::connect(format!("{}:{}", &self.peer.ip, &self.peer.port))?;
        self.connection = Some(connection);
        Ok(())
    }

    fn handshake(&mut self, info_hash: &[u8; 20]) -> Result<()> {
        let mut concatenated_bytes = Vec::new();
        concatenated_bytes
            .write_all(&19_u8.to_be_bytes())
            .expect("Failed to write number of bytes");
        concatenated_bytes.extend_from_slice("BitTorrent protocol00000000".as_bytes());
        concatenated_bytes.extend_from_slice(info_hash);
        if let Some(connection) = &mut self.connection {
            connection.write_all(&concatenated_bytes)?;
            let mut len = [0; 1];
            connection.read_exact(&mut len)?;
            let total_length = len[0] + 8 + 20 + 20;
            let mut response = vec![0; total_length as usize];
            connection.read_exact(&mut response)?;
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
        }
        Ok(())
    }

    fn bitfield(&mut self, torrent: &TorrentFile, download: &Download) -> Result<()> {
        let message = Message::bitfield(&torrent, &download);
        if let Some(connection) = &mut self.connection {
            connection.write_all(&message)?;
        }
        Ok(())
    }

    fn interested(&mut self) -> Result<()> {
        let message = Message::interested();
        if let Some(connection) = &mut self.connection {
            connection.write_all(&message)?;
        }
        Ok(())
    }

    fn request(&mut self) -> Result<()> {
        let message = Message::request(0, 0);
        if let Some(connection) = &mut self.connection {
            connection.write_all(&message)?;
        }
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
        if let Some(connection) = &mut self.connection {
            connection.write_all(&concatenated_bytes)?;

            let mut response = vec![0; 16];
            connection.read_exact(&mut response)?;
            dbg!(&response);
        }

        Ok(())
    }
}
