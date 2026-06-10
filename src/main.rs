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
