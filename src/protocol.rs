//! Module for creating data frames from the char stream read from the JeeLink LaCrosse firmware by FHEM.

use bytes::BytesMut;

/// Trait for protocol frame objects.
pub trait Frame: Sized {
    /// Check if a full frame is available in the buffer and returns it if possible.
    ///
    /// The input buffer will be advanced until a start sequence of a frame is reached.
    /// If a complete frame is in the buffer, the frames payload will be extraced and returned, and
    /// the frame data will be remove from the buffer.
    /// If no complete frame is found, the error FrameCheck::Incomplete is returned.
    fn check(buffer: &mut BytesMut) -> Result<BytesMut, error::FrameCheckError>;

    /// Consumes a buffer and returns the corresponding Frame object.
    fn parse(buffer: BytesMut) -> anyhow::Result<Self>;
}

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum FrameCheckError {
        #[error("No complete frame in buffer")]
        Incomplete,
        #[error("Other error occured: {0}")]
        Other(String),
    }

    #[derive(Error, Debug, PartialEq)]
    pub enum FrameValidation {
        #[error("Frame data contains invalid characters. Input: {0}")]
        InvalidChars(String),
        #[error("Insufficient data to parse to frame. Input: {0}")]
        WrongNumberOfFields(String),
    }
}
