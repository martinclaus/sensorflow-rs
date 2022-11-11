use sensorflow::devices::JeeLink;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(1000);

static DEVICE: &str = "/dev/tty.usbserial-AL006PX8";

fn main() -> std::io::Result<()> {
    println!("Open port on device");
    let mut reader = JeeLink::new(
        serialport::new(DEVICE, JeeLink::get_baud_rate())
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
