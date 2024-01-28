use anyhow::{anyhow, Result};
use sha1::{Digest, Sha1};
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::Mutex};

use crate::{
    download::{Download, PieceStatus},
    messages::{Message, BLOCK_BYTES},
    parse_torrent::TorrentFile,
    tracker::{get_info_hash, Peer},
};

pub enum PeerStatus {
    Chocked,
    Interested,
}

pub struct ConnectionManager {
    torrent: Arc<TorrentFile>,
    download: Arc<Mutex<Download>>,
    peer_connections: Vec<PeerConnection>,
    peer_id: String,
}

impl ConnectionManager {
    pub fn new(torrent: TorrentFile, download: Download, peer_id: &str) -> Self {
        Self {
            torrent: Arc::new(torrent),
            download: Arc::new(Mutex::new(download)),
            peer_connections: Vec::new(),
            peer_id: peer_id.to_owned(),
        }
    }

    pub fn add_peer(&mut self, peer: Peer) -> Result<()> {
        let peer_connection = PeerConnection::new(peer)?;
        self.peer_connections.push(peer_connection);
        Ok(())
    }

    pub async fn handle_messages(self) -> Result<Vec<tokio::task::JoinHandle<()>>> {
        let torrent = self.torrent.clone();
        let download = self.download.clone();
        let peer_id = self.peer_id.clone();
        let info_hash = get_info_hash(&torrent.info)?;
        println!("Number of peers: {}", &self.peer_connections.len());
        let mut tasks = Vec::new();
        for mut peer_connection in self.peer_connections {
            println!("Connecting to peer: {}", &peer_connection.peer.ip);
            let download = download.clone();
            let torrent = torrent.clone();
            let peer_id = peer_id.clone();
            let task = tokio::spawn(async move {
                if let Err(e) = peer_connection.connect() {
                    println!("Failed to connect to peer: {}", e);
                    return;
                }
                if let Err(e) = peer_connection.handshake(&info_hash, &peer_id) {
                    println!("Failed to handshake to peer: {}", e);
                    return;
                }

                // if let Err(e) = peer_connection.bitfield(&torrent, &download) {
                //     dbg!("Failed to handshake to peer: {}", e);
                //     return;
                // }

                if let Err(e) = peer_connection.interested() {
                    println!("Failed to send interest message to peer: {}", e);
                    return;
                }

                loop {
                    let mut len = Vec::new();
                    if let Some(connection) = peer_connection.connection.as_mut() {
                        match connection.take(4).read_to_end(&mut len) {
                            Ok(_) => {
                                if len.len() != 4 {
                                    println!(
                                        "Failed to read data from peer {}",
                                        &peer_connection.peer.ip
                                    );
                                    return;
                                }
                                let length = u32::from_be_bytes(len.try_into().unwrap());
                                let mut message = vec![0; length as usize];

                                connection.read_exact(&mut message).unwrap();
                                if length == 0 {
                                    peer_connection.keep_alive().unwrap();
                                    continue;
                                }
                                let message_id = message[0];
                                match message_id {
                                    0 => {
                                        println!("Choke from peer {}", &peer_connection.peer.ip);
                                        peer_connection.am_status = Some(PeerStatus::Chocked);
                                    }
                                    1 => {
                                        println!("Unchoke from peer {}", &peer_connection.peer.ip);
                                        peer_connection.am_status = Some(PeerStatus::Interested);
                                        let download = download.lock().await;
                                        let (piece, block) = download.find_first_block().unwrap();
                                        peer_connection
                                            .request(piece as u32, block as u32)
                                            .unwrap();
                                    }
                                    4 => {
                                        println!("Have");
                                    }
                                    5 => {
                                        println!("Bitfield from peer {}", &peer_connection.peer.ip);
                                        peer_connection.bitfield = message[1..].to_vec();
                                        peer_connection.interested().unwrap();
                                    }
                                    7 => {
                                        let piece_index =
                                            u32::from_be_bytes(message[1..5].try_into().unwrap());
                                        let piece_offset =
                                            u32::from_be_bytes(message[5..9].try_into().unwrap());
                                        dbg!(&piece_index, &piece_offset);
                                        let block = &message[9..];
                                        let mut download = download.lock().await;
                                        if !&download.pieces[piece_index as usize].content.is_some()
                                        {
                                            download.pieces[piece_index as usize].content =
                                                Some(vec![
                                                    None;
                                                    (torrent.info.piece_length as u32 / BLOCK_BYTES)
                                                        as usize
                                                ]);
                                        }
                                        download.pieces[piece_index as usize]
                                            .content
                                            .as_mut()
                                            .unwrap()
                                            [(piece_offset / BLOCK_BYTES) as usize] =
                                            Some(block.try_into().unwrap());

                                        if download.pieces[piece_index as usize]
                                            .content
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .all(|block| block.is_some())
                                        {
                                            download.pieces[piece_index as usize].status =
                                                PieceStatus::Downloaded;
                                            let data = download.pieces[piece_index as usize]
                                                .content
                                                .as_ref()
                                                .unwrap()
                                                .iter()
                                                .map(|block| block.as_ref().unwrap())
                                                .flatten()
                                                .cloned()
                                                .collect::<Vec<u8>>();

                                            let mut hasher = Sha1::new();
                                            hasher.update(&data);
                                            let info_hash = hasher.finalize();

                                            if info_hash.as_slice()
                                                == download.pieces[piece_index as usize]
                                                    .original_sha1
                                                    .as_slice()
                                            {
                                                download.pieces[piece_index as usize].status =
                                                    PieceStatus::ShaVerified;

                                                dbg!(&torrent.info.name);
                                                let mut file = OpenOptions::new()
                                                    .create(true)
                                                    .append(true)
                                                    .open(&torrent.info.name)
                                                    .await
                                                    .unwrap();
                                                file.write_all(&data).await.unwrap();
                                            } else {
                                                dbg!("Sha1 mismatch");
                                                download.pieces.remove(piece_index as usize);
                                            }

                                            println!("Piece {} downloaded", &piece_index);
                                        }

                                        let (piece, block) = download.find_first_block().unwrap();
                                        peer_connection
                                            .request(piece as u32, block as u32)
                                            .unwrap();
                                        println!("Piece");
                                    }
                                    20 => {
                                        println!("Extended");
                                    }
                                    _ => {
                                        println!("Unknown message, {}", &message_id);
                                        dbg!(String::from_utf8_lossy(&message));
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
            tasks.push(task);
        }
        Ok(tasks)
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

    fn handshake(&mut self, info_hash: &[u8; 20], peer_id: &str) -> Result<()> {
        let mut concatenated_bytes = Vec::new();
        Write::write_all(&mut concatenated_bytes, &[19_u8])
            .expect("Failed to write number of bytes");
        concatenated_bytes.extend_from_slice("BitTorrent protocol00000000".as_bytes());
        concatenated_bytes.extend_from_slice(info_hash);
        concatenated_bytes.extend_from_slice(&peer_id.as_bytes());
        if let Some(connection) = &mut self.connection {
            connection.write_all(&concatenated_bytes)?;
            let mut len = [0_u8; 1];
            connection.read_exact(&mut len)?;
            let mut message = vec![0; len[0] as usize];
            connection.read_exact(&mut message)?;
            let mut reserved = [0_u8; 8];
            connection.read_exact(&mut reserved)?;
            let mut info_hash = [0_u8; 20];
            let mut peer_id = [0_u8; 20];
            connection.read_exact(&mut info_hash)?;
            connection.read_exact(&mut peer_id)?;
        } else {
            dbg!("Failed to establish connection");
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

    fn keep_alive(&mut self) -> Result<()> {
        let message = Message::keep_alive();
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

    fn request(&mut self, piece_index: u32, piece_offset: u32) -> Result<()> {
        let message = Message::request(piece_index, piece_offset);
        if let Some(connection) = &mut self.connection {
            connection.write_all(&message)?;
        }
        Ok(())
    }
}
