
mod message;
mod streams;
mod udp;

use std::{net::{UdpSocket, SocketAddr}, collections::HashMap, str::FromStr};

use crate::{message::*, udp::UDPStream};


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
                CacheEntry::Record(records) => {
                    let mut cache = Cache::new();
                    cache.bind(domain.tail(), entry);
                    self.0.insert(domain.head(), CacheEntry::Zone(records.clone(), cache));
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
            self.0.insert(domain.0[0].clone(), CacheEntry::Zone(vec![], cache));
        }
    }

    fn resolve(&self, domain: Domain) -> Vec<ResourceRecordData> {
        let domain = domain.0.clone();
        if let Some(entry) = self.0.get(&domain[0]) {
            match entry {
                CacheEntry::Record(records) => records.clone(),
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
            vec![]
        }
    }
}

enum CacheEntry {
    Record(Vec<ResourceRecordData>),
    Zone(Vec<ResourceRecordData>, Cache),
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
        let domain = &question.name;
        let records = self.cache.resolve(domain.clone());
        records
            .iter()
            .filter(|r| question.typ == r.clone().into())
            .next()
            .map(|r| ResourceRecord::new(domain.clone(), Class::Internet, 60, r.clone()))
    }
}


fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8053").expect("Failed to bind socket");
    println!("Server listening on {}", socket.local_addr().unwrap());
    let mut stream = UDPStream::new(socket);

    let mut cache = HostCache::new();
    cache.bind(Domain(vec!["localhost".into()]), CacheEntry::Record(vec![ResourceRecordData::A(IPV4(127, 0, 0, 1))]));
    cache.bind(Domain(vec!["google".into(), "com".into()]), CacheEntry::Record(vec![ResourceRecordData::A(IPV4(142, 250, 187, 206))]));

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
            let mut upstream = UDPStream::new(UdpSocket::bind("0.0.0.0:0").unwrap());
            let destination = "1.1.1.1:53";
            upstream.set_target(SocketAddr::from_str(destination).unwrap());
            message.write_to_stream(&mut upstream);
            let response = Message::read_from_stream(&mut stream).with_id(message.id);
            
        };
        response.write_to_stream(&mut stream);
        stream.flush();
    }
}

