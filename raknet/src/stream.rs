use std::net::SocketAddr;

use async_std::net::{ToSocketAddrs, UdpSocket};
use futures::{channel::mpsc, SinkExt, StreamExt};

use crate::{
    conn::{ToConnMsg, ToStreamMsg},
    loop_task::LoopTask,
};

pub struct RakStream {
    pub(crate) msg_receiver: mpsc::Receiver<ToStreamMsg>,
    pub(crate) msg_sender: mpsc::Sender<ToConnMsg>,
}

impl RakStream {
    pub async fn connect<A: ToSocketAddrs>(addrs: A) -> std::io::Result<(Self, LoopTask)> {
        let socket = UdpSocket::bind(addrs).await?;

        todo!();
    }

    pub async fn receive(&mut self) -> Option<Vec<u8>> {
        self.msg_receiver.next().await.map(|msg| {
            let ToStreamMsg::Packet(packet) = msg;
            packet
        })
    }

    pub async fn send(&mut self, bytes: Vec<u8>) {
        self.msg_sender.send(ToConnMsg::Send(bytes)).await.unwrap();
    }

    pub fn split(self) -> (RakStreamSender, RakStreamReceiver) {
        (
            RakStreamSender {
                msg_sender: self.msg_sender,
            },
            RakStreamReceiver {
                msg_receiver: self.msg_receiver,
            },
        )
    }

    pub fn disconnect(self) {}
}

#[derive(Clone)]
pub struct RakStreamSender {
    msg_sender: mpsc::Sender<ToConnMsg>,
}

impl RakStreamSender {
    pub async fn send(&mut self, bytes: Vec<u8>) {
        self.msg_sender.send(ToConnMsg::Send(bytes)).await.unwrap();
    }

    pub fn disconnect(self) {}
}

pub struct RakStreamReceiver {
    msg_receiver: mpsc::Receiver<ToStreamMsg>,
}

impl RakStreamReceiver {
    pub async fn receive(&mut self) -> Option<Vec<u8>> {
        self.msg_receiver.next().await.map(|msg| {
            let ToStreamMsg::Packet(packet) = msg;
            packet
        })
    }
}

#[derive(Debug, Clone)]
pub struct StreamInformation {
    pub guid: i64,
    pub address: SocketAddr,
}
