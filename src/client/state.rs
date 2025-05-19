use byteorder::ByteOrder;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use std::sync::Arc;

use crate::instruction::InstructionSet;
use crate::client::calculate_draw_time;

use super::error::ClientError;
use super::read_header;

///
/// Empty struct for method implementation.
///
pub struct ClientState {}

impl ClientState {


    ///
    /// Creates a new TcpStream along with the machine configuration or an error.
    /// The TcpStream can be separated into the read/write halves in an implementation.
    /// This function also initialises a drawing with greeting bytes, if a connection is established.
    ///
    /// # Parameters:
    /// - `addr`: The IP address of the machine
    /// - `port`: The port address of the machine
    ///
    /// # Returns:
    /// - An owned TcpStream, and the machines configuration
    /// - A `ClientError` if the connection could not be established
    ///
    pub async fn new(addr: &str, port: u16) -> Result<(TcpStream, MachineConfiguration), ClientError> {
        // attempt to connect to the socket
        let socket = TcpStream::connect(format!("{}:{}", addr, port)).await;
        if let Err(_) = socket {
            return Err(ClientError::MachineNotFound { addr: addr.to_owned(), port });
        }

        let mut safe_socket = socket.unwrap();

        // send greeting byte and read response
        let _ = safe_socket.write_all(&[0x00, 0x01]).await;
        let mut inc_buffer = [0; 255];
        if let Err(_) = safe_socket.read(&mut inc_buffer).await {
            return Err(ClientError::GreetingTimedOut);
        };

        if *inc_buffer.get(0).unwrap() == 0x01 {
            // machine is okay to get started with drawing. so initialise machine config, and
            // return the client state instance so the implementation (frontend, cli) can takeover
            let (protocol_version, instruction_buffer_size, max_motor_speed, min_pulse_width) = read_header(&inc_buffer);
            let machine_configuration = MachineConfiguration { protocol_version, instruction_buffer_size, max_motor_speed, min_pulse_width };

            if machine_configuration.instruction_buffer_size < 1024 {
                return Err(ClientError::InsBufferSmall { size: machine_configuration.instruction_buffer_size });
            }

            return Ok((safe_socket, machine_configuration));

        } else if *inc_buffer.get(0).unwrap() == 0x00 {
            // machine is NOT okay to get started. protocol should parse this here
            return Err(ClientError::MachineInUse);

        } else { // TODO firmware returns protocol, return invalid protocol
            return Err(ClientError::InvalidBytes { reason: "Sent a greeting but the response header was not 0x01".to_owned() })

        }
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

        emit(r#"{"event":"pause", "is_paused":""#.to_owned() + (if flag_byte == 0x01 { "1" } else { "0" }) + r#""}"#);
    } 

    /// 
    /// TODO: Possibly add proper packet for graceful shutdown? Return current ins?
    ///
    /// Shuts the socket down, hence cancelling the drawing.
    ///
    /// # Parameters:
    /// - `writer`: A mutex-locked TcpStream write half
    /// - `emit`: A callback function to emit updates from the function
    ///
    pub async fn stop<F>(writer: &mut OwnedWriteHalf, mut emit: F)
    where
        F: FnMut(String) + Send + 'static {
        // shutdown byte
        let _ = writer.write_all(&[0x05]).await; 
        let _ = writer.shutdown().await;
        emit(r#"{"event":"shutdown"}"#.to_owned());
    }


    /// 
    /// TODO: If protocol enum implementations are added, can be used here
    ///
    /// Continuously listens for bytes from a TcpStream's read half. It handles the incoming bytes
    /// appropriately, sometimes writing to the stream.
    ///
    /// # Parameters:
    /// - `reader`: A mutex-locked read half of a TcpStream
    /// - `write_ref`: A reference to the guarded TcpStream write half
    /// - `buf_idx`: A usize identifying the ins_set bound to send to the machine
    /// - `ins_set`: The drawing instruction set
    /// - `emit`: A callback function to emit updates from the function
    ///
    pub async fn listen<F>(reader: &mut OwnedReadHalf, write_ref: &Arc<Mutex<Option<OwnedWriteHalf>>>, buf_idx: &Arc<Mutex<usize>>, ins_set: &InstructionSet, machine_config: &MachineConfiguration, mut emit: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        // continuous blocking loop
        loop {
            let mut incoming_buf: [u8; 255] = [0; 255];
            let _ = reader.read(&mut incoming_buf).await; // will block

            if *incoming_buf.get(0).unwrap() == 0x02 {}

            if *incoming_buf.get(0).unwrap() == 0x03 {
                let mut next_buf_lock = buf_idx.lock().await;
                *next_buf_lock += 1;

                let bounds = ins_set.get_buffer_bounds(machine_config.instruction_buffer_size as usize).unwrap();

                if *next_buf_lock - 1 == bounds.len() {

                    let mut write_lock = write_ref.lock().await;
                    let writer = write_lock.as_mut().unwrap();
                    let _ = writer.write_all(&[0x02]).await;
                    
                    // reader gets shutdown when write does im pretty sure
                    let _ = writer.shutdown().await;

                    drop(write_lock);
                    drop(next_buf_lock);

                    emit(r#"{"event":"drawing_finished"}"#.to_owned());

                    // println!("Drawing has finished. Stopped listen loop.");
                    return;
                }
                

                let (lb, ub) = bounds.get(*next_buf_lock - 1).unwrap();

                let mut write_lock = write_ref.lock().await;
                let writer = write_lock.as_mut().unwrap();
                let mut buf = Vec::with_capacity(1 + ub - lb + 1);
                buf.push(0x01);
                buf.extend_from_slice(&ins_set.get_binary()[*lb..=*ub]);
                let _ = writer.write_all(&buf).await;
                
                // this is a little progress update
                // event:drawing, new_ins: bytes:bytes (num/of num) time:newseconds
                let remaining_draw_time = calculate_draw_time(&ins_set.get_binary()[*lb..], machine_config.max_motor_speed, machine_config.min_pulse_width).as_secs();
                emit(
                    format!(
                        r#"{{"event":"drawing", "ins_pos":"{}", "secs_remaining":"{}"}}"#, format!("{} 🡲 {} ({}/{})", lb, ub, *next_buf_lock, ins_set.get_buffer_bounds(4096).unwrap().len()), remaining_draw_time
                    )
                );

                drop(write_lock);
                drop(next_buf_lock);
                continue;
            }

            if *incoming_buf.get(0).unwrap() == 0x05 {
                return;
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
pub struct MachineConfiguration {
    pub protocol_version: u16,
    pub instruction_buffer_size: u32,
    pub max_motor_speed: u32,
    pub min_pulse_width: u32,
}

