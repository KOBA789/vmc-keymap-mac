#[derive(Debug)]
pub enum Atom<'a> {
    Int32(i32),
    Float32(f32),
    String(&'a [u8]),
}

impl<'a> Atom<'a> {
    pub fn as_int32(&self) -> Option<i32> {
        if let Self::Int32(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_float32(&self) -> Option<f32> {
        if let Self::Float32(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&'a [u8]> {
        if let Self::String(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

pub mod atom {
    pub struct Parser<'a> {
        bytes: &'a [u8],
    }

    impl<'a> Parser<'a> {
        pub fn new(bytes: &'a [u8]) -> Self {
            Self { bytes }
        }

        pub fn read_string(&mut self) -> Result<&'a [u8], ()> {
            let len = self.bytes.iter().position(|b| *b == 0).ok_or(())?;
            let padded_len = len / 4 * 4 + 4;
            let (s, rest) = self.bytes.split_at(padded_len);
            self.bytes = rest;
            Ok(&s[..len])
        }

        pub fn read_int32(&mut self) -> Result<i32, ()> {
            if self.bytes.len() < 4 {
                return Err(());
            }
            let (quad, rest) = self.bytes.split_at(4);
            self.bytes = rest;
            Ok(i32::from_be_bytes(quad.try_into().map_err(|_| ())?))
        }

        pub fn read_float32(&mut self) -> Result<f32, ()> {
            if self.bytes.len() < 4 {
                return Err(());
            }
            let (quad, rest) = self.bytes.split_at(4);
            self.bytes = rest;
            Ok(f32::from_be_bytes(quad.try_into().map_err(|_| ())?))
        }

        pub fn read_timestamp(&mut self) -> Result<u64, ()> {
            if self.bytes.len() < 8 {
                return Err(());
            }
            Ok(u64::from_be_bytes(
                self.bytes[..8].try_into().map_err(|_| ())?,
            ))
        }

        pub fn is_end_of_data(&self) -> bool {
            self.bytes.is_empty()
        }

        pub fn rest(&self) -> &'a [u8] {
            self.bytes
        }
    }
}

pub mod message {
    use super::{atom, Atom};

    pub struct Parser<'a> {
        address: &'a [u8],
        type_tags: &'a [u8],
        atom_parser: atom::Parser<'a>,
    }

    impl<'a> Parser<'a> {
        pub fn new(bytes: &'a [u8]) -> Result<Self, ()> {
            let mut atom_parser = atom::Parser::new(bytes);
            let address = atom_parser.read_string()?;
            let type_tag = atom_parser.read_string()?;
            if !matches!(type_tag.first(), Some(b',')) {
                return Err(());
            }
            let type_tag = &type_tag[1..];
            Ok(Self {
                address,
                atom_parser,
                type_tags: type_tag,
            })
        }

        pub fn num_of_rest_arguments(&self) -> usize {
            self.type_tags.len()
        }

        pub fn read_argument(&mut self) -> Result<Atom<'a>, ()> {
            let tag = if let Some(tag) = self.type_tags.first() {
                self.type_tags = &self.type_tags[1..];
                *tag
            } else {
                return Err(());
            };
            match tag {
                b'i' => Ok(Atom::Int32(self.atom_parser.read_int32()?)),
                b'f' => Ok(Atom::Float32(self.atom_parser.read_float32()?)),
                b's' => Ok(Atom::String(self.atom_parser.read_string()?)),
                _ => Err(()),
            }
        }

        pub fn address(&self) -> &'a [u8] {
            self.address
        }
    }
}

pub mod bundle {
    use super::{atom, message};

    pub struct Parser<'a> {
        timestamp: u64,
        bytes: &'a [u8],
    }

    impl<'a> Parser<'a> {
        pub fn new(bytes: &'a [u8]) -> Result<Self, ()> {
            let mut atom_parser = atom::Parser::new(bytes);
            let s = atom_parser.read_string()?;
            if s != b"#bundle\0" {
                return Err(());
            }
            let timestamp = atom_parser.read_timestamp()?;
            let bytes = atom_parser.rest();
            Ok(Self { timestamp, bytes })
        }

        pub fn is_end_of_data(&self) -> bool {
            self.bytes.is_empty()
        }

        pub fn read_message(&mut self) -> Result<message::Parser<'a>, ()> {
            let mut atom_parser = atom::Parser::new(self.bytes);
            let message_len = atom_parser.read_int32()? as usize;
            let (message_bytes, rest) = atom_parser.rest().split_at(message_len);
            self.bytes = rest;
            message::Parser::new(message_bytes)
        }

        pub fn timestamp(&self) -> u64 {
            self.timestamp
        }
    }
}

pub mod packet {
    use super::{bundle, message};

    pub enum Parser<'a> {
        Bundle(bundle::Parser<'a>),
        Message(Option<message::Parser<'a>>),
    }

    impl<'a> Parser<'a> {
        pub fn new(bytes: &'a [u8]) -> Result<Self, ()> {
            if let Ok(bundle) = bundle::Parser::new(bytes) {
                Ok(Self::Bundle(bundle))
            } else {
                Ok(Self::Message(Some(message::Parser::new(bytes)?)))
            }
        }

        pub fn is_end_of_data(&self) -> bool {
            match self {
                Parser::Bundle(bundle) => bundle.is_end_of_data(),
                Parser::Message(message) => message.is_none(),
            }
        }

        pub fn read_message(&mut self) -> Result<message::Parser<'a>, ()> {
            match self {
                Parser::Bundle(bundle) => bundle.read_message(),
                Parser::Message(message) => message.take().ok_or(()),
            }
        }
    }
}
