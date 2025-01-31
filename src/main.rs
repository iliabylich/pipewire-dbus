use anyhow::Result;
use dbus::DBus;
use event::Event;
use request::Request;

mod dbus;
mod event;
mod pipewire;
mod request;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<Event>(100);
    let (connection, request_rx) = DBus::connect().await?;

    std::thread::spawn(move || {
        pipewire::start(event_tx, request_rx);
    });

    while let Some(event) = event_rx.recv().await {
        DBus::handle_event(&connection, event).await?;
    }

    Ok(())
}
