use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

pub trait Den {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<Self>
    where
        Self: Sized;
    fn encode(&self, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()>;
    fn size(&self) -> usize;
}

pub trait DenWith<T> {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<T>
    where
        T: Sized;
    fn encode(v: &T, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()>;
    fn size(v: &T) -> usize;
}

impl Den for u8 {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        bytes.read_u8()
    }

    fn encode(&self, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
        bytes.write_u8(*self)
    }

    fn size(&self) -> usize {
        1
    }
}

impl Den for i8 {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        bytes.read_i8()
    }

    fn encode(&self, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
        bytes.write_i8(*self)
    }

    fn size(&self) -> usize {
        1
    }
}

impl Den for bool {
    fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        Ok(bytes.read_u8()? != 0)
    }

    fn encode(&self, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
        bytes.write_u8(*self as u8)
    }

    fn size(&self) -> usize {
        1
    }
}

pub struct Big;
pub struct Little;

macro_rules! num {
    ($number_type:path, $byte_read_expr:ident, $byte_write_expr:ident, $size:expr) => {
        impl DenWith<$number_type> for Big {
            fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<$number_type> {
                bytes.$byte_read_expr::<BigEndian>()
            }

            fn encode(v: &$number_type, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
                bytes.$byte_write_expr::<BigEndian>(*v)
            }

            fn size(_: &$number_type) -> usize {
                $size
            }
        }

        impl DenWith<$number_type> for Little {
            fn decode(bytes: &mut Cursor<&[u8]>) -> std::io::Result<$number_type> {
                bytes.$byte_read_expr::<LittleEndian>()
            }

            fn encode(v: &$number_type, bytes: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
                bytes.$byte_write_expr::<LittleEndian>(*v)
            }

            fn size(_: &$number_type) -> usize {
                $size
            }
        }
    };
}

num!(i16, read_i16, write_i16, 2);
num!(u16, read_u16, write_u16, 2);
num!(i32, read_i32, write_i32, 4);
num!(u32, read_u32, write_u32, 4);
num!(i64, read_i64, write_i64, 8);
num!(u64, read_u64, write_u64, 8);
