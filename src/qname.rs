use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug, Clone)]
pub struct Qname<'a> {
    labels: Cow<'a, [u8]>,
}

impl<'a> Qname<'a> {
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend(self.labels.iter());
    }

    // Assume data _starts_ at the question section
    pub fn parse(data: &'a [u8]) -> Self {
        // Do a quick check to make sure every thing is valid
        // and save the slice

        let mut i = 0;
        while data[i] != 0 {
            let length = data[i] as usize;
            i += length + 1;
        }
        // NULL byte
        i += 1;

        Self {
            labels: Cow::Borrowed(&data[0..i]),
        }
    }

    pub fn length(&self) -> usize {
        self.labels.len()
    }
}

impl<'a> From<&'a str> for Qname<'a> {
    fn from(value: &'a str) -> Self {
        let mut labels = vec![];

        for label in value.split('.') {
            labels.push(label.len() as u8);
            labels.extend(label.bytes());
        }

        // for null byte
        labels.push(0);

        Qname {
            labels: Cow::Owned(labels),
        }
    }
}

impl<'a> Display for Qname<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        todo!()
    }
}
