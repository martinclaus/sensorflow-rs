use sensorflow::devices::{self, Device};

static DEVICE: &str = "/dev/tty.usbserial-AL006PX8";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut reader = devices::JeeLink::new(DEVICE)?;

    while let Ok(frame) = reader.read_frame().await {
        match frame {
            Some(frame) => println!("{}", frame.to_string()),
            None => (),
        }
    }

    return Ok(());
}
