use crate::Event;
use anyhow::{Context as _, Result};
use zbus::{Connection, interface};

#[derive(Default)]
struct Attribute<T>(Option<T>);

impl<T> Attribute<T>
where
    T: Clone + Copy + PartialEq,
{
    fn write(&mut self, new: T) -> Option<T> {
        if let Some(prev) = self.0 {
            if prev != new {
                self.0 = Some(new);
                Some(prev)
            } else {
                None
            }
        } else {
            // initial call doesn't emit any DBus property changes
            self.0 = Some(new);
            None
        }
    }
}

pub(crate) struct DBus {
    connection: Connection,
}

#[derive(Default)]
struct PipewireDBusState {
    volume: Attribute<u32>,
    muted: Attribute<bool>,
}

#[interface(name = "org.local.PipewireDBus")]
impl PipewireDBusState {
    #[zbus(property)]
    async fn volume(&self) -> u32 {
        self.volume.0.unwrap_or_default()
    }

    #[zbus(property)]
    async fn muted(&self) -> bool {
        self.muted.0.unwrap_or_default()
    }
}

impl DBus {
    pub(crate) async fn handle_event(&self, event: Event) -> Result<()> {
        let iface = self
            .connection
            .object_server()
            .interface::<_, PipewireDBusState>("/org/local/PipewireDBus")
            .await?;

        {
            let mut obj = iface.get_mut().await;

            match event {
                Event::Volume(volume) => {
                    if let Some(volume_was) = obj.volume.write(volume) {
                        log::info!("volume: {volume_was} -> {volume}");

                        obj.volume_changed(iface.signal_emitter())
                            .await
                            .context("failed to notify DBus about volume changes")?;
                    }
                }
                Event::Mute(muted) => {
                    if let Some(muted_was) = obj.muted.write(muted) {
                        log::info!("muted: {muted_was} -> {muted}");

                        obj.muted_changed(iface.signal_emitter())
                            .await
                            .context("failed to notify DBus about muted changes")?;
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn connect() -> Result<Self> {
        let connection = Connection::session().await?;

        connection
            .object_server()
            .at("/org/local/PipewireDBus", PipewireDBusState::default())
            .await?;
        connection.request_name("org.local.PipewireDBus").await?;

        Ok(Self { connection })
    }
}
