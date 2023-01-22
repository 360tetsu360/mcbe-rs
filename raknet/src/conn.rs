use std::sync::Arc;

use async_std::net::UdpSocket;
use futures::channel::mpsc;

pub enum ToStreamMsg {
    Packet(Vec<u8>),
}

pub enum ToConnMsg {
    Send(Vec<u8>),
    Disconnect,
}

enum ConnType {
    Incoming,
    Outgoing,
}

enum ConnStatus {
    Connecting(ConnectStatus),
}

enum ConnectStatus {
    WaitingConnectionRequest,
    WaitingConnectionRequestAccepted,
    WaitingNewIncomingConnection,
}

pub struct Conn {
    socket: Arc<UdpSocket>,
    msg_sender: mpsc::Sender<ToStreamMsg>,
    msg_receiver: mpsc::Receiver<ToConnMsg>,
    conn_type: ConnType,
    status: ConnStatus,
}

impl Conn {
    pub fn incoming_connection(
        socket: Arc<UdpSocket>,
        msg_sender: mpsc::Sender<ToStreamMsg>,
        msg_receiver: mpsc::Receiver<ToConnMsg>,
    ) -> Self {
        Self {
            socket,
            msg_sender,
            msg_receiver,
            conn_type: ConnType::Incoming,
            status: ConnStatus::Connecting(ConnectStatus::WaitingConnectionRequest),
        }
    }

    pub fn connect() -> Self {
        todo!()
    }

    pub async fn handle(&mut self, buffer: &[u8]) {}
}
