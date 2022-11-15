use clap::{Parser, ValueEnum};
use sensorflow::{
    devices::{self, jeelink::JeeLinkFrame},
    output::influx::LineProtocol,
    FramedListener,
};
use tokio_serial::SerialStream;

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
    let Cli {
        device,
        input,
        output,
    } = Cli::parse();

    let mut reader = make_reader(input, device)?;

    loop {
        let res = reader.read_frame().await;
        match res {
            Ok(Some(frame)) => println!("{}", to_output(output, frame)),
            Ok(_) => (),
            Err(e) => Err(e)?,
        }
    }
}

fn make_reader(
    input: ProtoEnum,
    path: String,
) -> anyhow::Result<FramedListener<SerialStream, JeeLinkFrame>> {
    match input {
        ProtoEnum::Jeelink => devices::jeelink::new(path),
    }
}

fn to_output(output: OutEnum, frame: JeeLinkFrame) -> String {
    match output {
        OutEnum::Stringify => frame.to_string(),
        OutEnum::Influxdb => LineProtocol::from(frame).to_string(),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
