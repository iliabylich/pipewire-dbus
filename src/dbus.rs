use crate::Event;
use anyhow::{Context as _, Result};
use zbus::{Connection, interface};

#[derive(Default)]
pub(crate) struct DBus {
    volume: u32,
    muted: bool,
}

#[interface(name = "org.local.PipewireDBus")]
impl DBus {
    #[zbus(property)]
    async fn volume(&self) -> u32 {
        self.volume
    }

    #[zbus(property)]
    async fn muted(&self) -> bool {
        self.muted
    }
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
                if volume != obj.volume {
                    log::info!("volume: {} -> {volume}", obj.volume);
                    obj.volume = volume;

                    obj.volume_changed(iface.signal_emitter())
                        .await
                        .context("failed to notify DBus about volume changes")?;
                }
            }
            if let Some(muted) = event.muted_changed {
                if muted != obj.muted {
                    log::info!("muted: {} -> {muted}", obj.muted);
                    obj.muted = muted;

                    obj.muted_changed(iface.signal_emitter())
                        .await
                        .context("failed to notify DBus about muted changes")?;
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn connect() -> Result<Connection> {
        let connection = Connection::session().await?;

        connection
            .object_server()
            .at("/org/local/PipewireDBus", DBus::default())
            .await?;
        connection.request_name("org.local.PipewireDBus").await?;

        Ok(connection)
    }
}
