use std::net::UdpSocket;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let _received_data = String::from_utf8_lossy(&buf[0..size]);
                println!("Received {} bytes from {}", size, source);
                let mut response = vec![];

                let packet = Packet {
                    header: Header {
                        ident: 1234,
                        query: true,
                        opcode: 0,
                        authoritative: false,
                        truncated: false,
                        recursion_desired: false,
                        recursion_avail: false,
                        reserved: 0,
                        rcode: 0,
                        question_records: 1,
                        answer_records: 0,
                        authority_records: 0,
                        additional_records: 0,
                    },
                    questions: vec![Question {
                        name: "codecrafters.io".into(),
                        q_type: 1,
                        class: 1,
                    }],
                };

                packet.write_to(&mut response);

                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}

struct Packet {
    header: Header,
    questions: Vec<Question>,
}

struct Header {
    ident: u16,
    query: bool,
    opcode: u8, // TODO: enum
    authoritative: bool,
    truncated: bool,
    recursion_desired: bool,
    recursion_avail: bool,
    reserved: u8,
    rcode: u8, // TODO: enum
    question_records: u16,
    answer_records: u16,
    authority_records: u16,
    additional_records: u16,
}

struct Question {
    name: Qname,
    q_type: u16,
    class: u16,
}

impl Question {
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        Err("invalid")
    }

    pub fn write_to(self, buf: &mut Vec<u8>) {
        self.name.write_to(buf);
        buf.extend(self.q_type.to_le_bytes());
        buf.extend(self.class.to_le_bytes());
    }
}

struct Qname(Vec<(u8, String)>);

impl Qname {
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        for (i, v) in &self.0 {
            buf.push(*i);
            buf.extend(v.bytes());
        }
    }
}

impl From<&str> for Qname {
    fn from(value: &str) -> Self {
        let mut output = vec![];

        for label in value.split('.') {
            output.push((label.len() as u8, label.to_string()));
        }

        Qname(output)
    }
}

impl Packet {
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        let header = Header::parse(&data[..12])?;
        let questions = vec![Question::parse(&data[12..])?]; // TODO: need some thing better here

        Ok(Self { header, questions })
    }

    pub fn write_to(self, buf: &mut Vec<u8>) {
        self.header.write_to(buf);

        for question in self.questions {
            question.write_to(buf);
        }
    }
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
            question_records: u16::from_be_bytes([data[4], data[5]]),
            answer_records: u16::from_be_bytes([data[6], data[7]]),
            authority_records: u16::from_be_bytes([data[8], data[9]]),
            additional_records: u16::from_be_bytes([data[10], data[11]]),
        })
    }

    pub fn write_to(self, buf: &mut Vec<u8>) {
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
        buf.extend(self.question_records.to_le_bytes());
        buf.extend(self.answer_records.to_le_bytes());
        buf.extend(self.authority_records.to_le_bytes());
        buf.extend(self.additional_records.to_le_bytes());
    }
}
