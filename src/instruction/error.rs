use thiserror::Error;

/// All errors emitted from the instruction module.
///
/// - `StartOutOfBounds`: When a buffer is initialised with a start index outside the length of the vector
///     Parameters:
///     - `start_idx`: The requested starting index
///     - `upper_bound`: The length of the vector
/// - `DrawingOutOfBounds`: When an error occurs due to drawing instructions putting the pen out of bounds
///     Parameters:
///     - `instruction_idx`: The instruction index
///     - `step_x`: The number of x steps
///     - `step_y`: The number of y steps
///     - `prev_x`: The previous x position of the pen 
///     - `prev_y`: The previous y position of the pen 
///     - `target_x`: The target x position of the pen 
///     - `target_y`: The target y position of the pen 
/// - `IncompleteInstructions`: When an instruction doesn't end in the 0x0C termination byte
/// - `EmptyInstructionSet`: When the vector contains no bytes
/// - `InvalidLength`: When the vectors length modulo 5 is not 0, indicating invalid instructions
/// - `BufferTooSmall`: When the requested instruction buffer size for the instruction stream is too small
///     Parameters:
///     - `usize`: The requested buffer size
#[derive(Error, Debug)]
pub enum InstructionError {
    #[error("Invalid start index: {start_idx}, expected between 0 and {}", .upper_bound)]
    StartOutOfBounds { start_idx: usize, upper_bound: usize },

    #[error(
        "The pen is out of bounds. Instruction index {} called steps l:{} r:{}. The belt has moved from x:{} y:{} to x:{} y:{}",
        .instruction_idx,
        .step_x,
        .step_y,
        .prev_x,
        .prev_y,
        .target_x,
        .target_y
    )]
    DrawingOutOfBounds { instruction_idx: usize, step_x: i16, step_y: i16, prev_x: f64, prev_y: f64, target_x: f64, target_y: f64 },

    #[error("An instruction did not end with the instruction termination 0x0C, instead {:#04x}", .0)]
    IncompleteInstructions(u8),

    #[error("The provided instruction set is empty")]
    EmptyInstructionSet,

    #[error("The provided instruction set is of invalid length, `length % 5 != 0`")]
    InvalidLength,

    #[error("The configured instruction buffer size is too small {}", .0)]
    BufferTooSmall(usize),
}
