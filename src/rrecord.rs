use crate::qname::Qname;

#[derive(Debug)]
pub struct RRecord<'a> {
    pub name: Qname<'a>,
    pub r_type: u16,
    pub class: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: RData,
}

impl<'a> RRecord<'a> {
    pub fn parse(mut data: &'a [u8]) -> Result<Self, &'static str> {
        let qname = Qname::parse(data);
        data = &data[qname.length()..];
        debug_assert!(data.len() >= 10);

        let r_type = u16::from_be_bytes([data[0], data[1]]);
        let class = u16::from_be_bytes([data[2], data[3]]);
        let ttl = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let rdlength = u16::from_be_bytes([data[8], data[9]]);

        data = &data[2 + 2 + 4 + 2..];

        let rdata = Self::parse_rdata(r_type, data);

        Ok(Self {
            name: qname,
            r_type,
            class,
            ttl,
            rdlength,
            rdata,
        })
    }

    fn parse_rdata(r_type: u16, data: &[u8]) -> RData {
        match r_type {
            1 => RData::A([data[0], data[1], data[2], data[3]]),

            _ => unimplemented!(),
        }
    }

    pub fn write_to(self, buf: &mut Vec<u8>) {
        buf.reserve(self.length());

        self.name.write_to(buf);
        buf.extend(self.r_type.to_be_bytes());
        buf.extend(self.class.to_be_bytes());
        buf.extend(self.ttl.to_be_bytes());
        buf.extend(self.rdlength.to_be_bytes());

        self.rdata.write_to(buf);
    }

    pub fn length(&self) -> usize {
        self.name.length() + 2 + 2 + 4 + 2 + self.rdlength as usize
    }
}

#[derive(Debug)]
pub enum RData {
    A([u8; 4]),
    Aaaa([u8; 16]),
}

impl RData {
    pub fn write_to(self, buf: &mut Vec<u8>) {
        match self {
            RData::A(addr) => buf.extend(addr),
            RData::Aaaa(addr) => buf.extend(addr),
        }
    }
}
