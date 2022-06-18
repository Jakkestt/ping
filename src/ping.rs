use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use rand::random;
use socket2::{Domain, Protocol, Socket, Type};

use crate::errors::Error;
use crate::packet::{EchoReply, EchoRequest, IcmpV4, IcmpV6, IpV4Packet, ICMP_HEADER_SIZE};

const TOKEN_SIZE: usize = 24;
const ECHO_REQUEST_BUFFER_SIZE: usize = ICMP_HEADER_SIZE + TOKEN_SIZE;
type Token = [u8; TOKEN_SIZE];

pub struct PingSocket {
    buffer: [u8; ECHO_REQUEST_BUFFER_SIZE],
    addr: SocketAddr,
    socket: Socket,
}

pub fn open_socket(
    addr: IpAddr,
    timeout: Option<Duration>,
    ttl: Option<u32>,
    ident: Option<u16>,
    seq_cnt: Option<u16>,
    payload: Option<&Token>,
) -> Result<PingSocket, Error> {
    let timeout = match timeout {
        Some(timeout) => Some(timeout),
        None => Some(Duration::from_secs(4)),
    };

    let dest = SocketAddr::new(addr, 0);
    let mut buffer = [0; ECHO_REQUEST_BUFFER_SIZE];

    let default_payload: &Token = &random();

    let request = EchoRequest {
        ident: ident.unwrap_or(random()),
        seq_cnt: seq_cnt.unwrap_or(1),
        payload: payload.unwrap_or(default_payload),
    };

    let socket = if dest.is_ipv4() {
        if request.encode::<IcmpV4>(&mut buffer[..]).is_err() {
            return Err(Error::InternalError.into());
        }
        Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?
    } else {
        if request.encode::<IcmpV6>(&mut buffer[..]).is_err() {
            return Err(Error::InternalError.into());
        }
        Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6))?
    };

    if dest.is_ipv4() {
        socket.set_ttl(ttl.unwrap_or(64))?;
    } else {
        socket.set_unicast_hops_v6(ttl.unwrap_or(64))?;
    }

    socket.set_write_timeout(timeout)?;

    socket.set_read_timeout(timeout)?;

    Ok(PingSocket {
        buffer,
        addr: dest,
        socket,
    })
}

pub fn ping(socket: &mut PingSocket) -> Result<(), Error> {
    socket
        .socket
        .send_to(&mut socket.buffer, &socket.addr.into())?;

    let mut buffer: [u8; 2048] = [0; 2048];
    socket.socket.read(&mut buffer)?;

    let _reply = if socket.addr.is_ipv4() {
        let ipv4_packet = match IpV4Packet::decode(&buffer) {
            Ok(packet) => packet,
            Err(_) => return Err(Error::InternalError),
        };
        match EchoReply::decode::<IcmpV4>(ipv4_packet.data) {
            Ok(reply) => reply,
            Err(_) => return Err(Error::InternalError.into()),
        }
    } else {
        match EchoReply::decode::<IcmpV6>(&buffer) {
            Ok(reply) => reply,
            Err(_) => return Err(Error::InternalError.into()),
        }
    };

    Ok(())
}
