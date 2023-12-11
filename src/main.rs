mod header;
mod qname;
use header::*;
mod question;
use question::*;
mod rrecord;
use rrecord::*;

use std::net::{SocketAddr, UdpSocket};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let resolver: SocketAddr = args[2].parse().expect("resolver address not provided");

    println!("resolver = {:?}", resolver);

    let upstream_socket =
        UdpSocket::bind("0.0.0.0:31000").expect("Failed to bind to upstream address");
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 1500];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let received_data = &buf[0..size];
                let mut recv_packet = Packet::parse(received_data).unwrap();

                for question in recv_packet.questions.iter() {
                    let mut packet = recv_packet.clone();
                    packet.header.qd_count = 1;
                    packet.questions = vec![question.clone()];

                    println!("sent packet: {:?}", packet);

                    let mut data = vec![];
                    packet.write_to(&mut data);

                    println!("written packet: {:?}", data);

                    upstream_socket
                        .send_to(&data, resolver)
                        .expect("error in sending data to upstream");
                }
                recv_packet.header.query = true;
                recv_packet.header.authoritative = false;
                recv_packet.header.truncated = false;
                recv_packet.header.recursion_avail = false;
                recv_packet.header.reserved = 0;
                recv_packet.header.rcode = if recv_packet.header.opcode == 0 { 0 } else { 4 };
                recv_packet.header.an_count = recv_packet.header.qd_count;
                recv_packet.header.authority_records = 0;
                recv_packet.header.additional_records = 0;

                let mut responses = vec![[0; 1500]; recv_packet.header.qd_count as usize];
                let mut upstream_packets = vec![None; recv_packet.header.qd_count as usize];

                for (lbuf, packet) in responses.iter_mut().zip(upstream_packets.iter_mut()) {
                    match upstream_socket.recv_from(lbuf) {
                        Ok((size, upstream)) => {
                            println!(
                                "Received {}bytes from {} on upstream socket",
                                size, upstream
                            );

                            *packet = Packet::parse(lbuf).ok();
                        }
                        Err(e) => {
                            eprintln!("error in receiving data from upstream: {}", e);
                        }
                    }
                }

                println!("{:?}", upstream_packets);
                recv_packet.answers = upstream_packets
                    .into_iter()
                    .filter(|x| x.is_some())
                    .flat_map(|packet| {
                        if let Some(packet) = packet {
                            if packet.answers.is_empty() {
                                return None;
                            }
                            Some(packet.answers.clone())
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect();

                let mut response = vec![];
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

#[derive(Debug, Clone)]
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
    fn parse_multiple_questions() {
        let data = [
            0xc2, 0xa5, 0x1, 0x0, 0x0, 0x2, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0x61, 0x62, 0x63,
            0x11, 0x6c, 0x6f, 0x6e, 0x67, 0x61, 0x73, 0x73, 0x64, 0x6f, 0x6d, 0x61, 0x69, 0x6e,
            0x6e, 0x61, 0x6d, 0x65, 0x3, 0x63, 0x6f, 0x6d, 0x0, 0x0, 0x1, 0x0, 0x1, 0x3, 0x64,
            0x65, 0x66, 0xc0, 0x10, 0x0, 0x1, 0x0, 0x1,
        ];

        let packet = match Packet::parse(&data) {
            Ok(v) => v,
            Err(e) => {
                panic!("error in parsing packet: {}", e);
            }
        };

        for question in packet.questions {
            println!("{}", question.name);
        }
    }

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

        let packet = match Packet::parse(&data) {
            Ok(v) => v,
            Err(e) => {
                panic!("failed to parse packet: {}", e);
            }
        };

        let mut buf = Vec::with_capacity(data.len());
        packet.write_to(&mut buf);

        assert_eq!(buf, data);
    }
}
