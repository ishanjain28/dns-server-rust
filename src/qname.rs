use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result as FmtResult, Write},
};

#[derive(Debug, Clone)]
pub struct Qname<'a> {
    name: Cow<'a, [u8]>,
    original: Cow<'a, [u8]>,
    length: usize,
}

impl<'a> Qname<'a> {
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend(self.name.iter());
    }

    // Assume data _starts_ at the question section
    pub fn parse(data: &'a [u8], original: &'a [u8]) -> Result<Self, &'static str> {
        // Do a quick check to make sure every thing is valid
        // and save the slice

        let mut i = 0;
        while i < data.len() - 1 && data[i + 1] != 0 {
            let is_pointer = (data[i] & 0xc0) == 0xc0;

            if is_pointer {
                let offset: u16 = ((data[i] & !0xc0) as u16) << 8 | data[i + 1] as u16;

                // Make sure offset points to a location we have already read
                // 12 bytes for the header
                if offset > i as u16 + 12 {
                    return Err("offset points to a location ahead of itself in the packet");
                }

                i += 1;
            } else {
                let length = data[i] as usize;
                i += length + 1;
            }
        }

        i += 1;

        let start = original.len() - data.len();

        Ok(Self {
            name: Cow::Borrowed(&data[0..i]),
            original: Cow::Borrowed(&original[0..start + i]),
            length: i,
        })
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn labels(&'a self) -> Vec<&'a [u8]> {
        let mut lookup: &[u8] = &self.name;

        let mut out = vec![];

        let mut i = 0;
        while i < lookup.len() - 1 && lookup[i + 1] != 0 {
            let is_pointer = (lookup[i] & 0xc0) == 0xc0;

            if is_pointer {
                let offset: u16 = (((lookup[i] & !0xc0) as u16) << 8) | lookup[i + 1] as u16;

                i = offset as usize;
                lookup = &self.original;
            } else {
                let length = lookup[i] as usize;

                out.push(&lookup[i + 1..i + length + 1]);
                i += length + 1;
            }
        }

        out
    }
}

impl<'a> Display for Qname<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut i = 0;
        let mut lookup: &[u8] = &self.name;

        while i < lookup.len() - 1 && lookup[i + 1] != 0 {
            let is_pointer = (lookup[i] & 0xc0) == 0xc0;

            if is_pointer {
                let offset: u16 = (((lookup[i] & !0xc0) as u16) << 8) | lookup[i + 1] as u16;
                i = offset as usize;
                lookup = &self.original;
            } else {
                let length = lookup[i] as usize;

                for c in &lookup[i + 1..i + 1 + length] {
                    f.write_char(*c as char).unwrap();
                }

                f.write_char('.').unwrap();

                i += length + 1;
            }
        }

        Ok(())
    }
}

impl<'a> From<&'a str> for Qname<'a> {
    fn from(value: &'a str) -> Self {
        let mut labels = vec![];
        let mut length = 0;

        for label in value.split('.') {
            labels.push(label.len() as u8);
            labels.extend(label.bytes());
            length += 1 + label.len();
        }

        // for null byte
        labels.push(0);
        length += 1;

        Qname {
            name: Cow::Owned(labels),
            // This should not cause any problems since
            // we are not yet compressing any thing
            // it might be better to get rid of this completely and provide
            // a custom implementation which also allows compression ?
            original: Cow::Owned(vec![]),
            length,
        }
    }
}
