use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub trait NetworkProtocol {
    type ClientPacket;
    type ServerPacket;
}

pub trait Connection: Read + Write {
    fn addr(&self) -> String;
}

// TCP
impl Connection for TcpStream {
    fn addr(&self) -> String {
        self.peer_addr()
            .expect("Could not resolve tcp connection remote address!")
            .to_string()
    }
}
