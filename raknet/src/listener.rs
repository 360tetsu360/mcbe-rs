use std::{net::SocketAddr, pin::Pin, sync::Arc};

use async_std::net::{ToSocketAddrs, UdpSocket};
use byte_util::Den;
use futures::{
    channel::{mpsc, oneshot},
    lock::Mutex,
    stream::FuturesUnordered,
    Future, FutureExt, StreamExt,
};

use crate::{
    conn::Conn, loop_task::LoopTask, packets::*, RakStream, StreamInformation,
    RAKNET_PROTOCOL_VERSION,
};

pub struct Listener {
    guid: i64,
    server_id: Arc<Mutex<String>>,
    raw_socket: Arc<UdpSocket>,
    destroy_sender: oneshot::Sender<Destroy>,
    new_stream_receiver: mpsc::Receiver<(RakStream, StreamInformation)>,
}

impl Listener {
    pub async fn bind<A: ToSocketAddrs>(
        addrs: A,
        guid: i64,
        server_id: &str,
    ) -> std::io::Result<(Self, LoopTask)> {
        let raw_socket = Arc::new(UdpSocket::bind(addrs).await?);
        let (destroy_sender, destroy_receiver) = oneshot::channel();
        let (new_stream_sender, new_stream_receiver) = mpsc::channel(8);
        let socket = raw_socket.clone();
        let server_id = Arc::new(Mutex::new(server_id.to_owned()));
        let server_id_cloned = server_id.clone();
        let server_loop_task = LoopTask {
            task: listener_loop(
                guid,
                server_id_cloned,
                socket,
                destroy_receiver,
                new_stream_sender,
            )
            .boxed(),
        };

        Ok((
            Self {
                guid,
                server_id,
                raw_socket,
                destroy_sender,
                new_stream_receiver,
            },
            server_loop_task,
        ))
    }

    pub fn destroy(self) {
        _ = self.destroy_sender.send(Destroy);
    }

    pub async fn accept(&mut self) -> Option<(RakStream, StreamInformation)> {
        self.new_stream_receiver.next().await
    }

    pub fn raw_socket(&self) -> Arc<UdpSocket> {
        self.raw_socket.clone()
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.raw_socket.local_addr()
    }

    pub async fn set_server_id(&self, new_server_id: &str) {
        *self.server_id.lock().await = new_server_id.to_owned()
    }

    pub async fn server_id(&self) -> String {
        self.server_id.lock().await.clone()
    }

    pub fn guid(&self) -> i64 {
        self.guid
    }
}

struct Destroy;

enum TaskResultWapper {
    Destroy,
    UdpReceived(std::io::Result<(usize, SocketAddr)>, Box<[u8; 4096]>),
}

type TaskManager = FuturesUnordered<Pin<Box<dyn Future<Output = TaskResultWapper> + Send>>>;

pub struct ConnectionManager {}

async fn listener_loop(
    guid: i64,
    server_id: Arc<Mutex<String>>,
    socket: Arc<UdpSocket>,
    destroy_receiver: oneshot::Receiver<Destroy>,
    mut new_stream_sender: mpsc::Sender<(RakStream, StreamInformation)>,
) {
    let tasks = Arc::new(Mutex::new(TaskManager::new()));
    let destroy_task = async move {
        _ = destroy_receiver.await;
        TaskResultWapper::Destroy
    }
    .boxed();
    let socket_clone = socket.clone();
    let receive_udp_task = async move {
        let mut buffer = [0u8; 4096];
        TaskResultWapper::UdpReceived(socket_clone.recv_from(&mut buffer).await, Box::new(buffer))
    }
    .boxed();

    tasks.lock().await.push(destroy_task);
    tasks.lock().await.push(receive_udp_task);

    let connection_manager = ConnectionManager {};

    while let Some(result) = tasks.lock().await.next().await {
        match result {
            TaskResultWapper::Destroy => {
                // Do something to end connections
                break;
            }
            TaskResultWapper::UdpReceived(res, mut buffer) => {
                // handle packet
                if let Err(_) = res {
                    todo!();
                }

                let (size, addr) = res.unwrap();
                let tasks_new = tasks.clone();
                handle_packet(
                    tasks_new,
                    &connection_manager,
                    addr,
                    &buffer[..size],
                    &socket.clone(),
                    guid,
                    server_id.clone(),
                    &mut new_stream_sender,
                )
                .await;

                let socket_clone = socket.clone();
                let receive_udp_task = async move {
                    TaskResultWapper::UdpReceived(
                        socket_clone.recv_from(buffer.as_mut_slice()).await,
                        buffer,
                    )
                }
                .boxed();
                tasks.lock().await.push(receive_udp_task)
            }
        }
    }
}

macro_rules! or_return {
    ($result:expr) => {
        match $result {
            Ok(p) => p,
            Err(_) => return,
        }
    };
}

async fn handle_packet(
    tasks: Arc<Mutex<TaskManager>>,
    connection_manager: &ConnectionManager,
    addr: SocketAddr,
    buffer: &[u8],
    socket: &Arc<UdpSocket>,
    guid: i64,
    server_id: Arc<Mutex<String>>,
    new_stream_sender: &mut mpsc::Sender<(RakStream, StreamInformation)>,
) {
    if buffer.is_empty() {
        return;
    }

    match buffer[0] {
        0x1 | 0x2 => {
            let ping = or_return!(decode::<UnconnectedPing>(buffer));
            let pong = UnconnectedPong {
                time: ping.time,
                server_guid: guid,
                magic: true,
                server_id: server_id.lock().await.clone(),
            };
            socket
                .send_to(&encode(pong, 0x1c).unwrap(), addr)
                .await
                .unwrap();
        }
        0x5 => {
            let openconnectionrequest1 = or_return!(decode::<OpenConnectionRequest1>(buffer));

            if openconnectionrequest1.protocol_version != RAKNET_PROTOCOL_VERSION {
                let incompatibleprotocolversion = IncompatibleProtocolVersion {
                    server_protocol: RAKNET_PROTOCOL_VERSION,
                    magic: true,
                    server_guid: guid,
                };
                socket
                    .send_to(&encode(incompatibleprotocolversion, 0x19).unwrap(), addr)
                    .await
                    .unwrap();
                return;
            }

            let openconnectionreply1 = OpenConnectionReply1 {
                magic: true,
                server_guid: guid,
                use_security: false,
                mtu: openconnectionrequest1.size() as i16,
            };
            socket
                .send_to(&encode(openconnectionreply1, 0x6).unwrap(), addr)
                .await
                .unwrap();
        }
        0x7 => {
            let openconnectionrequest2 = or_return!(decode::<OpenConnectionRequest2>(buffer));
            let openconnectionreply2 = OpenConnectionReply2 {
                magic: true,
                server_guid: guid,
                client_address: addr,
                mtu: openconnectionrequest2.mtu,
                encrypion_enabled: false,
            };
            socket
                .send_to(&encode(openconnectionreply2, 0x8).unwrap(), addr)
                .await
                .unwrap();

            let (to_stream_sender, to_stream_receiver) = mpsc::channel(8);
            let (to_conn_sender, to_conn_receiver) = mpsc::channel(8);

            let conn =
                Conn::incoming_connection(socket.clone(), to_stream_sender, to_conn_receiver);
            /*let stream = RakStream {
                msg_receiver: to_stream_receiver,
                msg_sender: to_conn_sender,
            };
            let info = StreamInformation {
                guid,
                address: addr,
            };

            new_stream_sender.send((stream, info)).await.unwrap();*/
        }

        _ => {}
    }
}
