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
        id: u8,
        sensor_type: u8,
        new_battery: bool,
        weak_battery: bool,
        temperature: f32,
        humidity: u8,
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
