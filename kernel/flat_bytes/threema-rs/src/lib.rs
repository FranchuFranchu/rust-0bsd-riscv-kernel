pub mod identity;
pub mod packets;
mod rest;

use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time;
use std::{error, fmt, io};

use flat_bytes::Flat;
use log::debug;
use log::warn;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;
use sodiumoxide::randombytes;

use packets::*;

// https://github.com/threema-ch/threema-android/blob/329b33d7bace99f5078ff08ef996a27c628be6e5/app/build.gradle#L91-L93
const MSG_SERVER: &str = "g-33.0.threema.ch:5222";
// https://github.com/threema-ch/threema-android/blob/329b33d7bace99f5078ff08ef996a27c628be6e5/app/build.gradle#L98
const SERVER_LONG_TERM_PUBKEY: [u8; 32] = [
    69, 11, 151, 87, 53, 39, 159, 222, 203, 51, 19, 100, 143, 95, 198, 238, 159, 244, 54, 14, 169,
    42, 140, 23, 81, 198, 97, 228, 192, 216, 201, 9,
];

type PrivateKey = SecretKey;

#[derive(Debug)]
pub enum Error {
    InvalidPrivateKey,
    InvalidPublicKey,
    InvalidBackupOrPassword,
    Io(io::Error),
    ParseError(String),
    RequestError,
    InvalidID,
    NotConnected,
    DecryptionFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrivateKey => f.write_str("Invalid private key"),
            Self::InvalidPublicKey => f.write_str("Invalid public key"),
            Self::InvalidBackupOrPassword => f.write_str("Invalid backup or password"),
            Self::ParseError(s) => write!(f, "Parser error: {}", s),
            Self::RequestError => f.write_str("Request failed"),
            Self::InvalidID => f.write_str("Invalid ID format"),
            Self::NotConnected => f.write_str("Not connected"),
            Self::DecryptionFailed => f.write_str("decryption failed"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

struct Nonce {
    prefix: Vec<u8>,
    counter: u64,
}

impl Nonce {
    fn new(prefix: Vec<u8>) -> Self {
        Self { prefix, counter: 1 }
    }

    fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut res = self.prefix.to_vec();
        res.extend_from_slice(&self.counter.to_le_bytes());
        res
    }

    fn as_nonce(&self) -> Option<box_::Nonce> {
        box_::Nonce::from_slice(&self.as_bytes())
    }

    fn inc(&mut self) {
        self.counter += 1;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Flat)]
pub struct MessageID([u8; 8]);

impl MessageID {
    pub fn from_bytes(data: [u8; 8]) -> Self {
        Self(data)
    }

    pub fn from_slice(data: &[u8]) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let mut tmp = [0u8; 8];
        tmp.copy_from_slice(data);
        Some(Self::from_bytes(tmp))
    }
}

impl fmt::Display for MessageID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0.iter() {
            write!(f, "{:02x}", b)?
        }
        Ok(())
    }
}

impl fmt::Debug for MessageID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MessageID").field(&self.to_string()).finish()
    }
}

impl Default for MessageID {
    fn default() -> Self {
        let mut res = Self(Default::default());
        randombytes::randombytes_into(&mut res.0);
        res
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Flat)]
pub struct ThreemaID([u8; 8]);

impl ThreemaID {
    pub fn from_slice(id: &[u8]) -> Result<Self> {
        if id.len() != 8 {
            return Err(Error::InvalidID);
        }
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        if id.iter().any(|c| !alphabet.contains(c)) {
            return Err(Error::InvalidID);
        }
        let mut tmp = [0u8; 8];
        tmp.copy_from_slice(id);
        Ok(Self(tmp))
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Self::from_slice(s.as_bytes())
    }

    fn as_bytes(&self) -> [u8; 8] {
        self.0
    }
}

impl fmt::Display for ThreemaID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(String::from_utf8_lossy(&self.0).as_ref())
    }
}

impl fmt::Debug for ThreemaID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ThreemaID")
            .field(&String::from_utf8_lossy(&self.0))
            .finish()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Flat)]
pub struct GroupID([u8; 8]);

pub struct Threema {
    id: ThreemaID,
    private_key: PrivateKey,
    peers: HashMap<ThreemaID, PublicKey>,
    pub nick: Option<String>,
    client_nonce: Option<Nonce>,
    server_nonce: Option<Nonce>,
    server_pubkey: Option<PublicKey>,
    ephemeral_private_key: Option<PrivateKey>,
    // ephemeral_public_key: Option<PublicKey>,
    conn: Option<TcpStream>,
}

impl Threema {
    pub fn new(id: ThreemaID, private_key: &[u8]) -> Result<Self> {
        Ok(Self {
            id,
            private_key: PrivateKey::from_slice(private_key).ok_or(Error::InvalidPrivateKey)?,
            peers: HashMap::new(),
            client_nonce: None,
            server_nonce: None,
            nick: None,
            server_pubkey: None,
            ephemeral_private_key: None,
            // ephemeral_public_key: None,
            conn: None,
        })
    }

