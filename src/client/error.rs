use thiserror::Error;

///
/// All errors emitted from the client module.
/// The error messages can be displayed to users on the frontend. Format nicely please.
///
/// - `MachineInUse`: When a target machine is already in use, usually because it's already
/// drawing.
/// - `MachineNotFound`: When the target machine is not found on the network.
/// - `GreetingTimedOut`: When a greeting is sent, but no response is received.
/// - `InvalidBytes`: A broad error for when the machine receives bytes it wasn't expecting
///     Parameters:
///     - `reason`: The reason the bytes weren't expected / what was expected / anything useful tbh
/// - `InsBufferSmall`: An error encountered when the instruction buffer on the firmware is too small
///     Parameters:
///     - `size`: The size of the instruction buffer
///     
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("The target drawing machine is already in use.")]
    MachineInUse,
    
    #[error("The target machine {}:{} did not respond. It may be the wrong address.", .addr, .port)]
    MachineNotFound { addr: String, port: u16 },

    #[error("Error reading greeting from machine. It's likely the connection timed out.")]
    GreetingTimedOut,

    #[error("Error reading bytes from client. {}", .reason)]
    InvalidBytes { reason: String },

    #[error("The target machine's instruction buffer size was too small: {} bytes", .size)]
    InsBufferSmall { size: u32 },
}
