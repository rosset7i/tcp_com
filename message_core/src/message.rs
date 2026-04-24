use bytes::Bytes;
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Text(String),
    JoinRequest(String),
}

impl Message {
    pub fn serialized(&self) -> Result<Bytes, rmp_serde::encode::Error> {
        Ok(Bytes::from(to_vec(self)?))
    }

    pub fn deserialized(buf: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        from_slice(buf)
    }
}
