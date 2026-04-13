pub struct IPv4Packet<'a> {
    pub version: u8,
    pub ihl: usize,
    pub total_length: u16,
    pub protocol: u8,
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
    pub payload: &'a [u8],
}

impl<'a> IPv4Packet<'a> {
    pub fn new(raw_data: &'a [u8]) -> Option<Self> {
        if raw_data.len() < 20 {
            return None;
        }

        let version_ihl = raw_data[0];
        let version = version_ihl >> 4;
        let ihl = ((version_ihl & 0xF) * 4) as usize;

        let total_length = u16::from_be_bytes([raw_data[2], raw_data[3]]);
        let protocol = raw_data[9];

        let mut src_ip = [0u8; 4];
        src_ip.copy_from_slice(&raw_data[12..16]);
        let mut dst_ip = [0u8; 4];
        dst_ip.copy_from_slice(&raw_data[16..20]);

        if raw_data.len() < total_length as usize {
            return None;
        }

        let payload = &raw_data[ihl..total_length as usize];

        Some(Self {
            version,
            ihl,
            total_length,
            protocol,
            src_ip,
            dst_ip,
            payload,
        })
    }

    pub fn calculate_checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        while i < data.len() {
            let word = if i + 1 < data.len() {
                u16::from_be_bytes([data[i], data[i + 1]]) as u32
            } else {
                u16::from_be_bytes([data[i], 0]) as u32
            };
            sum += word;
            i += 2;
        }

        while (sum >> 16) > 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }

        !(sum as u16)
    }

    pub fn build(src_ip: [u8; 4], dst_ip: [u8; 4], protocol: u8, payload: &[u8]) -> Vec<u8> {
        let mut header = vec![0u8; 20];
        header[0] = 0x45; // Version 4, IHL 5
        header[1] = 0;

        let total_len = (20 + payload.len()) as u16;
        header[2..4].copy_from_slice(&total_len.to_be_bytes());
        header[4..6].copy_from_slice(&0u16.to_be_bytes()); // ID
        header[6..8].copy_from_slice(&0x4000u16.to_be_bytes()); // Flags: Don't fragment
        header[8] = 64; // TTL
        header[9] = protocol;
        header[12..16].copy_from_slice(&src_ip);
        header[16..20].copy_from_slice(&dst_ip);

        let chksum = Self::calculate_checksum(&header);
        header[10..12].copy_from_slice(&chksum.to_be_bytes());

        let mut packet = header;
        packet.extend_from_slice(payload);
        packet
    }
}
