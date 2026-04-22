use rmp_serde::{Serializer, from_slice};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Text(String),
    JoinRequest(String),
}

impl Message {
    pub fn serialized(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.serialize(&mut Serializer::new(&mut buf)).unwrap();
        buf
    }

    pub fn deserialized(buf: Vec<u8>) -> Self {
        from_slice(&buf).unwrap()
    }
}
