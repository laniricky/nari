use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use std::net::{Ipv4Addr, SocketAddr};
use rand::Rng;
use crate::ip_packet::IPv4Packet;
use crate::tcp_packet::TCPPacket;

pub struct TCPConnection {
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
    pub src_port: u16,
    pub dst_port: u16,

    pub client_seq: u32,
    pub server_seq: u32,

    pub tx_to_client: mpsc::Sender<Vec<u8>>,
}

impl TCPConnection {
    pub fn new(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        tx_to_client: mpsc::Sender<Vec<u8>>,
    ) -> Self {
        Self {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            client_seq: 0,
            server_seq: rand::random::<u32>(),
            tx_to_client,
        }
    }

    pub async fn run(mut self, mut rx_from_client: mpsc::Receiver<Vec<u8>>) {
        let addr = SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::new(
                self.dst_ip[0], self.dst_ip[1], self.dst_ip[2], self.dst_ip[3],
            )),
            self.dst_port,
        );

        let mut stream = match TcpStream::connect(addr).await {
            Ok(s) => s,
            Err(_) => {
                self.send_rst().await;
                return;
            }
        };

        // Expect the first packet from the client to be the SYN packet
        if let Some(pkt_data) = rx_from_client.recv().await {
            if let Some(tcp) = TCPPacket::new(&pkt_data) {
                if tcp.syn {
                    self.client_seq = tcp.seq.wrapping_add(1);
                    self.send_syn_ack().await;
                }
            }
        }

        let (mut reader, mut writer) = stream.into_split();
        let mut remote_buf = vec![0u8; 8192];

        loop {
            tokio::select! {
                client_data_opt = rx_from_client.recv() => {
                    match client_data_opt {
                        Some(pkt_data) => {
                            if let Some(tcp) = TCPPacket::new(&pkt_data) {
                                if !tcp.payload.is_empty() {
                                    self.client_seq = tcp.seq.wrapping_add(tcp.payload.len() as u32);
                                    let _ = writer.write_all(tcp.payload).await;
                                    self.send_ack().await;
                                }

                                if tcp.fin {
                                    self.client_seq = tcp.seq.wrapping_add(1);
                                    self.send_ack().await;
                                    self.send_fin().await;
                                    break;
                                }
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
                remote_read_res = reader.read(&mut remote_buf) => {
                    match remote_read_res {
                        Ok(n) if n > 0 => {
                            self.send_data(&remote_buf[..n]).await;
                        }
                        _ => {
                            self.send_fin().await;
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn send_packet(&mut self, syn: bool, ack_flag: bool, psh: bool, fin: bool, rst: bool, payload: &[u8]) {
        let tcp_data = TCPPacket::build(
            self.dst_ip, self.src_ip,
            self.dst_port, self.src_port,
            self.server_seq, self.client_seq,
            syn, ack_flag, psh, fin, rst,
            65535, payload,
        );

        let ip_data = IPv4Packet::build(
            self.dst_ip, self.src_ip, 6, &tcp_data,
        );

        let _ = self.tx_to_client.send(ip_data).await;

        if !payload.is_empty() {
            self.server_seq = self.server_seq.wrapping_add(payload.len() as u32);
        }
        if syn || fin {
            self.server_seq = self.server_seq.wrapping_add(1);
        }
    }

    async fn send_syn_ack(&mut self) {
        self.send_packet(true, true, false, false, false, &[]).await;
    }

    async fn send_ack(&mut self) {
        self.send_packet(false, true, false, false, false, &[]).await;
    }

    async fn send_data(&mut self, data: &[u8]) {
        self.send_packet(false, true, true, false, false, data).await;
    }

    async fn send_fin(&mut self) {
        self.send_packet(false, true, false, true, false, &[]).await;
    }

    async fn send_rst(&mut self) {
        self.send_packet(false, true, false, false, true, &[]).await;
    }
}
