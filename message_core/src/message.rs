use bytes::Bytes;
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};

pub trait Packet<T> {
    fn serialized(&self) -> Result<Bytes, rmp_serde::encode::Error>;
    fn deserialized(buf: &[u8]) -> Result<T, rmp_serde::decode::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Message(String),
    Join(String),
    Encryption,
}

impl Packet<Request> for Request {
    fn serialized(&self) -> Result<Bytes, rmp_serde::encode::Error> {
        Ok(Bytes::from(to_vec(self)?))
    }

    fn deserialized(buf: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        from_slice(buf)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Message(String),
    Join(String),
    Encryption,
}

impl Packet<Response> for Response {
    fn serialized(&self) -> Result<Bytes, rmp_serde::encode::Error> {
        Ok(Bytes::from(to_vec(self)?))
    }

    fn deserialized(buf: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        from_slice(buf)
    }
}
