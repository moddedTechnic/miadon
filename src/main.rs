
mod message;
mod streams;

use std::{net::{UdpSocket, SocketAddr}, collections::HashMap};

use streams::{IStream, OStream};

use crate::message::*;


struct UDPStream {
    socket: UdpSocket,
    target: Option<SocketAddr>,

    in_buffer: [u8; 1024],
    in_buffer_count: usize,
    in_buffer_index: usize,
    
    out_buffer: Vec<u8>,
}

impl UDPStream {
    fn new(socket: UdpSocket) -> Self {
        Self {
            socket, target: None,
            in_buffer: [0u8; 1024],
            in_buffer_count: 0, in_buffer_index: 1024,
            out_buffer: vec![],
        }
    }

    fn flush(&mut self) {
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


struct Cache(HashMap<String, CacheEntry>);

impl Cache {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn bind(&mut self, domain: Domain, entry: CacheEntry) {
        if domain.0.len() == 1 {
            self.0.insert(domain.head().clone(), entry);
            return;
        }
        if let Some(current_entry) = self.0.get(&domain.head()) {
            match current_entry {
                CacheEntry::Record(ip) => {
                    let mut cache = Cache::new();
                    cache.bind(domain.tail(), entry);
                    self.0.insert(domain.head(), CacheEntry::Zone(Some(ip.clone()), cache));
                },
                CacheEntry::Zone(_, _) => {
                    let cache_entry = self.0.get_mut(&domain.head()).unwrap();
                    match cache_entry {
                        CacheEntry::Zone(_, zone) => zone.bind(domain.tail(), entry),
                        _ => panic!("Invalid state")
                    }
                },
            }
        } else {
            let mut cache = Cache::new();
            cache.bind(Domain(domain.0[1..].to_vec()), entry);
            self.0.insert(domain.0[0].clone(), CacheEntry::Zone(None, cache));
        }
    }

    fn resolve(&self, domain: Domain) -> Option<IPV4> {
        let domain = domain.0.clone();
        if let Some(entry) = self.0.get(&domain[0]) {
            match entry {
                CacheEntry::Record(ip) => Some(ip.clone()),
                CacheEntry::Zone(ip, zone) => {
                    if domain.len() == 1 {
                        ip.clone()
                    } else {
                        let rest = Domain(domain[1..].to_vec());
                        zone.resolve(rest)
                    }
                }
            }
        } else {
            None
        }
    }
}

enum CacheEntry {
    Record(IPV4),
    Zone(Option<IPV4>, Cache),
}

struct HostCache {
    cache: Cache,
}

impl HostCache {
    fn new() -> Self {
        Self { cache: Cache::new() }
    }

    fn bind(&mut self, domain: Domain, entry: CacheEntry) {
        self.cache.bind(domain, entry)
    }

    fn handle_question(&self, question: &Question) -> Option<ResourceRecord> {
        let domain = question.name.clone();
        if let Some(ip) = self.cache.resolve(domain) {
            Some(
                ResourceRecord::new(
                    question.name.clone(),
                    Class::Internet,
                    3600,
                    ResourceRecordData::A(ip),
                )
            )
        } else {
            None
        }
    }
}


fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8053").expect("Failed to bind socket");
    println!("Server listening on {}", socket.local_addr().unwrap());
    let mut stream = UDPStream::new(socket);

    let mut cache = HostCache::new();
    cache.bind(Domain(vec!["localhost".into()]), CacheEntry::Record(IPV4(127, 0, 0, 1)));
    cache.bind(Domain(vec!["google".into(), "com".into()]), CacheEntry::Record(IPV4(8, 8, 8, 8)));

    loop {
        let message = Message::read_from_stream(&mut stream);
        let answers: Vec<_> = message.questions.iter().map(|q| cache.handle_question(q)).filter_map(|x| x).collect();
        let response = if answers.len() > 0 {
            Message::new(
                message.id,
                Flags::new(
                    false, true, false, false, false,
                    Operation::Query, ResponseCode::NoError
                ),
                vec![], answers, vec![], vec![])
        } else {
            Message::new(
                message.id,
                Flags::new(
                    false, false, false, false, false,
                    Operation::Query, ResponseCode::NotInZone
                ),
                vec![], vec![], vec![], vec![]
            )
        };
        response.write_to_stream(&mut stream);
        stream.flush();
    }
}

