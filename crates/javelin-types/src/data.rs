use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{Error, Packet, PacketType};


type StringMap = HashMap<String, String>;
type StrMap<'a> = HashMap<&'a str, String>;


#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Timestamp {
    value: u64,
}

impl From<u32> for Timestamp {
    fn from(val: u32) -> Self {
        Self { value: val.into() }
    }
}

impl From<Timestamp> for u32 {
    fn from(val: Timestamp) -> Self {
        val.value as u32
    }
}

impl From<u64> for Timestamp {
    fn from(val: u64) -> Self {
        Self { value: val }
    }
}

impl From<Timestamp> for u64 {
    fn from(val: Timestamp) -> Self {
        val.value
    }
}

impl From<i64> for Timestamp {
    fn from(val: i64) -> Self {
        Self { value: val as u64 }
    }
}

impl From<Timestamp> for i64 {
    fn from(val: Timestamp) -> Self {
        val.value as i64
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata(StringMap);

impl Metadata {
    pub fn get<V, K>(&self, key: K) -> Option<V>
    where
        K: AsRef<str>,
        V: FromStr,
    {
        self.0.get(key.as_ref()).map(|v| v.parse().ok()).flatten()
    }
}

impl From<StringMap> for Metadata {
    fn from(val: HashMap<String, String>) -> Self {
        Self(val)
    }
}

impl<'a> From<StrMap<'a>> for Metadata {
    fn from(val: StrMap<'a>) -> Self {
        let new_map = val
            .into_iter()
            .fold(StringMap::new(), |mut acc, (key, value)| {
                acc.insert(key.to_owned(), value);
                acc
            });
        Self::from(new_map)
    }
}

impl TryFrom<Metadata> for Bytes {
    type Error = Box<dyn std::error::Error>;

    fn try_from(val: Metadata) -> Result<Self, Self::Error> {
        let data = bincode::serialize(&val)?;
        Ok(Bytes::from(data))
    }
}

impl TryFrom<&[u8]> for Metadata {
    type Error = Box<dyn std::error::Error>;

    fn try_from(val: &[u8]) -> Result<Self, Self::Error> {
        Ok(bincode::deserialize(val)?)
    }
}

impl TryFrom<Metadata> for Packet {
    type Error = Error;

    fn try_from(val: Metadata) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: PacketType::Meta,
            timestamp: None,
            payload: Bytes::try_from(val)?,
        })
    }
}

impl TryFrom<Packet> for Metadata {
    type Error = Error;

    fn try_from(val: Packet) -> Result<Self, Self::Error> {
        let payload = &*val.payload;
        Self::try_from(payload)
    }
}
