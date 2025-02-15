use anyhow::Result;
use dbus::DBus;
use event::Event;

mod dbus;
mod event;
mod pipewire;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(100);
    let connection = DBus::connect().await?;

    std::thread::spawn(move || {
        pipewire::start(tx);
    });

    while let Some(event) = rx.recv().await {
        DBus::handle_event(&connection, event).await?;
    }

    Ok(())
}
