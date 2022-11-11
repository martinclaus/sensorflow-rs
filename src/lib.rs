extern crate anyhow;

// Rexport main API
pub use device::FramedListener;
pub use protocol::Frame;

pub mod device;
pub mod protocol;

/// Rexports all error types
pub mod error {
    pub use crate::device::error::*;
    pub use crate::protocol::error::*;
}
