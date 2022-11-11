use sensorflow::jeelink;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(1000);

static DEVICE: &str = "/dev/tty.usbserial-AL006PX8";

fn main() -> std::io::Result<()> {
    println!("Open port on device");
    let mut reader = jeelink::new(
        serialport::new(DEVICE, jeelink::BAUD_RATE)
            .timeout(TIMEOUT)
            .open_native()?,
    );
    println!("Ready to read");
    while let Ok(frame) = reader.read_frame() {
        match frame {
            Some(frame) => println!("{frame}"),
            None => (),
        }
    }

    Ok(())
}
