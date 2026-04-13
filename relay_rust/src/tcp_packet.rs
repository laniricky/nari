use crate::ip_packet::IPv4Packet;

pub struct TCPPacket<'a> {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq: u32,
    pub ack: u32,
    pub data_offset: usize,
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack_flag: bool,
    pub payload: &'a [u8],
}

impl<'a> TCPPacket<'a> {
    pub fn new(raw_data: &'a [u8]) -> Option<Self> {
        if raw_data.len() < 20 {
            return None;
        }

        let src_port = u16::from_be_bytes([raw_data[0], raw_data[1]]);
        let dst_port = u16::from_be_bytes([raw_data[2], raw_data[3]]);
        let seq = u32::from_be_bytes([raw_data[4], raw_data[5], raw_data[6], raw_data[7]]);
        let ack = u32::from_be_bytes([raw_data[8], raw_data[9], raw_data[10], raw_data[11]]);

        let data_offset = ((raw_data[12] >> 4) * 4) as usize;
        
        if raw_data.len() < data_offset {
            return None;
        }

        let flags = raw_data[13];
        let fin = (flags & 0x01) != 0;
        let syn = (flags & 0x02) != 0;
        let rst = (flags & 0x04) != 0;
        let psh = (flags & 0x08) != 0;
        let ack_flag = (flags & 0x10) != 0;

        let payload = &raw_data[data_offset..];

        Some(Self {
            src_port,
            dst_port,
            seq,
            ack,
            data_offset,
            fin,
            syn,
            rst,
            psh,
            ack_flag,
            payload,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        seq: u32,
        ack: u32,
        syn: bool,
        ack_flag: bool,
        psh: bool,
        fin: bool,
        rst: bool,
        window_size: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        let mut header = vec![0u8; 20];
        header[0..2].copy_from_slice(&src_port.to_be_bytes());
        header[2..4].copy_from_slice(&dst_port.to_be_bytes());
        header[4..8].copy_from_slice(&seq.to_be_bytes());
        header[8..12].copy_from_slice(&ack.to_be_bytes());

        header[12] = 5 << 4; // Data offset 5 words

        let mut flags = 0u8;
        if fin { flags |= 0x01; }
        if syn { flags |= 0x02; }
        if rst { flags |= 0x04; }
        if psh { flags |= 0x08; }
        if ack_flag { flags |= 0x10; }
        header[13] = flags;

        header[14..16].copy_from_slice(&window_size.to_be_bytes());

        // Pseudo header inside checksum calculation
        let mut pseudo = vec![0u8; 12];
        pseudo[0..4].copy_from_slice(&src_ip);
        pseudo[4..8].copy_from_slice(&dst_ip);
        pseudo[8] = 0;
        pseudo[9] = 6;
        let tcp_len = (20 + payload.len()) as u16;
        pseudo[10..12].copy_from_slice(&tcp_len.to_be_bytes());

        let mut chksum_data = pseudo;
        chksum_data.extend_from_slice(&header);
        chksum_data.extend_from_slice(payload);

        let chksum = IPv4Packet::calculate_checksum(&chksum_data);
        header[16..18].copy_from_slice(&chksum.to_be_bytes());

        let mut tcp_packet = header;
        tcp_packet.extend_from_slice(payload);
        tcp_packet
    }
}
