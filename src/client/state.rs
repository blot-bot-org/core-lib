use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use std::sync::Arc;

use crate::instruction::InstructionSet;

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
    ///
    /// Creates a new TcpStream or an error. The TcpStream can be separated into the read/write halves in an implementation.
    /// This function also initialises a drawing with greeting bytes, if a connection is
    /// established.
    ///
    /// # Parameters:
    /// - `addr`: The IP address of the machine
    /// - `port`: The port address of the machine
    ///
    /// # Returns:
    /// - An owned TcpStream
    /// - A `ClientError` if the connection could not be established
    ///
    pub async fn new(addr: &str, port: u16) -> Result<TcpStream, ClientError> {
        // attempt to connect to the socket
        let socket = TcpStream::connect(format!("{}:{}", addr, port)).await;
        if let Err(_) = socket {
            return Err(ClientError::MachineNotFound { addr: addr.to_owned(), port });
        }

        let mut safe_socket = socket.unwrap();

        // send greeting byte and read response
        let _ = safe_socket.write_all(&[0x00, 0x01]).await;
        let mut inc_buffer = [0; 256];
        if let Err(_) = safe_socket.read(&mut inc_buffer).await {
            return Err(ClientError::GreetingTimedOut);
        };

        if *inc_buffer.get(0).unwrap() == 0x01 {
            // machine is okay to get started with drawing. so initialise machine config, and
            // return the client state instance so the implementation (frontend, cli) can takeover
            
            let (protocol_version, instruction_buffer_size, max_motor_speed, min_pulse_width) = read_header(&inc_buffer);
            // can init to machine_configuration if needed
            
            println!("machinedrawing ready to draw...\nmachine_protocol:{}\nmax_buffer_size:{}\nmax_motor_speed:{}\nmin_pulse_width:{}", protocol_version, instruction_buffer_size, max_motor_speed, min_pulse_width);

        } else if *inc_buffer.get(0).unwrap() == 0x00 {
            // machine is NOT okay to get started. protocol should parse this here
            return Err(ClientError::MachineInUse);
        } else { // TODO firmware returns protocol, return invalid protocol
            return Err(ClientError::MachineInUse);
        }

        Ok(safe_socket)
    }


    /// 
    /// TODO: If protocol enum implementations are added, can be used here
    ///
    /// Writes a pause packet to a given TcpStream write half.
    ///
    /// # Parameters:
    /// - `writer`: A mutex-locked TcpStream write half
    /// - `should_pause`: true to pause, false to resume
    /// - `emit`: A callback function to emit updates from the function
    ///
    pub async fn pause<F>(writer: &mut OwnedWriteHalf, should_pause: bool, mut emit: F)
    where
        F: FnMut(String) + Send + 'static {
        let flag_byte: u8 = match should_pause {
            true => 0x01,
            _ => 0x00
        };

        // 0x01 = pause, 0x00 = resume
        let _ = writer.write_all(&[0x04, flag_byte]).await;

        emit(r#"{"event":"drawing", "message":"The drawing was "#.to_owned() + (if flag_byte == 0x01 { "paused" } else { "resumed" }) + r#""}"#);
    } 


    /// 
    /// TODO: If protocol enum implementations are added, can be used here
    ///
    /// Continuously listens for bytes from a TcpStream's read half. It handles the incoming bytes
    /// appropriately, sometimes writing to the stream.
    ///
    /// # Parameters:
    /// - `reader`: A mutex-locked read half of a TcpStream
    /// - `writer`: A reference to the guarded TcpStream write half
    /// - `buf_idx`: A usize identifying the ins_set bound to send to the machine
    /// - `ins_set`: The drawing instruction set
    /// - `emit`: A callback function to emit updates from the function
    ///
    pub async fn listen<F>(reader: &mut OwnedReadHalf, write_ref: &Arc<Mutex<Option<OwnedWriteHalf>>>, buf_idx: &Arc<Mutex<usize>>, ins_set: &InstructionSet, mut emit: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        // println!("starting listen loop");
    
        loop {
            let mut incoming_buf: [u8; 255] = [0; 255];
            let _ = reader.read(&mut incoming_buf).await; // will block

            /*
            println!("Received something...");
            for b in 0..32 {
                print!("0x{:02x} ", incoming_buf[b]);
            }
            println!();
            */

            if *incoming_buf.get(0).unwrap() == 0x03 {
                let mut next_buf_lock = buf_idx.lock().await;
                *next_buf_lock += 1;

                let bounds = ins_set.get_buffer_bounds(32768).unwrap();

                if *next_buf_lock - 1 == bounds.len() {

                    let mut write_lock = write_ref.lock().await;
                    let writer = write_lock.as_mut().unwrap();
                    let _ = writer.write_all(&[0x02]).await;
                    
                    // reader gets shutdown when write does im pretty sure
                    let _ = writer.shutdown().await;

                    drop(write_lock);
                    drop(next_buf_lock);

                    // println!("Drawing has finished. Stopped listen loop.");
                    return;
                }
                
                // this is a little console progress update
                // println!("Sending more instructions (buf_idx {}/{})", *next_buf_lock, ins_set.get_buffer_bounds(4096).unwrap().len());
                emit(r#"{"event":"drawing", "message":"Sent more drawing instructions ("#.to_owned() + (format!("{}/{}", *next_buf_lock, ins_set.get_buffer_bounds(32768).unwrap().len())).as_str() + r#")"}"#);


                let (lb, ub) = bounds.get(*next_buf_lock - 1).unwrap();
                drop(next_buf_lock);

                let mut write_lock = write_ref.lock().await;
                let writer = write_lock.as_mut().unwrap();
                let _ = writer.write_all(&[0x01]).await;
                let _ = writer.write_all(&ins_set.get_binary()[*lb..=*ub]).await;
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



/// 
/// Converts 2 bytes to a u16
///
/// # Parameters:
/// - `array`: The byte buffer
/// - `index`: The first-byte's index
/// 
/// # Returns:
/// - The value of the bytes, as a u16
///
fn bytes_to_u16(array: &[u8], index: usize) -> u16 {
    if index + 1 > array.len() {
        println!("Error converting byteslice to u16 - bytes out of array index");
        return 0;
    }

    (array[index] as u16) << 8 | array[index + 1] as u16
}

/// 
/// Converts 4 bytes to a u32
///
/// # Parameters:
/// - `array`: The byte buffer
/// - `index`: The first-byte's index
/// 
/// # Returns:
/// - The value of the bytes, as a u32
///
fn bytes_to_u32(array: &[u8], index: usize) -> u32 {
    if index + 3 > array.len() {
        println!("Error converting byteslice to u32 - bytes out of array index");
        return 0;
    }

     (array[index] as u32) << 24 | (array[index + 1] as u32) << 16 | (array[index + 2] as u32) << 8 | array[index + 3] as u32
}

/// 
/// Extracts and returns bytes from the greeting response
///
/// # Parameters:
/// - `header`: The incoming buffer
///
/// # Returns:
/// - (protocol_version, instruction_buffer_size, max_motor_speed, min_pulse_width) as reported by
/// the machine
///
fn read_header(header: &[u8; 256]) -> (u16, u32, u32, u32) {
    // ignore first byte.
    
    (
        bytes_to_u16(header, 1),
        // start from ins here, 4 bytes, ignoring it for now
        bytes_to_u32(header, 7),
        bytes_to_u32(header, 11),
        bytes_to_u32(header, 15),
    )
}