    pub fn from_backup(data: &str, password: &str) -> Result<Self> {
        let (id, private_key) =
            identity::decrypt_identity(data, password).ok_or(Error::InvalidBackupOrPassword)?;
        Self::new(ThreemaID::from_string(&id)?, &private_key)
    }

    fn fetch_peer_key(peer: &ThreemaID) -> Result<PublicKey> {
        let resp: rest::messages::GetPubKeyResponse =
            rest::request(&format!("/identity/{}", peer)).unwrap();
        PublicKey::from_slice(resp.public_key.as_ref()).ok_or(Error::InvalidPublicKey)
    }

    pub fn connect(&mut self) -> Result<()> {
        let mut conn = TcpStream::connect(MSG_SERVER)?;
        let client_nonce_prefix = randombytes::randombytes(16);
        let mut client_nonce = Nonce::new(client_nonce_prefix);

        let (eph_pub, eph_priv) = box_::gen_keypair();

        conn.write_all(eph_pub.as_ref()).unwrap();
        conn.write_all(client_nonce.prefix()).unwrap();

        let mut server_nonce_prefix = [0u8; 16];
        conn.read_exact(&mut server_nonce_prefix).unwrap();
        let mut ciphertext = [0u8; 64];
        conn.read_exact(&mut ciphertext).unwrap();

        let mut server_nonce = Nonce::new(server_nonce_prefix.to_vec());
        let server_lt_pub = box_::PublicKey::from_slice(&SERVER_LONG_TERM_PUBKEY).unwrap();

        let plaintext = box_::open(
            &ciphertext,
            &server_nonce.as_nonce().unwrap(),
            &server_lt_pub,
            &eph_priv,
        )
        .unwrap();

        let (server_pkey, tmp) = plaintext.split_at(32);
        assert!(client_nonce.prefix() == tmp);
        let server_pkey = box_::PublicKey::from_slice(server_pkey).unwrap();

        server_nonce.inc();

        let nonce = Nonce::new(randombytes::randombytes(16));

        let mut inner = box_::seal(
            eph_pub.as_ref(),
            &nonce.as_nonce().unwrap(),
            &server_lt_pub,
            &self.private_key,
        );
        assert!(inner.len() == 48);

        let mut outer = vec![];
        outer.extend(self.id.as_bytes().iter());
        outer.resize(outer.len() + 32, 0);
        outer.extend(server_nonce.prefix());
        outer.append(&mut nonce.as_bytes());
        outer.append(&mut inner);

        let outer = box_::seal(
            &outer,
            &client_nonce.as_nonce().unwrap(),
            &server_pkey,
            &eph_priv,
        );
        assert!(outer.len() == 144);

        conn.write_all(&outer).unwrap();
        client_nonce.inc();

        let mut ack = [0u8; 32];
        conn.read_exact(&mut ack).unwrap();
        let ack = box_::open(
            &ack,
            &server_nonce.as_nonce().unwrap(),
            &server_pkey,
            &eph_priv,
        )
        .unwrap();
        server_nonce.inc();

        assert!(ack == [0u8; 16]);

        self.client_nonce = Some(client_nonce);
        self.server_nonce = Some(server_nonce);
        self.server_pubkey = Some(server_pkey);
        self.ephemeral_private_key = Some(eph_priv);
        // self.ephemeral_public_key = Some(eph_pub);
        self.conn = Some(conn);
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<()> {
        let enc_packet = box_::seal(
            data,
            &self
                .client_nonce
                .as_ref()
                .and_then(Nonce::as_nonce)
                .ok_or(Error::NotConnected)?,
            self.server_pubkey.as_ref().ok_or(Error::NotConnected)?,
            self.ephemeral_private_key
                .as_ref()
                .ok_or(Error::NotConnected)?,
        );
        let len = enc_packet.len() as u16;
        self.conn
            .as_ref()
            .ok_or(Error::NotConnected)?
            .write_all(&len.to_le_bytes())?;
        self.conn
            .as_ref()
            .ok_or(Error::NotConnected)?
            .write_all(&enc_packet)?;
        self.client_nonce.as_mut().map(Nonce::inc);
        Ok(())
    }

    fn get_peer_key<'a>(&'a mut self, peer: ThreemaID) -> Result<&'a PublicKey> {
        use std::collections::hash_map::Entry::*;
        let pk = match self.peers.entry(peer) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => {
                let pk = Self::fetch_peer_key(&peer)?;
                entry.insert(pk)
            }
        };
        Ok(pk)
    }

    fn get_nickname(&self) -> [u8; 32] {
        let id_bytes = &self.id.as_bytes();
        let nick = self.nick.as_ref().map(String::as_bytes).unwrap_or(id_bytes);
        let mut nickname = [0u8; 32];
        let n = if nick.len() < 32 { nick.len() } else { 32 };
        nickname[..n].copy_from_slice(&nick[..n]);
        nickname
    }

    fn send_message(&mut self, receiver: ThreemaID, mut data: Vec<u8>) -> Result<MessageID> {
        let sender = self.id;
        let nickname = self.get_nickname();
        // workaround for https://github.com/rust-lang/rust/issues/21906
        let priv_key = self.private_key.clone();
        let public_key = self.get_peer_key(receiver)?;
        let now = time::SystemTime::now();
        let now = now.duration_since(time::UNIX_EPOCH).unwrap_or_default();
        let timestamp = now.as_secs() as u32;
        let mut header = Header {
            sender,
            receiver,
            nonce: Default::default(),
            msg_id: Default::default(),
            nickname,
            timestamp,
            flags: 1,
        };
        randombytes::randombytes_into(&mut header.nonce);
        let msg_id = header.msg_id;

        let pad = randombytes::randombytes_uniform(32) as u8;
        data.append(&mut vec![pad; pad as usize]);

        let ciphertext = box_::seal(
            &data,
            &box_::Nonce::from_slice(&header.nonce).unwrap(),
            public_key,
            &priv_key,
        );

        let pt = Packet::ClientToServer(header);
        debug!("Sending packet {:#?}", pt);

        let mut packet = pt.serialize();
        packet.extend(ciphertext.into_iter());
        self.send(&packet)?;

        Ok(msg_id)
    }

    pub fn send_text_message(&mut self, receiver: ThreemaID, message: String) -> Result<MessageID> {
        let msg = Message::Text(Text { message });
        debug!("Sending text {:#?}", msg);
        let data = msg.serialize();
        self.send_message(receiver, data)
    }

    fn confirm_receipt(&mut self, receiver: ThreemaID, msg_id: MessageID) -> Result<MessageID> {
        let rcpt = Message::DeliveryReceipt(MessageStatus::Delivered, msg_id);
        debug!("Sending receipt {:#?}", rcpt);
        let data = rcpt.serialize();
        self.send_message(receiver, data)
    }

    fn send_ack(&mut self, receiver: ThreemaID, msg_id: MessageID) -> Result<()> {
        let ack = Packet::ClientAck(receiver, msg_id);
        debug!("Sending ack {:#?}", ack);
        let data = ack.serialize();
        self.send(&data)
    }

    pub fn receive_packet(&mut self) -> Result<(Packet, Vec<u8>)> {
        let mut l = [0u8; 2];
        let conn = self.conn.as_mut().ok_or(Error::NotConnected)?;
        conn.read_exact(&mut l)?;
        let l = u16::from_le_bytes(l);
        let mut buf = vec![0u8; l as usize];
        conn.read_exact(&mut buf).unwrap();
        let server_nonce = self.server_nonce.as_mut().ok_or(Error::NotConnected)?;
        let mut msg = box_::open(
            &buf,
            &server_nonce.as_nonce().unwrap(),
            self.server_pubkey.as_ref().ok_or(Error::NotConnected)?,
            self.ephemeral_private_key
                .as_ref()
                .ok_or(Error::NotConnected)?,
        )
        .map_err(|_| Error::DecryptionFailed)?;
        server_nonce.inc();
        let (packet, size) = Packet::deserialize_with_size(&msg)
            .ok_or_else(|| Error::ParseError(format!("packet: {:?}", msg)))?;
        msg.drain(0..size);
        Ok((packet, msg))
    }

    pub fn receive(&mut self) -> Result<ServerMessage> {
        loop {
            let (packet, payload) = self.receive_packet()?;
            match packet {
                Packet::ServerToClient(hdr) => {
                    let sender = hdr.sender;
                    self.send_ack(sender, hdr.msg_id)?;
                    // workaround for https://github.com/rust-lang/rust/issues/21906
                    let priv_key = self.private_key.clone();
                    let pub_key = self.get_peer_key(sender)?;
                    let data = box_::open(
                        &payload,
                        &box_::Nonce::from_slice(&hdr.nonce).unwrap(),
                        pub_key,
                        &priv_key,
                    )
                    .map_err(|_| Error::DecryptionFailed)?;
                    let pad = *data.last().unwrap() as usize;
                    let data = &data[..data.len() - pad];
                    let (msg, s) = Message::deserialize_with_size(&data)
                        .ok_or_else(|| Error::ParseError(format!("message: {:?}", data)))?;
                    if s < data.len() {
                        warn!("Unprocessed data: {:#x?}", &data[s..]);
                    }

                    match msg {
                        Message::DeliveryReceipt(_, _) => {}
                        Message::TypingNotification => {}
                        _ => {
                            self.confirm_receipt(sender, hdr.msg_id)?;
                        }
                    }

                    return Ok(ServerMessage {
                        msg_id: hdr.msg_id,
                        sender,
                        data: msg,
                    });
                }
                Packet::ConnectionEstablished => debug!("Connection established"),
                Packet::ServerAck(_, mid) => debug!("Packet {} acked by server", mid),
                _ => {
                    warn!("Unhandled packet: {:#?}", packet);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ServerMessage {
    pub msg_id: MessageID,
    pub sender: ThreemaID,
    pub data: Message,
}
