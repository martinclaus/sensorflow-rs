use clap::{Parser, ValueEnum};
use sensorflow::{devices::JeeLink, output::influx::LineProtocol};
use tokio_serial::SerialPortBuilderExt;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// Input device to read from
    // #[arg(long, short)]
    device: String,

    /// Input protocol
    #[arg(long, value_enum, default_value_t=ProtoEnum::Jeelink)]
    input: ProtoEnum,

    /// Output protocol
    #[arg(long, value_enum, default_value_t=OutEnum::Stringify)]
    output: OutEnum,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ProtoEnum {
    /// Jeelink v3
    Jeelink,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutEnum {
    /// Stringify
    Stringify,
    /// InfluxDB Line Protocol
    Influxdb,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut port = tokio_serial::new(cli.device, JeeLink::get_baud_rate()).open_native_async()?;

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Failed to set serial port to exclusive.");

    let mut reader = match cli.input {
        ProtoEnum::Jeelink => JeeLink::new(port),
    };

    loop {
        let res = reader.read_frame().await;
        match res {
            Ok(Some(frame)) => match cli.output {
                OutEnum::Stringify => println!("{frame}"),
                OutEnum::Influxdb => println!("{}", LineProtocol::from(frame)),
            },
            Ok(_) => (),
            Err(e) => Err(e)?,
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
