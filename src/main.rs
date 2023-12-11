mod header;
mod qname;
use header::*;
mod question;
use question::*;
mod rrecord;
use rrecord::*;

use std::net::UdpSocket;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let received_data = &buf[0..size];
                println!("Received {} bytes from {}", size, source);
                let mut response = vec![];

                let mut recv_packet = Packet::parse(received_data).unwrap();

                println!("{:?}", recv_packet);

                recv_packet.header.query = true;
                recv_packet.header.authoritative = false;
                recv_packet.header.truncated = false;
                recv_packet.header.recursion_avail = false;
                recv_packet.header.reserved = 0;
                recv_packet.header.rcode = if recv_packet.header.opcode == 0 { 0 } else { 4 };
                recv_packet.header.an_count = recv_packet.header.qd_count;
                recv_packet.header.authority_records = 0;
                recv_packet.header.additional_records = 0;

                for question in recv_packet.questions.iter() {
                    recv_packet.answers.push(RRecord {
                        name: question.name.clone(),
                        r_type: 1,
                        class: 1,
                        ttl: 1337,
                        rdlength: 4,
                        rdata: RData::A([0x8, 0x8, 0x8, 0x8]),
                    })
                }

                recv_packet.write_to(&mut response);

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

#[derive(Debug)]
struct Packet<'a> {
    header: Header,
    questions: Vec<Question<'a>>,
    answers: Vec<RRecord<'a>>,
}

impl<'a> Packet<'a> {
    pub fn parse(mut data: &'a [u8]) -> Result<Self, &'static str> {
        let header = Header::parse(&data[..12])?;
        let original = data;
        data = &data[12..];

        let mut questions = Vec::with_capacity(header.qd_count as usize);

        for _ in 0..header.qd_count {
            let rec = Question::parse(data, original)?;

            println!("label = {}", rec.name);

            data = &data[rec.length()..];
            questions.push(rec);
        }
        let mut answers = Vec::with_capacity(header.an_count as usize);
        for _ in 0..header.an_count {
            let rec = RRecord::parse(data, original)?;

            data = &data[rec.length()..];
            answers.push(rec);
        }

        Ok(Self {
            header,
            questions,
            answers,
        })
    }

    pub fn write_to(self, buf: &mut Vec<u8>) {
        self.header.write_to(buf);

        for question in self.questions {
            question.write_to(buf);
        }

        for answer in self.answers {
            answer.write_to(buf);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Packet;

    #[test]
    fn parse_compressed_packet() {
        let data = [
            // Header
            0xAA, 0xAA, 0x01, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // codecrafters.io, A, Class 1
            0x0C, 0x63, 0x6F, 0x64, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x65, 0x72, 0x73, 0x02,
            0x69, 0x6F, 0x00, 0x00, 0x01, 0x00, 0x01,
            //
            // testing.codecrafters.io, A, class 1
            0x7, 0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67, 0xc0, 0xc, 0x00, 0x01, 0x00, 0x01,
        ];

        let packet = Packet::parse(&data);

        assert!(packet.is_ok());

        println!("{:?}", packet);
    }
}
