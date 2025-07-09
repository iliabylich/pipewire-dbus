use crate::Event;
use anyhow::{Context as _, Result};
use std::ops::Deref;
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

impl<T> Deref for Attribute<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub(crate) struct DBus {
    volume: Attribute<u32>,
    muted: Attribute<bool>,
}

impl DBus {
    fn set_volume(&mut self, volume: u32) -> Option<u32> {
        self.volume.write(volume)
    }

    fn set_muted(&mut self, muted: bool) -> Option<bool> {
        self.muted.write(muted)
    }
}

#[interface(name = "org.local.PipewireDBus")]
impl DBus {
    #[zbus(property)]
    async fn volume(&self) -> u32 {
        self.volume.unwrap_or_default()
    }

    #[zbus(property)]
    async fn muted(&self) -> bool {
        self.muted.unwrap_or_default()
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

            match event {
                Event::Volume(volume) => {
                    if let Some(volume_was) = obj.set_volume(volume) {
                        log::info!("volume: {volume_was} -> {volume}");

                        obj.volume_changed(iface.signal_emitter())
                            .await
                            .context("failed to notify DBus about volume changes")?;
                    }
                }
                Event::Mute(muted) => {
                    if let Some(muted_was) = obj.set_muted(muted) {
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
