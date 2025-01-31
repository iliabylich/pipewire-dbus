use crate::{Event, Request};
use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use zbus::{interface, object_server::SignalEmitter, Connection};

pub(crate) struct DBus {
    volume: f32,
    muted: bool,
    request_tx: tokio::sync::mpsc::Sender<Request>,
}

#[interface(name = "org.local.PipewireDBus")]
impl DBus {
    async fn get_volume(&self) -> f32 {
        log::info!("Received GetVolume");

        self.volume
    }

    async fn set_volume(&mut self, volume: f32) {
        log::info!("Received SetVolume({volume})");

        if let Err(err) = self.request_tx.send(Request::SetVolume(volume)).await {
            log::error!("Failed to process request, pipe is closed: {:?}", err);
        }
    }

    #[zbus(signal)]
    async fn volume_updated(emitter: &SignalEmitter<'_>, volume: f32) -> zbus::Result<()>;

    async fn get_muted(&self) -> bool {
        log::info!("Received GetMuted");

        self.muted
    }

    async fn set_muted(&mut self, muted: bool) {
        log::info!("Received SetMuted({muted})");

        if let Err(err) = self.request_tx.send(Request::SetMuted(muted)).await {
            log::error!("Failed to process request, pipe is closed: {:?}", err);
        }
    }

    #[zbus(signal)]
    async fn muted_updated(emitter: &SignalEmitter<'_>, muted: bool) -> zbus::Result<()>;
}

impl DBus {
    pub(crate) async fn handle_event(connection: &Connection, event: Event) -> Result<()> {
        let iface = connection
            .object_server()
            .interface::<_, DBus>("/org/local/PipewireDBus")
            .await?;

        match event {
            Event::VolumeChanged(volume) => {
                log::info!("Emitting PW event {:?}", event);

                iface.get_mut().await.volume = volume;
                iface.volume_updated(volume).await?;
            }
            Event::MuteChanged(muted) => {
                if iface.get().await.muted != muted {
                    log::info!("Emitting PW event {:?}", event);

                    iface.get_mut().await.muted = muted;
                    iface.muted_updated(muted).await?;
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn connect() -> Result<(Connection, Receiver<Request>)> {
        let connection = Connection::session().await?;
        let (request_tx, request_rx) = tokio::sync::mpsc::channel::<Request>(100);

        connection
            .object_server()
            .at(
                "/org/local/PipewireDBus",
                DBus {
                    volume: 0.0,
                    muted: false,
                    request_tx,
                },
            )
            .await?;
        connection.request_name("org.local.PipewireDBus").await?;

        Ok((connection, request_rx))
    }
}
