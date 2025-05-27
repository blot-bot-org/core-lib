//!
//! Opcode instructions representations and handlers
//! 

pub mod error;

use once_cell::sync::OnceCell;

use byteorder::{BigEndian, ByteOrder};
use error::InstructionError;

///
/// An instruction set, to represent all instructions required to draw an image.
///
/// # Fields:
/// - `binary`: Vector of bytes, containing the raw binary instructions
/// - `buffer_bound_cache`: The bounds of slices to be passed to the machine
/// - `init_x`: The initial x position of the pen in a given drawing
/// - `init_y`: The initial y position of the pen in a given drawing
///
pub struct InstructionSet {
    binary: Vec<u8>,
    buffer_bound_cache: OnceCell<Vec<(usize, usize)>>,
    init_x: f64,
    init_y: f64,
}

impl InstructionSet {
    ///
    /// Creates a new instance of an `InstructionSet`. If an `InstructionSet` instance is returned,
    /// the instruction bytes are valid.
    ///
    /// # Parameters:
    /// - `ins_bytes`: Vector of bytes, containing the proposed raw binary instructions
    /// - `init_x`: The initial x position of the pen in a given drawing
    /// - `init_y`: The initial y position of the pen in a given drawing
    ///
    /// # Returns:
    /// - An InstructionSet with a valid binary sequence
    /// - An error explaining why the provided `ins` was invalid
    /// 
    pub fn new(ins_bytes: Vec<u8>, init_x: f64, init_y: f64) -> Result<InstructionSet, InstructionError> {
        match is_stream_valid(&ins_bytes) {
            None => {
                Ok(InstructionSet { binary: ins_bytes, buffer_bound_cache: OnceCell::new(), init_x, init_y })
            }
            Some(err) => {
                Err(err)
            }
        }
    }

    ///
    /// Creates a new instance of an `InstructionSet` with an initial byte-offset. If an `InstructionSet` instance is returned,
    /// the instruction bytes are valid. The offset bytes are trimmed before being stored in the struct.
    ///
    /// # Parameters:
    /// - `ins_bytes`: Vector of bytes, containing the proposed raw binary instructions
    /// - `init_x`: The initial x position of the pen in a given drawing
    /// - `init_y`: The initial y position of the pen in a given drawing
    /// - `start_idx`: Index of byte to start on, must be within the length of `ins_bytes`
    /// 
    /// # Returns:
    /// - An InstructionSet with a valid binary sequence
    /// - An error explaining why the provided `ins` was invalid
    /// 
    pub fn new_from_idx(ins_bytes: Vec<u8>, init_x: f64, init_y: f64, start_idx: usize) -> Result<InstructionSet, InstructionError> {
        if start_idx >= ins_bytes.len() {
            return Err(InstructionError::StartOutOfBounds { start_idx, upper_bound: ins_bytes.len() });
        }

        match is_stream_valid(&ins_bytes[start_idx..].to_vec()) {
            None => {
                // ideally we wouldn't reallocate here but whatever
                Ok(InstructionSet { binary: ins_bytes[start_idx..].to_vec(), buffer_bound_cache: OnceCell::new(), init_x, init_y })
            }
            Some(err) => {
                Err(err)
            }
        }
    }

