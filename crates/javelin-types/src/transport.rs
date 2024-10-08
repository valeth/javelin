use {
    std::convert::TryFrom,
    bytes::Bytes,
    serde::{Serialize, Deserialize},
    crate::{Error, Timestamp},
};


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PacketType {
    Meta,
    Video,
    Audio,
    Bytes,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Packet {
    pub kind: PacketType,
    pub timestamp: Option<Timestamp>,
    pub payload: Bytes,
}

impl Packet {
    pub fn new<T, B>(kind: PacketType, timestamp: Option<T>, payload: B) -> Self
        where T: Into<Timestamp>,
              B: Into<Bytes>
    {
        let timestamp = timestamp.map(|v| v.into());
        Self { kind, timestamp, payload: payload.into() }
    }

    pub fn new_video<T, B>(timestamp: T, payload: B) -> Self
        where T: Into<Timestamp>,
              B: Into<Bytes>
    {
        Self::new(PacketType::Video, Some(timestamp), payload)
    }

    pub fn new_audio<T, B>(timestamp: T, payload: B) -> Self
        where T: Into<Timestamp>,
              B: Into<Bytes>
    {
        Self::new(PacketType::Audio, Some(timestamp), payload)
    }

    pub fn pack(&self) -> Result<Bytes, Error> {
        let data = bincode::serialize(&self)?;
        Ok(Bytes::from(data))
    }

    pub fn unpack(bytes: &[u8]) -> Result<Self, Error> {
        Ok(bincode::deserialize(bytes)?)
    }
}

impl AsRef<[u8]> for Packet {
    fn as_ref(&self) -> &[u8] {
        &self.payload
    }
}

impl TryFrom<Packet> for Bytes {
    type Error = Error;

    fn try_from(val: Packet) -> Result<Self, Self::Error> {
        val.pack()
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self, Self::Error> {
        Packet::unpack(&val)
    }
}


