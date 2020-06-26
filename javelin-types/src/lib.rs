pub mod data;
pub mod transport;


pub type Error = Box<dyn std::error::Error>;


pub use self::{
    data::{Timestamp, Metadata},
    transport::{Packet, PacketType},
};
