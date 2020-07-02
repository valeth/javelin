pub mod manager;
mod instance;
mod transport;


type Event = &'static str;
type AppName = String;
type StreamKey = String;

pub use self::{
    manager::Manager,
    transport::{
        ManagerMessage, ManagerHandle,
        Message, Watcher, Handle,
        trigger_channel,
    },
};
