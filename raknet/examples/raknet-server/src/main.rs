use std::io::Cursor;

use raknet::*;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (mut listener, loop_task) = Listener::bind("127.0.0.1:19132", 0, "raknet_server!").await?;
    tokio::spawn(loop_task);

    while let Some((stream, info)) = listener.accept().await {
        tokio::spawn(handle(stream, info));
    }
    Ok(())
}

async fn handle(stream: RakStream, info: StreamInformation) {
    let (s, mut r) = stream.split();
    while let Some(packet) = r.receive().await {
        if packet.is_empty() {
            continue;
        }

        if packet[0] == 0xfe {
            let mut cursor = Cursor::new(&packet[..]);
            let text_length = cursor.read_u16().await.unwrap();
            if text_length == 0 {
                s.disconnect();
                break;
            }
            let mut text_buffer = vec![0u8; text_length as usize];
            cursor.read_exact(&mut text_buffer).await.unwrap();
            let text = String::from_utf8(text_buffer).unwrap();
            println!("from : {}, msg : {}", info.address, text);
        }
    }
    todo!();
}
