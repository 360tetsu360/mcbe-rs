use byte_util::*;
use packet_builder::*;

#[derive(Debug, PartialEq, Den)]
struct Hoge {
    u8: u8,
    i8: i8,
    #[den(with = "Big")]
    u16: u16,
    #[den(with = "Little")]
    u16_le: u16,
    #[den(with = "Big")]
    i16: i16,
    #[den(with = "Little")]
    i16_le: i16,
    #[den(with = "Big")]
    u32: u32,
    #[den(with = "Little")]
    u32_le: u32,
    #[den(with = "Big")]
    i32: i32,
    #[den(with = "Little")]
    i32_le: i32,
    #[den(with = "Big")]
    u64: u64,
    #[den(with = "Little")]
    u64_le: u64,
    #[den(with = "Big")]
    i64: i64,
    #[den(with = "Little")]
    i64_le: i64,
    bool: bool,
}

#[test]
fn den() {
    let hoge = Hoge {
        u8: 0x0F,
        i8: -0x80,
        u16: 0xF000,
        u16_le: 0x000F,
        i16: -0x00F0,
        i16_le: -0x0F00,
        u32: 0xF0000000,
        u32_le: 0x0000000F,
        i32: -0x80000000,
        i32_le: -0x00000008,
        u64: 0xF,
        u64_le: 0xF00000000000000,
        i64: -0x8,
        i64_le: -0x800000000000000,
        bool: false,
    };

    dbg!(hoge);
}