    ///
    /// Generates the index bounds of buffers to send over the socket to the drawing machine.
    /// The bounds, say (0, 14) means send all bytes from 0 to 14 inclusive.
    ///
    /// # Parameters:
    /// - `max_chunk_size`: The maximum preferred chunk size of buffers
    ///
    /// # Returns:
    /// - A vector of tuples, denoting the index boundaries of buffers
    /// - An error explaining why the index boundaries could not be computed
    ///
    pub fn get_buffer_bounds(&self, max_chunk_size: usize) -> Result<&Vec<(usize, usize)>, InstructionError> {

        self.buffer_bound_cache.get_or_try_init(|| {
            if max_chunk_size < 8 {
                return Err(InstructionError::BufferTooSmall(max_chunk_size));
            }

            let mut chunk_bounds: Vec<(usize, usize)> = vec![];
            let mut start_idx: usize = 0;

            let mut current_idx = start_idx;
            
            loop {
                if current_idx >= self.binary.len() - 1 {
                    break;
                }

                let max_end_idx = (start_idx + max_chunk_size).min(self.binary.len() - 1); // max end bound
                let mut last_valid_max = current_idx; // last valid end bound

                while current_idx < max_end_idx {
                    current_idx += 4;

                    if self.get_binary()[current_idx] == 0x0C {
                        current_idx += 1;
                    } else if self.get_binary()[current_idx] == 0x0A {
                        current_idx += 1;
                        if self.get_binary()[current_idx] != 0x0C {
                            return Err(InstructionError::IncompleteInstructions(self.get_binary()[current_idx]));
                        } else {
                            current_idx += 1;
                        }
                    } else if self.get_binary()[current_idx] == 0x0B {
                        current_idx += 1;
                        if self.get_binary()[current_idx] != 0x0C {
                            return Err(InstructionError::IncompleteInstructions(self.get_binary()[current_idx]));
                        } else {
                            current_idx += 1;
                        }
                    } else {
                        return Err(InstructionError::IncompleteInstructions(self.get_binary()[current_idx]));
                    }


                    last_valid_max = current_idx - 1;
                }

                chunk_bounds.push((start_idx, last_valid_max));
                start_idx = last_valid_max + 1;

            }

            Ok(chunk_bounds)
        })
    }

    ///
    /// Parses an `InstructionSet` into a set of numerical step values the motors will perform.
    ///
    /// # Parameters:
    /// - `instruction_set`: An instruction set
    ///
    /// # Returns:
    /// - A vector of tuple (i16, i16, bool) values the belts will move by, and whether the pen is up, as per the provided instruction set.
    ///
    pub fn parse_to_numerical_steps(&self) -> Result<Vec<(i16, i16, bool)>, InstructionError> {
        let result_buffer_bounds = match self.get_buffer_bounds(512) {
            Ok(value) => value,
            Err(err) => return Err(err)
        };

        let mut numerical_instructions: Vec<(i16, i16, bool)> = vec![];
        let mut pen_up = true;

        for (s_idx, e_idx) in result_buffer_bounds {
            
            let mut c_idx = *s_idx;
            while c_idx < *e_idx {
                c_idx += 4;
                if c_idx >= *e_idx { break; };

                let left_steps = BigEndian::read_i16(&[*self.binary.get(c_idx - 4).unwrap() as u8, *self.binary.get(c_idx - 3).unwrap() as u8]);
                let right_steps = BigEndian::read_i16(&[*self.binary.get(c_idx - 2).unwrap() as u8, *self.binary.get(c_idx - 1).unwrap() as u8]);

                let next_byte = self.get_binary()[c_idx];
                if next_byte == 0x0C {
                } else if next_byte == 0x0A {
                    pen_up = true;
                    c_idx += 1;
                    if self.get_binary()[c_idx] != 0x0C { return Err(InstructionError::IncompleteInstructions(self.get_binary()[c_idx])); }
                } else if next_byte == 0x0B {
                    pen_up = false;
                    c_idx += 1;
                    if self.get_binary()[c_idx] != 0x0C { return Err(InstructionError::IncompleteInstructions(self.get_binary()[c_idx])); }
                } else {
                    return Err(InstructionError::IncompleteInstructions(next_byte));
                }

                numerical_instructions.push((left_steps, right_steps, pen_up));
                c_idx += 1;

            }
        }

        Ok(numerical_instructions)
    }

    ///
    /// # Returns:
    /// - The binary instructions, as a vector of bytes
    ///
    pub fn get_binary(&self) -> &Vec<u8> {
        &self.binary
    }

    ///
    /// # Returns:
    /// - The initial pen position of the drawing
    ///
    pub fn get_init(&self) -> (f64, f64) {
        (self.init_x, self.init_y)   
    }
}


