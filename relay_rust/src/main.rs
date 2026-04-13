mod ip_packet;
mod tcp_packet;
mod udp_packet;
mod connection;

use tokio::net::{TcpListener, UdpSocket};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use std::collections::HashMap;

use ip_packet::IPv4Packet;
use tcp_packet::TCPPacket;
use udp_packet::UDPPacket;
use connection::TCPConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:4242").await?;
    println!("Rust Relay serving on {} - run \"adb reverse tcp:4242 tcp:4242\" to bridge.", listener.local_addr()?);

    loop {
        let (socket, _addr) = listener.accept().await?;
        println!("Android client connected!");

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Client handler error: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let (mut reader, mut writer) = socket.into_split();
    
    // Channel for writing back to Android client
    let (to_android_tx, mut to_android_rx) = mpsc::channel::<Vec<u8>>(1024);
    
    // Spawn writer task
    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        while let Some(data) = to_android_rx.recv().await {
            let len = (data.len() as u32).to_be_bytes();
            if writer.write_all(&len).await.is_err() {
                break;
            }
            if writer.write_all(&data).await.is_err() {
                break;
            }
        }
    });

    let mut connections = HashMap::new();

    loop {
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).await.is_err() {
            break; // EOF or error
        }
        let length = u32::from_be_bytes(len_buf) as usize;

        let mut packet_buf = vec![0u8; length];
        if reader.read_exact(&mut packet_buf).await.is_err() {
            break;
        }

        if let Some(ip) = IPv4Packet::new(&packet_buf) {
            if ip.protocol == 6 { // TCP
                if let Some(tcp) = TCPPacket::new(ip.payload) {
                    let conn_key = (ip.src_ip, ip.dst_ip, tcp.src_port, tcp.dst_port);
                    
                    if !connections.contains_key(&conn_key) {
                        let (client_tx, client_rx) = mpsc::channel::<Vec<u8>>(1024);
                        
                        let conn = TCPConnection::new(
                            ip.src_ip, 
                            ip.dst_ip, 
                            tcp.src_port, 
                            tcp.dst_port, 
                            to_android_tx.clone()
                        );
                        
                        println!("New connection to {:?}:{}", ip.dst_ip, tcp.dst_port);
                        
                        tokio::spawn(async move {
                            conn.run(client_rx).await;
                        });
                        
                        connections.insert(conn_key, client_tx);
                    }

                    if let Some(tx) = connections.get(&conn_key) {
                        // send TCP packet to connection actor
                        let _ = tx.send(ip.payload.to_vec()).await;
                    }
                }
            } else if ip.protocol == 17 { // UDP
                if let Some(udp) = UDPPacket::new(ip.payload) {
                    if udp.dst_port == 53 {
                        // DNS Proxy
                        let src_ip = ip.src_ip;
                        let dst_ip = ip.dst_ip;
                        let src_port = udp.src_port;
                        let payload = udp.payload.to_vec();
                        let tx = to_android_tx.clone();

                        tokio::spawn(async move {
                            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                                if socket.send_to(&payload, "8.8.8.8:53").await.is_ok() {
                                    let mut buf = [0u8; 2048];
                                    if let Ok((len, _)) = socket.recv_from(&mut buf).await {
                                        let resp_payload = &buf[..len];
                                        
                                        let udp_resp = UDPPacket::build(
                                            dst_ip, src_ip, 
                                            53, src_port, 
                                            resp_payload
                                        );
                                        
                                        let ip_resp = IPv4Packet::build(
                                            dst_ip, src_ip, 
                                            17, &udp_resp
                                        );
                                        
                                        let _ = tx.send(ip_resp).await;
                                    }
                                }
                            }
                        });
                    } else {
                        // println!("Dropped non-DNS UDP packet to port {}", udp.dst_port);
                    }
                }
            } else {
                // println!("Dropped non-TCP packet: protocol {}", ip.protocol);
            }
        }
    }
    
    println!("Android client disconnected.");
    Ok(())
}
