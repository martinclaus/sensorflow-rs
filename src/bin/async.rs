use sensorflow::jeelink;
use tokio_serial::SerialPortBuilderExt;

static DEVICE: &str = "/dev/tty.usbserial-AL006PX8";

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let mut port = tokio_serial::new(DEVICE, jeelink::BAUD_RATE).open_native_async()?;

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Failed to set serial port to exclusive.");

    let mut reader = jeelink::new(port);

    while let Ok(frame) = reader.read_frame().await {
        match frame {
            Some(frame) => println!("{frame}"),
            None => (),
        }
    }

    return Ok(());
}
