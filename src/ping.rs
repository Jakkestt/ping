use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use rand::random;
use socket2::{Domain, Protocol, Socket, Type};

use crate::errors::Error;
use crate::packet::{EchoReply, EchoRequest, IcmpV4, IcmpV6, IpV4Packet, ICMP_HEADER_SIZE};

const TOKEN_SIZE: usize = 24;
const ECHO_REQUEST_BUFFER_SIZE: usize = ICMP_HEADER_SIZE + TOKEN_SIZE;

pub struct PingSocket {
    request: EchoRequest,
    addr: SocketAddr,
    socket: Socket,
}

pub fn open_socket(
    addr: IpAddr,
    timeout: Option<Duration>,
    ttl: Option<u32>,
    ident: Option<u16>,
) -> Result<PingSocket, Error> {
    let timeout = match timeout {
        Some(timeout) => Some(timeout),
        None => Some(Duration::from_secs(4)),
    };

    let dest = SocketAddr::new(addr, 0);

    let request = EchoRequest {
        ident: ident.unwrap_or_else(random),
        seq_cnt: 0,
    };

    let socket = if dest.is_ipv4() {
        Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?
    } else {
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
        request,
        addr: dest,
        socket,
    })
}

pub fn ping(socket: &mut PingSocket) -> Result<(), Error> {
    let mut buffer = [0; ECHO_REQUEST_BUFFER_SIZE];
    if socket.addr.is_ipv4() {
        if socket.request.encode::<IcmpV4>(&mut buffer[..]).is_err() {
            return Err(Error::InternalErr.into());
        }
    } else {
        if socket.request.encode::<IcmpV6>(&mut buffer[..]).is_err() {
            return Err(Error::InternalErr.into());
        }
    }

    println!("{:?}", buffer);

    let bytes = socket.socket.send_to(&buffer, &socket.addr.into())?;
    socket.socket.flush()?;
    println!("Pinged {bytes} bytes");

    let mut buffer: [u8; 2048] = [0; 2048];
    Read::read(&mut socket.socket, &mut buffer)?;

    println!("{:?}", buffer);

    let _reply = if socket.addr.is_ipv4() {
        let ipv4_packet = match IpV4Packet::decode(&buffer) {
            Ok(packet) => packet,
            Err(_) => return Err(Error::InternalErr),
        };
        match EchoReply::decode::<IcmpV4>(ipv4_packet.data) {
            Ok(reply) => reply,
            Err(_) => return Err(Error::InternalErr),
        }
    } else {
        match EchoReply::decode::<IcmpV6>(&buffer) {
            Ok(reply) => reply,
            Err(_) => return Err(Error::InternalErr),
        }
    };

    Ok(())
}
