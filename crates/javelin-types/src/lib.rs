pub mod data;
pub mod models;
pub mod transport;


pub type Error = Box<dyn std::error::Error>;


// foreign re-exports
pub use async_trait::async_trait;

pub use self::data::{Metadata, Timestamp};
pub use self::transport::{Packet, PacketType};
