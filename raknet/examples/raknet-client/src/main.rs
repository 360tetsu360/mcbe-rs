use raknet::*;
use tokio::{io::AsyncReadExt, select};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (stream, loop_task) = RakStream::connect("127.0.0.1:0").await.unwrap();
    tokio::spawn(loop_task);

    let mut buffer = String::new();
    let mut stdin = tokio::io::stdin();

    let (mut s, mut r) = stream.split();

    loop {
        select! {
            stdin = async { stdin.read_to_string(&mut buffer).await.unwrap(); buffer.clone() } => s.send(stdin.into_bytes()).await,
            received = r.receive() => {
                if let Some(packet) = received {
                    if packet.is_empty() {
                        continue;
                    }

                    if packet[0] == 0xfe {
                        let mut cursor = std::io::Cursor::new(&packet[..]);
                        let text_length = cursor.read_u16().await.unwrap();
                        let mut text_buffer = vec![0u8;text_length as usize];
                        cursor.read_exact(&mut text_buffer).await.unwrap();
                        let text = String::from_utf8(text_buffer).unwrap();
                        println!("msg : {}", text);
                    }
                }
            }
        }
    }
}
