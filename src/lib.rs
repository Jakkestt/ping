mod errors;
mod packet;
mod ping;

pub use crate::ping::{open_socket, ping, PingSocket};
