use crate::ip_packet::IPv4Packet;

pub struct UDPPacket<'a> {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
    pub payload: &'a [u8],
}

impl<'a> UDPPacket<'a> {
    pub fn new(raw_data: &'a [u8]) -> Option<Self> {
        if raw_data.len() < 8 {
            return None;
        }

        let src_port = u16::from_be_bytes([raw_data[0], raw_data[1]]);
        let dst_port = u16::from_be_bytes([raw_data[2], raw_data[3]]);
        let length = u16::from_be_bytes([raw_data[4], raw_data[5]]);
        let checksum = u16::from_be_bytes([raw_data[6], raw_data[7]]);

        if raw_data.len() < length as usize {
            return None;
        }

        let payload = &raw_data[8..length as usize];

        Some(Self {
            src_port,
            dst_port,
            length,
            checksum,
            payload,
        })
    }

    pub fn build(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        let mut header = vec![0u8; 8];
        header[0..2].copy_from_slice(&src_port.to_be_bytes());
        header[2..4].copy_from_slice(&dst_port.to_be_bytes());
        
        let length = (8 + payload.len()) as u16;
        header[4..6].copy_from_slice(&length.to_be_bytes());
        
        // Zero out checksum primarily, wait we have to compute it
        header[6..8].copy_from_slice(&0u16.to_be_bytes());

        // Pseudo header inside checksum calculation
        let mut pseudo = vec![0u8; 12];
        pseudo[0..4].copy_from_slice(&src_ip);
        pseudo[4..8].copy_from_slice(&dst_ip);
        pseudo[8] = 0;
        pseudo[9] = 17; // UDP protocol index
        pseudo[10..12].copy_from_slice(&length.to_be_bytes());

        let mut chksum_data = pseudo;
        chksum_data.extend_from_slice(&header);
        chksum_data.extend_from_slice(payload);

        let mut chksum = IPv4Packet::calculate_checksum(&chksum_data);
        if chksum == 0 {
            chksum = 0xFFFF; // Per RFC, 0 means no checksum, so we must transmit all-ones
        }

        header[6..8].copy_from_slice(&chksum.to_be_bytes());

        let mut udp_packet = header;
        udp_packet.extend_from_slice(payload);
        udp_packet
    }
}
