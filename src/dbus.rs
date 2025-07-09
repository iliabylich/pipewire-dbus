use crate::Event;
use anyhow::Result;
use zbus::{Connection, interface, object_server::SignalEmitter};

pub(crate) struct DBus {
    volume: u32,
    muted: bool,
}

#[interface(name = "org.local.PipewireDBus")]
impl DBus {
    #[zbus(property(emits_changed_signal = "false"))]
    async fn data(&self) -> (u32, bool) {
        (self.volume, self.muted)
    }

    #[zbus(signal)]
    async fn data_changed(
        emitter: &SignalEmitter<'_>,
        volume: u32,
        muted: bool,
    ) -> zbus::Result<()>;
}

impl DBus {
    pub(crate) async fn handle_event(connection: &Connection, event: Event) -> Result<()> {
        let iface = connection
            .object_server()
            .interface::<_, DBus>("/org/local/PipewireDBus")
            .await?;

        {
            let mut obj = iface.get_mut().await;
            if let Some(volume) = event.volume_changed {
                log::info!("volume: {} -> {volume}", obj.volume);
                obj.volume = (volume * 100.0) as u32;
            }
            if let Some(muted) = event.muted_changed {
                log::info!("muted: {} -> {muted}", obj.muted);
                obj.muted = muted;
            }
            iface.data_changed(obj.volume, obj.muted).await?;
        }

        Ok(())
    }

    pub(crate) async fn connect() -> Result<Connection> {
        let connection = Connection::session().await?;

        connection
            .object_server()
            .at(
                "/org/local/PipewireDBus",
                DBus {
                    volume: 0,
                    muted: false,
                },
            )
            .await?;
        connection.request_name("org.local.PipewireDBus").await?;

        Ok(connection)
    }
}
