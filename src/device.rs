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

    pub mod jeelink {
        use crate::{error::*, Frame, FramedListener};
        use bytes::{Buf, BytesMut};
        use std::fmt::{self, Display};

        /// Baud rate of the device. For the JeeLink it is 57.6 KBd
        pub const BAUD_RATE: u32 = 57600;

        pub fn new<P>(port: P) -> FramedListener<P, JeeLinkFrame> {
            FramedListener::<P, JeeLinkFrame>::new(port)
        }

        /// Data Frame received from JeeLink device
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct JeeLinkFrame {
            pub id: u8,
            pub sensor_type: u8,
            pub new_battery: bool,
            pub weak_battery: bool,
            pub temperature: f32,
            pub humidity: u8,
        }

        impl Frame for JeeLinkFrame {
            fn check(buffer: &mut BytesMut) -> Result<BytesMut, FrameCheckError> {
                const START_SEQ: &[u8; 5] = b"OK 9 ";
                const END_SEQ: &[u8; 2] = b"\r\n";
                loop {
                    if buffer.remaining() <= START_SEQ.len() {
                        return Err(FrameCheckError::Incomplete);
                    }
                    if buffer.chunk().starts_with(START_SEQ) {
                        break;
                    }
                    buffer.advance(1)
                }

                if let Some((i, _)) = buffer
                    .windows(2)
                    .enumerate()
                    .find(|(_, win)| win == END_SEQ)
                {
                    let mut frame_data = buffer.split_to(i);
                    frame_data.advance(START_SEQ.len());
                    buffer.advance(END_SEQ.len());
                    Ok(frame_data)
                } else {
                    Err(FrameCheckError::Incomplete)
                }
            }

            fn parse(buffer: BytesMut) -> anyhow::Result<Self> {
                let s = std::str::from_utf8(&buffer)?;
                Self::validate(s)?;

                let fields: Vec<&str> = s.split(|b| b == ' ').collect();

                let id: u8 = fields[0].parse()?;

                let (new_battery, sensor_type) = {
                    let field: u8 = fields[1].parse()?;
                    ((field / 128) != 0, field % 128)
                };

                let temp = {
                    let field1: u16 = fields[2].parse()?;
                    let field2: u16 = fields[3].parse()?;
                    let temp: u16 = (field1 << 8) + field2;
                    let temp: f32 = (temp as f32 - 1000.) / 10.;
                    temp
                };

                let (weak_battery, hum) = {
                    let field: u8 = fields[4].parse()?;
                    // first bit is weak battery flag
                    ((field & 0x80 != 0), field & 0x7F)
                };

                Ok(JeeLinkFrame {
                    id,
                    sensor_type,
                    new_battery,
                    weak_battery,
                    temperature: temp,
                    humidity: hum,
                })
            }
        }

        impl JeeLinkFrame {
            /// Validate string to be parsable as a Frame object.
            fn validate(s: &str) -> Result<(), FrameValidation> {
                if !s.chars().all(|c| {
                    c.is_numeric() || c.is_whitespace() || c.is_ascii_control() || c.is_control()
                }) {
                    return Err(FrameValidation::InvalidChars(s.to_string()));
                }
                if s.chars().filter(|c| c.is_whitespace()).count() != 4 {
                    return Err(FrameValidation::WrongNumberOfFields(s.to_string()));
                }
                Ok(())
            }
        }

        impl Display for JeeLinkFrame {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
            f,
            "Sensor {:2}: Temperatur {:4}, Humidity {:2}, weak battery: {}, new battery: {}",
            self.id, self.temperature, self.humidity, self.weak_battery, self.new_battery
        )
            }
        }

        #[cfg(test)]
        mod test {
            use super::{Frame, FrameCheckError, JeeLinkFrame};
            use bytes::BytesMut;

            #[test]
            fn test_frame_parsing() {
                let frame = JeeLinkFrame::parse(BytesMut::from(&b"50 1 4 193 65"[..])).unwrap();
                assert_eq!(
                    frame,
                    JeeLinkFrame {
                        id: 50,
                        sensor_type: 1,
                        new_battery: false,
                        weak_battery: false,
                        temperature: 21.7,
                        humidity: 65
                    }
                );
            }

            #[test]
            fn test_frame_check_detects_incomplete_frame() {
                assert_eq!(
                    JeeLinkFrame::check(&mut BytesMut::from(&b"OK 9 93 954 29"[..])),
                    Err(FrameCheckError::Incomplete)
                );
            }

            #[test]
            fn test_frame_check_extracts_frame_data() {
                assert_eq!(
                    JeeLinkFrame::check(&mut BytesMut::from(
                        &b"45 2 5OK 9 93 954 29\r\nOK 9 25 24 63\r\n"[..]
                    )),
                    Ok(BytesMut::from(&b"93 954 29"[..]))
                )
            }

            #[test]
            fn test_frame_check_drops_frame_from_read_buffer() {
                let mut buf = BytesMut::from(&b"45 2 5OK 9 93 954 29\r\nOK 9 25 24 63\r\n"[..]);
                let _ = JeeLinkFrame::check(&mut buf);
                assert_eq!(buf, &b"OK 9 25 24 63\r\n"[..]);
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