///
/// Performs validty checks as to if the bytes are valid instructions.
///
/// # Parameters:
/// - `ins_bytes`: Vector of bytes, containing raw binary instructions
///
/// # Returns:
/// - `None`, if the bytes are valid instructions
/// - An error to explain why the bytes were invalid
///
fn is_stream_valid(ins_bytes: &[u8]) -> Option<InstructionError> {
    if ins_bytes.is_empty() {
        return Some(InstructionError::EmptyInstructionSet);
    }

    let mut c_idx = 0;
    loop {
        c_idx += 4;
        if c_idx >= ins_bytes.len() {
            break;
        }

        if ins_bytes[c_idx] == 0x0C { // end instruction
            c_idx += 1; // skip 0x0c, check next ins
            continue;
        } else if ins_bytes[c_idx] == 0x0A { // pen up
            c_idx += 1;
            if ins_bytes[c_idx] == 0x0C {
                c_idx += 1;
                continue
            }
        } else if ins_bytes[c_idx] == 0x0B { // pen down
            c_idx += 1;
            if ins_bytes[c_idx] == 0x0C {
                c_idx += 1;
                continue;
            }
        } else {
            // terminator byte wasnt 0x0c, or technically if extra bytes weren't pen up/down
            return Some(InstructionError::IncompleteInstructions(ins_bytes[c_idx]));
        }

        return Some(InstructionError::IncompleteInstructions(ins_bytes[c_idx]));
    }

    None
}





///
/// Tests relating to the InstructionSet struct and associated functions.
///
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn too_small_chunk_size() {
        InstructionSet::new("\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C".to_owned().into_bytes(), 0., 0.).unwrap().get_buffer_bounds(6).unwrap();
    }

    #[test]
    fn valid_instruction_stream() {
        let is = InstructionSet::new("\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C".to_owned().into_bytes(), 0., 0.).unwrap();
        let bb = is.get_buffer_bounds(11).unwrap();
        assert_eq!(*bb, [(0, 9), (10, 14)]);
    }

    #[test]
    fn invalid_instruction_stream_0xc() {
        assert!(InstructionSet::new("\x0A\x0B\x2A\x3A\x0C\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A".to_owned().into_bytes(), 0., 0.).is_err());
    }

    #[test]
    fn invalid_instruction_stream_indexed_0xc() {
        assert!(InstructionSet::new_from_idx("\x0A\x0B\x2A\x3A\x0C\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A".to_owned().into_bytes(), 0., 0., 2).is_err());
    }

    #[test]
    fn empty_instruction_stream() {
        assert!(InstructionSet::new("".to_owned().into_bytes(), 0., 0.).is_err());
    }

    #[test]
    fn empty_instruction_stream_indexed() {
        assert!(InstructionSet::new_from_idx("".to_owned().into_bytes(), 0., 0., 4).is_err());
    }

    #[test]
    fn invalid_instruction_stream_indexed() {
        assert!(InstructionSet::new_from_idx("".to_owned().into_bytes(), 0., 0., 4).is_err());
    }

    #[test]
    fn invalid_instruction_stream_indexed_oub() {
        assert!(InstructionSet::new_from_idx("\x0A\x0B\x2A\x3A\x0A\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C".to_owned().into_bytes(), 0., 0., 20).is_err());
    }

    #[test]
    fn valid_instruction_stream_indexed() {
        let is = InstructionSet::new_from_idx("\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C".to_owned().into_bytes(), 0., 0., 5).unwrap();
        let bb = is.get_buffer_bounds(64).unwrap();
        assert_eq!(*bb, [(0, 9)]);
    }

    #[test]
    fn validate_valid_stream() {
        assert!(is_stream_valid(&InstructionSet::new("\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C".to_owned().into_bytes(), 0., 0.).unwrap().get_binary()).is_none());
    }

    #[test]
    fn validate_invalid_stream() {
        assert!(InstructionSet::new("\x0A\x0B\x2A\x0C\x0C\x0B\x2A\x3A\x0C\x0A\x0B\x2A\x3A\x0C".to_owned().into_bytes(), 0., 0.).is_err());
    }

    #[test]
    fn validate_pen_up_down_stream() {
        assert!(is_stream_valid(&InstructionSet::new("\x0A\x0B\x2A\x0C\x0A\x0C\x2A\x3A\x0C\x0A\x0B\x0C".to_owned().into_bytes(), 0., 0.).unwrap().get_binary()).is_none());
    }

    #[test]
    fn validate_pen_up_down_stream_2() {
        assert!(is_stream_valid(&InstructionSet::new("\x0A\x0B\x2A\x0C\x0A\x0C\x2A\x3A\x0C\x0A\x0C".to_owned().into_bytes(), 0., 0.).unwrap().get_binary()).is_none());
    }

    #[test]
    fn validate_not_pen_up_down_stream() {
        assert!(InstructionSet::new("\x0A\x0B\x2A\x0C\x0D\x0C\x2A\x3A\x0C\x0A\x0C".to_owned().into_bytes(), 0., 0.).is_err());
    }
}
