#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(deprecated_in_future)]
#![warn(unused_lifetimes)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::map_unwrap_or)]
#![expect(clippy::redundant_pub_crate)]
#![expect(clippy::future_not_send)]

use anyhow::Result;
use dbus::DBus;
use event::Event;
use pipewire::Pipewire;

mod dbus;
mod event;
mod pipewire;
mod warmup;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    warmup::play_silence()?;

    let connection = DBus::connect().await?;
    let pipewire = Pipewire::connect()?;

    loop {
        for event in pipewire.wait_and_dispatch().await? {
            DBus::handle_event(&connection, event).await?;
        }
    }
}
