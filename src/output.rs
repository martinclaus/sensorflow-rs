//! Adapter for data output

pub mod influx {
    use std::fmt;

    pub enum LineProtocolValue {
        Float(f64),
        Integer(i64),
        UInteger(u64),
        String(String),
        Boolean(bool),
        Tag(String),
    }

    impl fmt::Display for LineProtocolValue {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Float(x) => write!(f, "{}", x),
                Self::Integer(x) => write!(f, "{}i", x),
                Self::UInteger(x) => write!(f, "{}u", x),
                Self::String(x) => write!(f, "\"{}\"", x),
                Self::Boolean(x) => write!(f, "{}", x),
                Self::Tag(x) => write!(f, "{}", x),
            }
        }
    }

    impl From<i64> for LineProtocolValue {
        fn from(x: i64) -> Self {
            LineProtocolValue::Integer(x)
        }
    }

    impl From<u64> for LineProtocolValue {
        fn from(x: u64) -> Self {
            LineProtocolValue::UInteger(x)
        }
    }

    impl From<f64> for LineProtocolValue {
        fn from(x: f64) -> Self {
            LineProtocolValue::Float(x)
        }
    }

    impl From<&str> for LineProtocolValue {
        fn from(x: &str) -> Self {
            LineProtocolValue::String(x.into())
        }
    }

    impl From<bool> for LineProtocolValue {
        fn from(x: bool) -> Self {
            LineProtocolValue::Boolean(x)
        }
    }

    struct Item(String, LineProtocolValue);

    impl fmt::Display for Item {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}={}", self.0, self.1)
        }
    }

    pub struct LineProtocol {
        measurement: String,
        tags: Vec<Item>,
        values: Vec<Item>,
    }

    impl LineProtocol {
        pub fn new(measurement: impl Into<String>) -> LineProtocol {
            return LineProtocol {
                measurement: measurement.into(),
                tags: vec![],
                values: vec![],
            };
        }

        pub fn add_tag(mut self, name: impl Into<String>, tag: impl fmt::Display) -> LineProtocol {
            self.tags.push(Item(
                name.into(),
                LineProtocolValue::Tag(format!("{}", tag)),
            ));
            self
        }

        pub fn add_value<V>(mut self, name: impl Into<String>, value: V) -> LineProtocol
        where
            V: Into<LineProtocolValue>,
        {
            self.values.push(Item(name.into(), value.into()));
            self
        }
    }

    impl fmt::Display for LineProtocol {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut tag_string = "".to_string();
            tag_string.extend(self.tags.iter().map(|item| format!(",{}", item)));

            let value_string = self
                .values
                .iter()
                .map(|item| format!("{}", item))
                .collect::<Vec<_>>()
                .join(",");
            write!(f, "{}{} {}", self.measurement, tag_string, value_string)
        }
    }

    #[cfg(test)]
    mod test {
        use super::LineProtocol;

        #[test]
        fn line_protocol_fmt() {
            let line = LineProtocol::new("measurement1");
            assert_eq!(format!("{}", line), "measurement1 ");

            assert_eq!(
                format!(
                    "{}",
                    LineProtocol::new("measurement1")
                        .add_value("keyI64", 1i64)
                        .add_value("keyU64", 1u64)
                        .add_value("keyStr", "value")
                        .add_value("keyBool", true)
                        .add_value("keyF64", 1.1)
                ),
                "measurement1 keyI64=1i,keyU64=1u,keyStr=\"value\",keyBool=true,keyF64=1.1"
            );

            assert_eq!(
                format!(
                    "{}",
                    LineProtocol::new("measurement1")
                        .add_tag("tag1", "1")
                        .add_tag("tag2", "something")
                ),
                "measurement1,tag1=1,tag2=something "
            );
        }
    }
}
