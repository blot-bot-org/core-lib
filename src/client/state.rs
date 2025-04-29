use std::io::prelude::*;
use std::time::Duration;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf};
use std::sync::Arc;

use super::error::ClientError;

/// 
/// A representation of a connection to the drawing machine.
/// Typically in frontends, to be usable, this should be wrapped in an Arc<Mutex<...>> so it is
/// thread-safe, as in this manner you can send pause requests whilst blocking for incoming bytes.
///
/// # Fields:
/// - `socket`: The TCP socket connection to the machine
/// - `awaiting_pause`: Whether a client has sent a pause request, and is awaiting a response
/// - `drawing_paused`: Whether the machine recognises itself as paused
/// - `machine_configuration`: The configuration of the machine
/// - `terminating_instruction_idx`: If a drawing is stopped for any reason, this holds the
/// instruction stopped at. This includes stopping when a drawing has finished
///
pub struct ClientState {}

impl ClientState {
    pub async fn new(addr: &str, port: u16) -> Result<TcpStream, ClientError> {
        // attempt to connect to the socket
        let socket = TcpStream::connect(format!("{}:{}", addr, port)).await;
        if let Err(_) = socket {
            return Err(ClientError::MachineNotFound { addr: addr.to_owned(), port });
        }

        let mut safe_socket = socket.unwrap();

        // send greeting byte and read response
        let _ = safe_socket.write(&[0x00, 0x01]).await;
        let mut inc_buffer = [0; 64];
        if let Err(_) = safe_socket.read(&mut inc_buffer).await {
            return Err(ClientError::GreetingTimedOut);
        };

        if *inc_buffer.get(0).unwrap() == 0x01 {
            // machine is okay to get started with drawing. so initialise machine config, and
            // return the client state instance so the implementation (frontend, cli) can takeover
            let (protocol_version, instruction_buffer_size, max_motor_speed, min_pulse_width) = read_header(&inc_buffer);

            println!("{} {} {} {}", protocol_version, instruction_buffer_size, max_motor_speed, min_pulse_width);
        } else if *inc_buffer.get(0).unwrap() == 0x00 {
            // machine is NOT okay to get started. protocol should parse this here
            return Err(ClientError::MachineInUse);
        } else { // TODO firmware returns protocol, return invalid protocol
            return Err(ClientError::MachineInUse);
        }

        Ok(safe_socket)
    }


    // specific client implementation can be handled here
    pub async fn pause(writer: &mut OwnedWriteHalf) {
        let _ = writer.write(&[0x04, 0x01]).await;
    } 


    pub async fn listen(reader: &mut OwnedReadHalf, write_ref: &Arc<Mutex<Option<OwnedWriteHalf>>>) {
        println!("starting listen");
    
        loop {
            let mut incoming_buf: [u8; 255] = [0; 255];
            println!("AWAITING BYTES.");
            let _ = reader.read(&mut incoming_buf).await;


            println!("Received something...");
            for b in incoming_buf.iter() {
                print!("0x{:02x} ", b);
            }
            println!();



            if *incoming_buf.get(0).unwrap() == 0x03 {
                println!("Sending more instructions.");

                let bytes = "\x01\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C".to_owned().into_bytes();

                let mut write_lock = write_ref.lock().await;
                let writer = write_lock.as_mut().unwrap();
                let _ = writer.write(&bytes).await;
                drop(write_lock);
            }
        }
    }
}


/// 
/// Wrapper of basic machine configuration information.
/// This is received from the machine when a connection is established.
///
/// # Fields:
/// - `protocol_version`: The protocol version of the drawing machine
/// - `instruction_buffer_size`: The size of the machines instruction buffer
/// - `max_motor_speed`: The maximum steps per second
/// - `min_pulse_width`: The minimum pulse width of a motor step, in nanoseconds
///
struct MachineConfiguration {
    protocol_version: u16,
    instruction_buffer_size: u32,
    max_motor_speed: u32,
    min_pulse_width: u32,
}












fn bytes_to_u16(array: &[u8], index: usize) -> u16 {
    if index + 1 > array.len() {
        println!("Error converting byteslice to u16 - bytes out of array index");
        return 0;
    }

    (array[index] as u16) << 8 | array[index + 1] as u16
}

fn bytes_to_u32(array: &[u8], index: usize) -> u32 {
    if index + 3 > array.len() {
        println!("Error converting byteslice to u32 - bytes out of array index");
        return 0;
    }

     (array[index] as u32) << 24 | (array[index + 1] as u32) << 16 | (array[index + 2] as u32) << 8 | array[index + 3] as u32
}

// these functions should be extracted to protocol handlers
fn read_header(header: &[u8; 64]) -> (u16, u32, u32, u32) {
    // ignore first byte.
    
    (
        bytes_to_u16(header, 1),
        // start from ins here, 4 bytes, ignoring it for now
        bytes_to_u32(header, 7),
        bytes_to_u32(header, 11),
        bytes_to_u32(header, 15),
    )
}
