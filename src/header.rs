#[derive(Debug)]
pub struct Header {
    pub ident: u16,
    pub query: bool,
    pub opcode: u8, // TODO: enum
    pub authoritative: bool,
    pub truncated: bool,
    pub recursion_desired: bool,
    pub recursion_avail: bool,
    pub reserved: u8,
    pub rcode: u8, // TODO: enum
    pub qd_count: u16,
    pub an_count: u16,
    pub authority_records: u16,
    pub additional_records: u16,
}

impl Header {
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() != 12 {
            return Err("input bytes len is not equal to 12");
        }

        Ok(Self {
            ident: u16::from_be_bytes([data[0], data[1]]),
            query: ((data[2] >> 7) & 1) == 1,
            opcode: (data[2] >> 3),
            authoritative: ((data[2] >> 2) & 1) == 1,
            truncated: ((data[2] >> 1) & 1) == 1,
            recursion_desired: (data[2] & 1) == 1,
            recursion_avail: ((data[3] >> 7) & 1) == 1,
            reserved: ((data[3] >> 4) & 0b111),
            rcode: (data[3] & 0b1111),
            qd_count: u16::from_be_bytes([data[4], data[5]]),
            an_count: u16::from_be_bytes([data[6], data[7]]),
            authority_records: u16::from_be_bytes([data[8], data[9]]),
            additional_records: u16::from_be_bytes([data[10], data[11]]),
        })
    }

    pub fn write_to(self, buf: &mut Vec<u8>) {
        buf.reserve(12);

        // write ident
        buf.extend(self.ident.to_be_bytes());

        // Write flags
        let flag0_byte = (self.query as u8) << 7
            | self.opcode << 3
            | (self.authoritative as u8) << 2
            | (self.truncated as u8) << 1
            | self.recursion_desired as u8;
        let flag1_byte = (self.recursion_avail as u8) << 7 | self.reserved << 4 | self.rcode;

        buf.push(flag0_byte);
        buf.push(flag1_byte);

        // Write counts
        buf.extend(self.qd_count.to_be_bytes());
        buf.extend(self.an_count.to_be_bytes());
        buf.extend(self.authority_records.to_be_bytes());
        buf.extend(self.additional_records.to_be_bytes());
    }
}
