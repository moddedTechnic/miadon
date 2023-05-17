use crate::streams::{IStream, OStream};


#[derive(Clone, Debug)]
pub struct IPV4(pub u8, pub u8, pub u8, pub u8);

impl IPV4 {
    pub fn read_from_stream(stream: &mut dyn IStream) -> Self {
        let p0 = stream.read_u8();
        let p1 = stream.read_u8();
        let p2 = stream.read_u8();
        let p3 = stream.read_u8();
        Self(p0, p1, p2, p3)
    }

    pub fn write_to_stream(&self, stream: &mut dyn OStream) {
        stream.write_u8(self.0);
        stream.write_u8(self.1);
        stream.write_u8(self.2);
        stream.write_u8(self.3);
    }
}

#[derive(Clone, Debug)]
pub struct Domain(pub Vec<String>);

impl Domain {
    pub fn head(&self) -> String  {
        self.0.last().unwrap().clone()
    }

    pub fn tail(&self) -> Self {
        Self(self.0[..self.0.len()-1].to_vec())
    }

    pub fn read_from_stream(stream: &mut dyn IStream) -> Self {
        let mut parts: Vec<String> = Vec::new();
        let mut size: u8;
        while ({size = stream.read_u8(); size}) > 0 {
            parts.push(stream.read_string(size as usize));
        };
        Self(parts)
    }

    pub fn write_to_stream(&self, stream: &mut dyn OStream) {
        for part in self.0.clone() {
            stream.write_u8(part.len() as u8);
            stream.write_string(part);
        }
        stream.write_u8(0);
    }
}


#[derive(Clone, Debug)]
pub enum ResourceRecordType {
    A,
    Options,
}

impl Into<u8> for ResourceRecordType {
    fn into(self) -> u8 {
        match self {
            Self::A => 1,
            Self::Options => 41,
        }
    }
}

impl Into<u16> for ResourceRecordType {
    fn into(self) -> u16 {
        Into::<u8>::into(self) as u16
    }
}

impl Into<ResourceRecordType> for u8 {
    fn into(self) -> ResourceRecordType {
        (self as u16).into()
    }
}

