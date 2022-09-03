use std::any::Any;
use std::io::Read;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use boringtun::noise::handshake::Handshake;
use protobuf::{EnumOrUnknown, Message};
use tokio::{io, select, time};
use tokio::net::{UdpSocket, ToSocketAddrs};
use tokio::sync::{Notify};
use tokio_util::codec::BytesCodec;
use tokio_util::udp::UdpFramed;
use shared::message_proto::{mesh_client_message, mesh_control, mesh_meta, MeshClientMessage, MeshControl, MeshHandshake, MeshMeta, MeshPing};
use shared::message_proto::mesh_ping::MessageType;

type MeshResult<T, E = anyhow::Error> = anyhow::Result<T, E>;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum NatType {
    UNKNOWN = 0,
    ASYMMETRIC = 1,
    SYMMETRIC = 2,
}

pub struct MeshMediator {
    shutdown: Arc<Notify>,
}

impl Default for MeshMediator {
    fn default() -> Self {
        MeshMediator {
            shutdown: Arc::new(Notify::new()),
        }
    }
}

impl MeshMediator {
    async fn start<A: ToSocketAddrs>(&self, listen_addr: A, server_addr: A) -> io::Result<()> {
        // 1. 创建端口监听, 连接服务器
        // 2. 循环读取端口消息
        // 3. 处理消息
        // 4. 定期 ping
        let socket = Arc::new(UdpSocket::bind(listen_addr).await?);

        let listen_socket = socket.clone();
        let mut interval = time::interval(time::Duration::from_secs(2));
        let mut buffer = [0u8; 2048];
        let shutdown = self.shutdown.clone();

        loop {
            select! {
                    result = listen_socket.recv_from(&mut buffer) => {
                        match result {
                            Ok((len, from_addr)) => {
                                self.process_message(&listen_socket, &server_addr, &buffer[..len]);
                            },
                            Err(e) => {

                            }
                        }
                    },
                    _ = interval.tick() => {
                        self.ping(&listen_socket, &server_addr);
                    },
                    _ = shutdown.notified() => {

                        return Ok(());
                    }
                }
        }

        Ok(())
    }

    pub fn shutdown(&self) {
        self.shutdown.notify_one();
    }

    fn process_message(&self, sock: &UdpSocket, server_addr: &impl ToSocketAddrs, data: &[u8]) {
        if let Ok(meshMessage) = MeshClientMessage::parse_from_bytes(data) {
            match meshMessage.message {
                Some(mesh_client_message::Message::MeshMeta(meta)) => self.process_mesh_meta_message(sock, server_addr, meta),
                Some(mesh_client_message::Message::Handshake(handshake)) => self.process_handshake(sock, server_addr, handshake),
                Some(mesh_client_message::Message::Control(control)) => self.process_control(sock, server_addr, control),
                _ => unreachable!()
            }
        }
    }

    fn process_control(&self, sock: &UdpSocket, server_addr: &impl ToSocketAddrs, control: MeshControl) {
        match control.Type.enum_value() {
            Ok(mesh_control::MessageType::CreateRelayResponse) => {}
            _ => {}
        }
    }

    fn process_mesh_meta_message(&self, sock: &UdpSocket, server_addr: &impl ToSocketAddrs, meta: MeshMeta) {
        match meta.Type.enum_value() {
            Ok(mesh_meta::MessageType::HostQuery) => {}
            Ok(mesh_meta::MessageType::HostUpdateNotification) => {}
            Ok(mesh_meta::MessageType::HostMovedNotification) => {}
            Ok(mesh_meta::MessageType::HostPunchNotification) => {}
            Ok(mesh_meta::MessageType::HostWhoami) => {}
            Ok(mesh_meta::MessageType::PathCheck) => {}
            _ => {}
        }
    }

    fn process_handshake(&self, sock: &UdpSocket, server_addr: &impl ToSocketAddrs, handshake: MeshHandshake) {}

    fn ping(&self, sock: &UdpSocket, server_addr: &impl ToSocketAddrs) -> MeshResult<()> {
        let ping = MeshPing {
            Time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            ..Default::default()
        };

        let data = bytes::Bytes::from(ping.write_to_bytes()?);

        sock.send_to(&data, server_addr);

        Ok(())
    }
}

#[cfg(test)]
mod Test {
    #[test]
    fn it_works() {
        println!("it works!")
    }
}