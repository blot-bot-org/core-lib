use thiserror::Error;

/// All errors emitted from the instruction module.
///
/// - `StartOutOfBounds`: When a buffer is initialised with a start index outside the length of the vector
///     Parameters:
///     - `start_idx`: The requested starting index
///     - `upper_bound`: The length of the vector
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

    #[error("An instruction did not end with the instruction termination 0x0C, instead {:#04x}", .0)]
    IncompleteInstructions(u8),

    #[error("The provided instruction set is empty")]
    EmptyInstructionSet,

    #[error("The provided instruction set is of invalid length, `length % 5 != 0`")]
    InvalidLength,

    #[error("The configured instruction buffer size is too small {}", .0)]
    BufferTooSmall(usize),
}
