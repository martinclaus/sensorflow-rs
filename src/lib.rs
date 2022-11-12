extern crate anyhow;

pub mod devices;
pub mod input;
pub mod output;

// Rexport main API
pub use input::protocol::Frame;
pub use input::FramedListener;

/// Rexports all error types
pub mod error {
    pub use crate::input::error::*;
    pub use crate::input::protocol::error::*;
}
