use std::{io::Write, net::SocketAddr};

use byte_util::{Big, Den, DenWith};
use packet_builder::Den;

use crate::bytes::{Magic, RakAddress, RakString};

pub fn decode<P: Den>(buffer: &[u8]) -> std::io::Result<P> {
    let mut cursor = std::io::Cursor::new(buffer);
    P::decode(&mut cursor)
}

pub fn encode<P: Den>(packet: P, id: u8) -> std::io::Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(vec![]);
    cursor.write_all(&[id])?;
    P::encode(&packet, &mut cursor)?;
    Ok(cursor.into_inner())
}

#[derive(Clone, Den)]
pub struct UnconnectedPing {
    #[den(with = "Big")]
    pub time: i64,
    #[den(with = "Magic")]
    pub magic: bool,
    #[den(with = "Big")]
    pub client_guid: i64,
}

#[derive(Clone, Den)]
pub struct UnconnectedPong {
    #[den(with = "Big")]
    pub time: i64,
    #[den(with = "Big")]
    pub server_guid: i64,
    #[den(with = "Magic")]
    pub magic: bool,
    #[den(with = "RakString")]
    pub server_id: String,
}

#[derive(Clone, Den)]
pub struct ConnectedPing {
    #[den(with = "Big")]
    time: i64,
}

#[derive(Clone, Den)]
pub struct ConnectedPong {
    #[den(with = "Big")]
    ping_time: i64,
    #[den(with = "Big")]
    pong_time: i64,
}

#[derive(Clone)]
pub struct OpenConnectionRequest1 {
    pub magic: bool,
    pub protocol_version: u8,
    pub zero_padding: usize,
}

impl Den for OpenConnectionRequest1 {
    fn decode(bytes: &mut std::io::Cursor<&[u8]>) -> std::io::Result<Self> {
        Ok(Self {
            magic: <Magic as DenWith<bool>>::decode(bytes)?,
            protocol_version: Den::decode(bytes)?,
            zero_padding: bytes.get_ref().len() - bytes.position() as usize,
        })
    }

    fn encode(&self, bytes: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> {
        <Magic as DenWith<bool>>::encode(&true, bytes)?;
        Den::encode(&self.protocol_version, bytes)?;
        let zero_padding = vec![0u8; self.zero_padding];
        bytes.write_all(&zero_padding)
    }

    fn size(&self) -> usize {
        Magic::size(&true) + 1 + self.zero_padding
    }
}

#[derive(Clone, Den)]
pub struct OpenConnectionReply1 {
    #[den(with = "Magic")]
    pub magic: bool,
    #[den(with = "Big")]
    pub server_guid: i64,
    pub use_security: bool,
    #[den(with = "Big")]
    pub mtu: i16,
}

#[derive(Clone, Den)]
pub struct OpenConnectionRequest2 {
    #[den(with = "Magic")]
    pub magic: bool,
    #[den(with = "RakAddress")]
    pub server_address: SocketAddr,
    #[den(with = "Big")]
    pub mtu: i16,
    #[den(with = "Big")]
    pub client_guid: i64,
}

#[derive(Clone, Den)]
pub struct OpenConnectionReply2 {
    #[den(with = "Magic")]
    pub magic: bool,
    #[den(with = "Big")]
    pub server_guid: i64,
    #[den(with = "RakAddress")]
    pub client_address: SocketAddr,
    #[den(with = "Big")]
    pub mtu: i16,
    pub encrypion_enabled: bool,
}

#[derive(Clone, Den)]
pub struct ConnectionRequest {
    #[den(with = "Big")]
    guid: i64,
    #[den(with = "Big")]
    time: i64,
}

#[derive(Clone)]
pub struct ConnectionRequestAccepted {
    client_address: SocketAddr,
    system_index: i16,
    request_time: i64,
    time: i64,
}

impl Den for ConnectionRequestAccepted {
    fn decode(bytes: &mut std::io::Cursor<&[u8]>) -> std::io::Result<Self> {
        let client_address = <RakAddress as DenWith<SocketAddr>>::decode(bytes)?;
        let system_index = <Big as DenWith<i16>>::decode(bytes)?;
        bytes.set_position(bytes.get_ref().len() as u64 - 16);
        let request_time = <Big as DenWith<i64>>::decode(bytes)?;
        let time = <Big as DenWith<i64>>::decode(bytes)?;
        Ok(Self {
            client_address,
            system_index,
            request_time,
            time,
        })
    }

    fn encode(&self, bytes: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> {
        <RakAddress as DenWith<SocketAddr>>::encode(&self.client_address, bytes)?;
        <Big as DenWith<i16>>::encode(&self.system_index, bytes)?;
        bytes.write_all(&[6u8; 10])?;
        <Big as DenWith<i64>>::encode(&self.request_time, bytes)?;
        <Big as DenWith<i64>>::encode(&self.time, bytes)
    }

    fn size(&self) -> usize {
        <RakAddress as DenWith<SocketAddr>>::size(&self.client_address) + 28
    }
}

#[derive(Clone, Den)]
pub struct NewIncomingConnection {
    #[den(with = "RakAddress")]
    server_address: SocketAddr,
    #[den(with = "RakAddress")]
    internal_address: SocketAddr,
}

#[derive(Clone, Den)]
pub struct IncompatibleProtocolVersion {
    pub server_protocol: u8,
    pub magic: bool,
    #[den(with = "Big")]
    pub server_guid: i64,
}
