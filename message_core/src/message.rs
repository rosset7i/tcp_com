use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Text(String),
    JoinRequest(String),
}

impl Message {
    pub fn serialized(&self) -> Vec<u8> {
        to_vec(self).unwrap()
    }

    pub fn deserialized(buf: &[u8]) -> Self {
        from_slice(buf).unwrap()
    }
}
