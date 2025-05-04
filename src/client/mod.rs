use std::{io::Read, net::TcpStream};
use std::io::prelude::*;
use error::ClientError;

use crate::{drawing::DrawSurface, hardware::PhysicalDimensions, instruction::InstructionSet};

pub mod state;
pub mod error;


///
/// An all-inclusive function which will start a drawing, move the pen from 0, 0 to a given
/// position, stop the drawing and end the client.
/// This function is used to move the pen from the top-right corner of the page, to the
/// starting position of the drawing. It is used before ClientState::new(...) is used to complete a
/// proper drawing.
///
/// # Parameters:
/// - `addr`: The IP address of the machine
/// - `port`: The port address of the machine
/// - `physical_dimensions`: A physical dimension object representing the current physical layout
/// - `x`: The x position to move to
/// - `y`: The y position to move to
///
/// # Returns:
/// - Void if the function completed successfully
/// - An error, explaining why the pen could not be moved to the start position
///
pub fn move_to_start(addr: &str, port: u16, physical_dimensions: &PhysicalDimensions, x: f64, y: f64) -> Result<(), ClientError> {
    let raw_ins = DrawSurface::pen_to_start_ins(physical_dimensions, x, y);
    let ins_set = match InstructionSet::new(raw_ins, 0., 0.) {
        Ok(val) => val,
        Err(str) => return Err(ClientError::InvalidBytes { reason: format!("Instructions to move pen to starting position were invalid. {}", str).to_owned() }),
    };

    // okay so here we have the instructions, we will now do a very lightweight, blocking drawing loop
    // with no simultaneously read/write functionality whatsoever.
    let socket = TcpStream::connect(format!("{}:{}", addr, port));
    if let Err(_) = socket {
            return Err(ClientError::MachineNotFound { addr: addr.to_owned(), port });
        }
    let mut safe_socket = socket.unwrap();
    
    // send the greeting bytes
    let _ = safe_socket.write_all(&[0x00, 0x01]);
    let mut sent_move_bytes = false;

    // then we loop, doing a blocking await for bytes
    loop {
        
        let mut incoming_buf: [u8; 255] = [0; 255];
        let _ = safe_socket.read(&mut incoming_buf);
        
        // its asking for what to do next
        if *incoming_buf.get(0).unwrap() == 0x03 {
            if !sent_move_bytes {
                
                let _ = safe_socket.write_all(&[0x01]);
                let _ = safe_socket.write_all(&ins_set.get_binary());
                sent_move_bytes = true;

            } else {
                
                let _ = safe_socket.write_all(&[0x02]);
                return Ok(());
            }
        }

        // its saying the machine is in use
        if *incoming_buf.get(0).unwrap() == 0x00 {
            return Err(ClientError::MachineInUse);
        }

        // its sent a response to the greeting bytes
        // this (should) run first in the loop
        if *incoming_buf.get(0).unwrap() == 0x01 {
            let (_, ins_buf_size, _, _) = read_header(&incoming_buf);
            if (ins_buf_size as usize) < ins_set.get_binary().len() {
                return Err(ClientError::InsBufferSmall { size: ins_buf_size });
            }
        }
    }
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
fn read_header(header: &[u8; 255]) -> (u16, u32, u32, u32) {
    // ignore first byte.
    
    (
        bytes_to_u16(header, 1),
        // start from ins here, 4 bytes, ignoring it for now
        bytes_to_u32(header, 7),
        bytes_to_u32(header, 11),
        bytes_to_u32(header, 15),
    )
}
