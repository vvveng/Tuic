use super::UdpRelayMode;
use futures_util::StreamExt;
use quinn::{ConnectionError, Datagrams, IncomingUniStreams};
use std::sync::Arc;
use tokio::sync::oneshot::{self, Receiver, Sender};

pub async fn listen_incoming(
    udp_relay_mode: UdpRelayMode,
    mut dg_next: NextDatagrams,
    mut uni_next: NextIncomingUniStreams,
) {
    match udp_relay_mode {
        UdpRelayMode::Native => loop {
            let (mut datagrams, even_next) = dg_next.next().await;
            dg_next = even_next;

            let err = loop {
                let pkt = if let Some(pkt) = datagrams.next().await {
                    match pkt {
                        Ok(pkt) => pkt,
                        Err(err) => break err,
                    }
                } else {
                    break ConnectionError::LocallyClosed;
                };

                // TODO: process datagram
                tokio::spawn(async move {});
            };

            match err {
                ConnectionError::LocallyClosed | ConnectionError::TimedOut => {}
                err => log::error!("[relay] [connection] {err}"),
            }

            // TODO: close connection
        },
        UdpRelayMode::Quic => loop {
            let (mut uni_streams, even_next) = uni_next.next().await;
            uni_next = even_next;

            let err = loop {
                let stream = if let Some(stream) = uni_streams.next().await {
                    match stream {
                        Ok(stream) => stream,
                        Err(err) => break err,
                    }
                } else {
                    break ConnectionError::LocallyClosed;
                };

                // TODO: process stream
                tokio::spawn(async move {});
            };

            match err {
                ConnectionError::LocallyClosed | ConnectionError::TimedOut => {}
                err => log::error!("[relay] [connection] {err}"),
            }

            // TODO: close connection
        },
    }
}

pub struct NextIncomingUniStreams(Receiver<(IncomingUniStreams, NextIncomingUniStreams)>);
pub type NextIncomingUniStreamsSender = Sender<(IncomingUniStreams, NextIncomingUniStreams)>;

impl NextIncomingUniStreams {
    pub fn new() -> (Self, NextIncomingUniStreamsSender) {
        let (tx, rx) = oneshot::channel();
        (Self(rx), tx)
    }

    async fn next(self) -> (IncomingUniStreams, Self) {
        self.0.await.unwrap() // safety: the current task that waiting new incoming will be cancelled if the sender's scope is dropped
    }
}

pub struct NextDatagrams(Receiver<(Datagrams, NextDatagrams)>);
pub type NextDatagramsSender = Sender<(Datagrams, NextDatagrams)>;

impl NextDatagrams {
    pub fn new() -> (Self, NextDatagramsSender) {
        let (tx, rx) = oneshot::channel();
        (Self(rx), tx)
    }

    async fn next(self) -> (Datagrams, Self) {
        self.0.await.unwrap() // safety: the current task that waiting new incoming will be cancelled if the sender's scope is dropped
    }
}
