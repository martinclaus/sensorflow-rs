//! Read from IO devices.

use async_trait::async_trait;

pub use jeelink::JeeLink;

use crate::output::ToOutput;

pub mod jeelink;

#[async_trait]
pub trait Device {
    async fn read_frame(&mut self) -> anyhow::Result<Option<Box<dyn ToOutput>>>;
}
