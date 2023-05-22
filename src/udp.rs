use std::net::{UdpSocket, SocketAddr};

use crate::streams::{IStream, OStream};


pub struct UDPStream {
    socket: UdpSocket,
    target: Option<SocketAddr>,

    in_buffer: [u8; 1024],
    in_buffer_count: usize,
    in_buffer_index: usize,
    
    out_buffer: Vec<u8>,
}

impl UDPStream {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket, target: None,
            in_buffer: [0u8; 1024],
            in_buffer_count: 0, in_buffer_index: 1024,
            out_buffer: vec![],
        }
    }

    pub fn set_target(&mut self, target: SocketAddr) {
        self.target = Some(target);
    }

    pub fn flush(&mut self) {
        match self.target {
            Some(target) => self.socket.send_to(&self.out_buffer, target).unwrap(),
            None => panic!("Tried to send data with no target"),
        };
        self.out_buffer.clear();
    }
}

impl IStream for UDPStream {
    fn read_u8(&mut self) -> u8 {
        if self.in_buffer_count == 0 || self.in_buffer_count == self.in_buffer_index {
            let (buffer_count, target) = self.socket.recv_from(&mut self.in_buffer).expect("Failed to read from stream");
            self.in_buffer_index = 0;
            self.in_buffer_count = buffer_count;
            self.target = Some(target);
        }
        self.in_buffer_index += 1;
        self.in_buffer[self.in_buffer_index - 1]
    }
}

impl OStream for UDPStream {
    fn write_u8(&mut self, x: u8) {
        self.out_buffer.push(x);
    }
}