impl Into<ResourceRecordType> for u16 {
    fn into(self) -> ResourceRecordType {
        match self {
            1 => ResourceRecordType::A,
            41 => ResourceRecordType::Options,
            x => panic!("Unknown resource record type {x}"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ResourceRecordData {
    A(IPV4),
    Options(Vec<u8>),
}

impl ResourceRecordData {
    pub fn read_from_stream(stream: &mut dyn IStream, typ: &ResourceRecordType) -> Self {
        let size = stream.read_u16();
        match typ {
            ResourceRecordType::A => Self::A(IPV4::read_from_stream(stream)),
            ResourceRecordType::Options => Self::Options(stream.read_bytes(size as usize)),
        }
    }

    pub fn write_to_stream(&self, stream: &mut dyn OStream) {
        stream.write_u16(self.len() as u16);
        match self {
            Self::A(ip) => ip.write_to_stream(stream),
            Self::Options(options) => stream.write_bytes(options.clone()),
        };
    }

    fn len(&self) -> usize {
        match self {
            Self::A(_) => 4,
            Self::Options(options) => options.len(),
        }
    }
}

impl Into<ResourceRecordType> for ResourceRecordData {
    fn into(self) -> ResourceRecordType {
        match self {
            Self::A(_) => ResourceRecordType::A,
            Self::Options(_) => ResourceRecordType::Options,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ResponseCode {
    NoError,
    FormatError,
    ServerFailure,
    NonExistentDomain,
    NotImplemented,
    Refused,
    NameExists,
    ResourceRecordSet,
    ResourceRecordNotSet,
    NotAuthorized,
    NotInZone,
    DSOTypeNotImplemented,
    BadVersion,
    BadKey,
    BadTime,
    BadMode,
    BadName,
    BadAlgorithm,
    BadTruncation,
    BadCookie,
}

impl Into<ResponseCode> for u8 {
    fn into(self) -> ResponseCode {
        (self as u16).into()
    }
}

impl Into<ResponseCode> for u16 {
    fn into(self) -> ResponseCode {
        match self {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NonExistentDomain,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            6 => ResponseCode::NameExists,
            7 => ResponseCode::ResourceRecordSet,
            8 => ResponseCode::ResourceRecordNotSet,
            9 => ResponseCode::NotAuthorized,
            10 => ResponseCode::NotInZone,
            11 => ResponseCode::DSOTypeNotImplemented,
            16 => ResponseCode::BadVersion,
            17 => ResponseCode::BadKey,
            18 => ResponseCode::BadTime,
            19 => ResponseCode::BadMode,
            20 => ResponseCode::BadName,
            21 => ResponseCode::BadAlgorithm,
            22 => ResponseCode::BadTruncation,
            23 => ResponseCode::BadCookie,
            x => panic!("Unknown response code {x}"),
        }
    }
}

impl Into<u8> for ResponseCode {
    fn into(self) -> u8 {
        match self {
            Self::NoError => 0,
            Self::FormatError => 1,
            Self::ServerFailure => 2,
            Self::NonExistentDomain => 3,
            Self::NotImplemented => 4,
            Self::Refused => 5,
            Self::NameExists => 6,
            Self::ResourceRecordSet => 7,
            Self::ResourceRecordNotSet => 8,
            Self::NotAuthorized => 9,
            Self::NotInZone => 10,
            Self::DSOTypeNotImplemented => 11,
            Self::BadVersion => 16,
            Self::BadKey => 17,
            Self::BadTime => 18,
            Self::BadMode => 19,
            Self::BadName => 20,
            Self::BadAlgorithm => 21,
            Self::BadTruncation => 22,
            Self::BadCookie => 23,
        }
    }
}

impl Into<u16> for ResponseCode {
    fn into(self) -> u16 {
        Into::<u8>::into(self) as u16
    }
}

#[derive(Clone, Debug)]
pub enum Operation {
    Query,
    InverseQuery,
    Status,
    Notify,
    Update,
    StatefulOperation,
}

impl Into<Operation> for u8 {
    fn into(self) -> Operation {
        (self as u16).into()
    }
}

impl Into<Operation> for u16 {
    fn into(self) -> Operation {
        match self {
            0 => Operation::Query,
            1 => Operation::InverseQuery,
            2 => Operation::Status,
            4 => Operation::Notify,
            5 => Operation::Update,
            6 => Operation::StatefulOperation,
            x => panic!("Unknown operation with code {x}"),
        }
    }
}

impl Into<u8> for Operation {
    fn into(self) -> u8 {
        match self {
            Self::Query => 0,
            Self::InverseQuery => 1,
            Self::Status => 2,
            Self::Notify => 4,
            Self::Update => 5,
            Self::StatefulOperation => 6,
        }
    }
}

impl Into<u16> for Operation {
    fn into(self) -> u16 {
        Into::<u8>::into(self) as u16
    }
}

#[derive(Clone, Debug)]
pub enum Class {
    Internet,
    Chaos,
    Hesiod,
    None,
    Any,
    Unknown(u16),
}

impl Into<Class> for u16 {
    fn into(self) -> Class {
        match self {
            1 => Class::Internet,
            3 => Class::Chaos,
            4 => Class::Hesiod,
            254 => Class::None,
            255 => Class::Any,
            x => Class::Unknown(x),
        }
    }
}

impl Into<u16> for Class {
    fn into(self) -> u16 {
        match self {
            Self::Internet => 1,
            Self::Chaos => 3,
            Self::Hesiod => 4,
            Self::None => 254,
            Self::Any => 255,
            Self::Unknown(x) => x,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Flags {
    is_query: bool,
    is_authoritative_answer: bool,
    is_truncated: bool,
    is_recursion_desired: bool,
    is_recursion_available: bool,
    operation: Operation,
    response_code: ResponseCode,
}

impl Flags {
    pub fn new(
        is_query: bool,
        is_authoritative_answer: bool,
        is_truncated: bool,
        is_recursion_desired: bool,
        is_recursion_available: bool,
        operation: Operation,
        response_code: ResponseCode,
    ) -> Self {
        Self {
            is_query,
            is_authoritative_answer,
            is_truncated,
            is_recursion_desired,
            is_recursion_available,
            operation,
            response_code,
        }
    }

    pub fn read_from_stream(stream: &mut dyn IStream) -> Self {
        let flags = stream.read_u16();
        Flags {
            is_query: flags & 0x8000 == 0,
            is_authoritative_answer: flags & 0x0400 != 0,
            is_truncated: flags & 0x0200 != 0,
            is_recursion_desired: flags & 0x0100 != 0,
            is_recursion_available: flags & 0x0080 != 0,
            operation: ((flags & 0x7800) >> 11).into(),
            response_code: (flags & 0x000f).into(),
        }
    }

    pub fn write_to_stream(&self, stream: &mut dyn OStream) {
        let mut flags: u16 = 0;
        flags |= (!self.is_query as u16) << 15;
        flags |= (Into::<u16>::into(self.operation.clone())) << 11;
        flags |= (self.is_authoritative_answer as u16) << 10;
        flags |= (self.is_truncated as u16) << 9;
        flags |= (self.is_recursion_desired as u16) << 8;
        flags |= (self.is_recursion_available as u16) << 7;
        flags |= Into::<u16>::into(self.response_code.clone());
        stream.write_u16(flags);
    }
}


#[derive(Clone, Debug)]
pub struct Question {
    pub name: Domain,
    pub typ: ResourceRecordType,
    pub class: Class,
}

impl Question {
    pub fn read_from_stream(stream: &mut dyn IStream) -> Self {
        Question {
            name: Domain::read_from_stream(stream),
            typ: stream.read_u16().into(),
            class: stream.read_u16().into(),
        }
    }

    pub fn write_to_stream(&self, stream: &mut dyn OStream) {
        self.name.write_to_stream(stream);
        stream.write_u16(self.typ.clone().into());
        stream.write_u16(self.class.clone().into());
    }
}


#[derive(Clone, Debug)]
pub struct ResourceRecord {
    name: Domain,
    class: Class,
    time_to_live: u32,
    data: ResourceRecordData,
}

impl ResourceRecord {
    pub fn new(name: Domain, class: Class, time_to_live: u32, data: ResourceRecordData) -> Self {
        Self {
            name, class,
            time_to_live,
            data,
        }
    }

    pub fn read_from_stream(stream: &mut dyn IStream) -> Self {
        let name = Domain::read_from_stream(stream);
        let typ: ResourceRecordType = stream.read_u16().into();
        ResourceRecord {
            name,
            class: stream.read_u16().into(),
            time_to_live: stream.read_u32(),
            data: ResourceRecordData::read_from_stream(stream, &typ),
        }
    }

    pub fn write_to_stream(&self, stream: &mut dyn OStream) {
        self.name.write_to_stream(stream);
        stream.write_u16(Into::<ResourceRecordType>::into(self.data.clone()).into());
        stream.write_u16(self.class.clone().into());
        stream.write_u32(self.time_to_live);
        self.data.write_to_stream(stream);
    }
}


#[derive(Clone, Debug)]
pub struct Message {
    pub id: u16,
    pub flags: Flags,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authoritative_records: Vec<ResourceRecord>,
    pub additional_records: Vec<ResourceRecord>,
}

impl Message {
    pub fn new(
        id: u16, flags: Flags,
        questions: Vec<Question>, answers: Vec<ResourceRecord>,
        authoritative_records: Vec<ResourceRecord>,
        additional_records: Vec<ResourceRecord>
    ) -> Self {
        Self {
            id, flags,
            questions, answers,
            authoritative_records,
            additional_records,
        }
    }

    pub fn read_from_stream(stream: &mut dyn IStream) -> Self {
        let id = stream.read_u16();
        let flags = Flags::read_from_stream(stream);
        let questions_count = stream.read_u16();
        let answers_count = stream.read_u16();
        let authoritative_records_count = stream.read_u16();
        let additional_records_count = stream.read_u16();
        Self {
            id, flags,
            questions: (0..questions_count).map(|_| Question::read_from_stream(stream)).collect(),
            answers: (0..answers_count).map(|_| ResourceRecord::read_from_stream(stream)).collect(),
            authoritative_records: (0..authoritative_records_count).map(|_| ResourceRecord::read_from_stream(stream)).collect(),
            additional_records: (0..additional_records_count).map(|_| ResourceRecord::read_from_stream(stream)).collect(),
        }
    }

    pub fn write_to_stream(&self, stream: &mut dyn OStream) {
        stream.write_u16(self.id);
        self.flags.write_to_stream(stream);
        stream.write_u16(self.questions.len() as u16);
        stream.write_u16(self.answers.len() as u16);
        stream.write_u16(self.authoritative_records.len() as u16);
        stream.write_u16(self.additional_records.len() as u16);
        self.questions.iter().for_each(|q| q.write_to_stream(stream));
        self.answers.iter().for_each(|a| a.write_to_stream(stream));
        self.authoritative_records.iter().for_each(|ar| ar.write_to_stream(stream));
        self.additional_records.iter().for_each(|ar| ar.write_to_stream(stream));
    }
}
