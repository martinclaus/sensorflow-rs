use sensorflow::{SerialPortListener, BAUD_RATE};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(1000);

static DEVICE: &str = "/dev/tty.usbserial-AL006PX8";

fn main() -> std::io::Result<()> {
    println!("Open port on device");
    let mut reader = SerialPortListener::new(
        serialport::new(DEVICE, BAUD_RATE)
            .timeout(TIMEOUT)
            .open_native()?,
    );
    println!("Ready to read");
    loop {
        match reader.read_frame() {
            Ok(Some(frame)) => println!("{frame}"),
            Ok(None) => (),
            Err(e) => eprintln!("{}", e),
            //     if let Some(frame) = frame {
            //     }
        }
    }
}
