pub mod data;
pub mod transport;
pub mod models;


pub type Error = Box<dyn std::error::Error>;


pub use self::{
    data::{Timestamp, Metadata},
    transport::{Packet, PacketType},
};

// foreign re-exports
pub use async_trait::async_trait;
