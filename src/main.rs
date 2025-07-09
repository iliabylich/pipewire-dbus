use anyhow::Result;
use dbus::DBus;
use event::Event;

mod dbus;
mod event;
mod pipewire;
mod warmup;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    warmup::play_silence().await?;

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
