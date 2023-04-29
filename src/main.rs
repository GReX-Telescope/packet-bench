use num_complex::Complex;
use socket2::{Domain, Socket, Type};
use std::{error::Error, net::SocketAddr};

const PORT: u16 = 60000;
const PAYLOAD_SIZE: usize = 8200;
const CHANNELS: usize = 2048;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Payload {
    /// Number of packets since the first packet
    pub count: u64,
    pub pol_a: [Complex<i8>; CHANNELS],
    pub pol_b: [Complex<i8>; CHANNELS],
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create the socket
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    // Bind our listening address
    let address = SocketAddr::from(([0, 0, 0, 0], PORT));
    socket.bind(&address.into())?;
    // Reuse local address without timeout
    socket.reuse_address()?;
    // Set the buffer size to 256M
    socket.set_recv_buffer_size(256 * 1024 * 1024)?;
    // Transform into a std::UdpSocket
    let socket: std::net::UdpSocket = socket.into();

    // Main loop
    let mut last_count = 0;
    let mut buf = [0u8; PAYLOAD_SIZE];
    let mut first = true;
    loop {
        let n = socket.recv(&mut buf)?;
        if n != buf.len() {
            eprintln!("Malformed packet");
            continue;
        }
        // Now we have a valid packet as bytes in the buffer, transform into our struct
        let payload = unsafe { &*(buf.as_ptr() as *const Payload) };
        // And observe the count
        if first {
            last_count = payload.count;
            first = false;
        } else {
            if payload.count != (last_count + 1) {
                eprintln!("Dropped {} packets", payload.count - last_count + 1);
            }
            last_count = payload.count;
        }
    }
}
