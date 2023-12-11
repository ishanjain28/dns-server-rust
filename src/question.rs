use crate::qname::Qname;

#[derive(Debug, Clone)]
pub struct Question<'a> {
    pub name: Qname<'a>,
    pub q_type: u16,
    pub class: u16,
}

impl<'a> Question<'a> {
    pub fn parse(mut data: &'a [u8], original: &'a [u8]) -> Result<Self, &'static str> {
        let qname = Qname::parse(data, original)?;
        data = &data[qname.length()..];

        debug_assert!(data.len() >= 4);
        let q_type = u16::from_be_bytes([data[0], data[1]]);
        let class = u16::from_be_bytes([data[2], data[3]]);

        Ok(Self {
            name: qname,
            q_type,
            class,
        })
    }

    pub fn length(&self) -> usize {
        self.name.length() + 2 + 2
    }

    pub fn write_to(self, buf: &mut Vec<u8>) {
        buf.reserve(self.length());

        self.name.write_to(buf);
        buf.extend(self.q_type.to_be_bytes());
        buf.extend(self.class.to_be_bytes());
    }
}
