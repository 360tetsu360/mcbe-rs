use byte_util::{Big, Den, DenWith, Little};
use std::{
    io::{Cursor, Error, ErrorKind, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

pub struct U24;

impl DenWith<u32> for U24 {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<u32> {
        let mut buf = [0; 3];
        bytes.read_exact(&mut buf).unwrap();
        Ok(((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32))
    }

    fn encode(v: &u32, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
        let buf = [(*v >> 16) as u8, (*v >> 8) as u8, *v as u8];
        bytes.write_all(&buf)
    }

    fn size(_: &u32) -> usize {
        3
    }
}

pub struct RakString;

impl DenWith<String> for RakString {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<String> {
        let str_len = <Big as DenWith<u16>>::decode(bytes)?;
        let mut buffer = vec![0u8; str_len as usize];
        bytes.read_exact(&mut buffer)?;
        String::from_utf8(buffer)
            .map_err(|utferr| Error::new(ErrorKind::InvalidData, utferr.to_string()))
    }

    fn encode(v: &String, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
        let buffer = v.as_bytes();
        <Big as DenWith<u16>>::encode(&(buffer.len() as u16), bytes)?;
        bytes.write_all(buffer)
    }

    fn size(v: &String) -> usize {
        2 + v.len()
    }
}

pub struct Magic;
const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

impl DenWith<bool> for Magic {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<bool> {
        let mut buffer = [0u8; 16];
        bytes.read_exact(&mut buffer)?;
        Ok(buffer.eq(&MAGIC))
    }

    fn encode(_: &bool, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
        bytes.write_all(&MAGIC)
    }

    fn size(_: &bool) -> usize {
        16
    }
}

pub struct RakAddress;

impl DenWith<SocketAddr> for RakAddress {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<SocketAddr> {
        let ip_ver: u8 = Den::decode(bytes)?;
        if ip_ver == 4 {
            let ip = Ipv4Addr::new(
                0xff - <u8 as Den>::decode(bytes)?,
                0xff - <u8 as Den>::decode(bytes)?,
                0xff - <u8 as Den>::decode(bytes)?,
                0xff - <u8 as Den>::decode(bytes)?,
            );
            let port = <Big as DenWith<u16>>::decode(bytes)?;
            Ok(SocketAddr::new(IpAddr::V4(ip), port))
        } else {
            bytes.set_position(bytes.position() + 2);
            let port = <Little as DenWith<u16>>::decode(bytes)?;
            bytes.set_position(bytes.position() + 4);
            let mut addr_buf = [0; 16];
            bytes.read_exact(&mut addr_buf)?;

            let mut address_cursor = Cursor::new(&addr_buf[..]);
            bytes.set_position(bytes.position() + 4);
            Ok(SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(
                    <Big as DenWith<u16>>::decode(&mut address_cursor)?,
                    <Big as DenWith<u16>>::decode(&mut address_cursor)?,
                    <Big as DenWith<u16>>::decode(&mut address_cursor)?,
                    <Big as DenWith<u16>>::decode(&mut address_cursor)?,
                    <Big as DenWith<u16>>::decode(&mut address_cursor)?,
                    <Big as DenWith<u16>>::decode(&mut address_cursor)?,
                    <Big as DenWith<u16>>::decode(&mut address_cursor)?,
                    <Big as DenWith<u16>>::decode(&mut address_cursor)?,
                )),
                port,
            ))
        } //IPv6 address = 128bit = u8 * 16
    }

    fn encode(v: &SocketAddr, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
        if v.is_ipv4() {
            Den::encode(&4u8, bytes)?;
            let ip_bytes = match v.ip() {
                IpAddr::V4(ip) => ip.octets().to_vec(),
                _ => vec![0; 4],
            };

            Den::encode(&(0xff - ip_bytes[0]), bytes)?;
            Den::encode(&(0xff - ip_bytes[1]), bytes)?;
            Den::encode(&(0xff - ip_bytes[2]), bytes)?;
            Den::encode(&(0xff - ip_bytes[3]), bytes)?;
            <Big as DenWith<u16>>::encode(&v.port(), bytes)?;
            Ok(())
        } else {
            <Little as DenWith<i16>>::encode(&23, bytes)?;
            <Big as DenWith<u16>>::encode(&v.port(), bytes)?;
            <Big as DenWith<i32>>::encode(&0, bytes)?;
            let ip_bytes = match v.ip() {
                IpAddr::V6(ip) => ip.octets(),
                _ => unreachable!(),
            };
            bytes.write_all(&ip_bytes)?;
            <Big as DenWith<i32>>::encode(&0, bytes)?;
            Ok(())
        }
    }

    fn size(v: &SocketAddr) -> usize {
        match v {
            SocketAddr::V4(_) => 6,
            SocketAddr::V6(_) => 28,
        }
    }
}
