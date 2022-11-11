//! IO devices to read and write data from.
use crate::error::FrameCheckError;
use crate::Frame;
use bytes::BytesMut;
use std::marker::PhantomData;

/// Listener on IO device
///
/// Allows to read frames from device stream.
pub struct FramedListener<P, F> {
    port: P,
    buffer: BytesMut,
    frame_type: PhantomData<F>,
}

impl<P, F: Frame> FramedListener<P, F> {
    pub fn new(port: P) -> FramedListener<P, F> {
        FramedListener {
            port,
            // Allocate buffer with 256 bytes
            buffer: BytesMut::with_capacity(256),
            frame_type: PhantomData,
        }
    }

    fn parse(&mut self) -> anyhow::Result<Option<F>> {
        match F::check(&mut self.buffer) {
            Ok(frame_data) => {
                // parse frame
                let frame = F::parse(frame_data)?;
                Ok(Some(frame))
            }
            Err(FrameCheckError::Incomplete) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

/// Serial devices such as USB
pub mod serial {
    use super::FramedListener;
    use crate::Frame;
    use serialport::TTYPort;
    use std::io::Read;
    use tokio::io::AsyncReadExt;

    impl<F> FramedListener<tokio_serial::SerialStream, F> {
        pub async fn read_frame(&mut self) -> anyhow::Result<Option<F>>
        where
            F: Frame,
        {
            loop {
                if let Some(frame) = self.parse()? {
                    return Ok(Some(frame));
                }

                if 0 == AsyncReadExt::read_buf(&mut self.port, &mut self.buffer).await? {
                    // stream closed. If buffer empty, normal close.
                    if self.buffer.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(super::error::DeviceError::ConnectionLost)?;
                    }
                }
            }
        }
    }

    impl<F> FramedListener<TTYPort, F> {
        pub fn read_frame(&mut self) -> anyhow::Result<Option<F>>
        where
            F: Frame,
        {
            let mut stack_buf = [b'0'; 256];
            loop {
                if let Some(frame) = self.parse()? {
                    return Ok(Some(frame));
                }

                match self.port.read(&mut stack_buf) {
                    Ok(n) if n == 0 => (),
                    Ok(n) => self.buffer.extend_from_slice(&stack_buf[0..n]),
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                    Err(e) => return Err(e)?,
                }
            }
        }
    }
}

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum DeviceError {
        #[error("Connection lost to device")]
        ConnectionLost,
    }
}
