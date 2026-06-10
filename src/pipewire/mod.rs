use crate::Event;
use anyhow::{Context as _, Result};
use pending_events::PendingEvents;
use pipewire::{
    context::ContextRc,
    core::CoreRc,
    loop_::Timeout,
    main_loop::MainLoopRc,
    metadata::Metadata,
    node::Node,
    registry::{GlobalObject, Listener as RegistryListener, RegistryRc},
    spa::{param::ParamType, pod::Pod, utils::dict::DictRef},
};
use sink_props::SinkProps;
use std::os::fd::{AsRawFd, RawFd};
use store::Store;
use tokio::io::unix::AsyncFd;

mod pending_events;
mod sink_props;
mod store;

macro_rules! try_or_log {
    ($e:expr, $msg:expr) => {
        match $e {
            Ok(()) => {}
            Err(err) => {
                log::error!("{}: {err:?}", $msg)
            }
        }
    };
}

pub(crate) struct Pipewire {
    fd: AsyncFd<RawFd>,
    mainloop: MainLoopRc,
    _context: ContextRc,
    _core: CoreRc,
    _registry: RegistryRc,
    _global_listener: RegistryListener,
    _store: Store,
    events: PendingEvents,
}

impl Pipewire {
    pub(crate) fn connect() -> Result<Self> {
        let mainloop = MainLoopRc::new(None)?;
        let fd = AsyncFd::new(mainloop.loop_().fd().as_raw_fd())?;

        let context = ContextRc::new(&mainloop, None)?;
        let core = context.connect_rc(None)?;
        let registry = core.get_registry_rc()?;

        let store = Store::new();
        let events = PendingEvents::new();

        Ok(Self {
            fd,
            mainloop,
            _context: context,
            _core: core,
            _registry: registry.clone(),
            _global_listener: add_global_listener(registry, store.clone(), events.clone()),
            _store: store,
            events,
        })
    }

    pub(crate) async fn wait_and_dispatch(&self) -> Result<Vec<Event>> {
        let mut guard = self
            .fd
            .readable()
            .await
            .context("failed to wait for PW loop fd")?;

        while self.mainloop.loop_().iterate(Timeout::None) > 0 {}

        guard.clear_ready();

        Ok(self.events.take())
    }
}

fn add_global_listener(
    registry: RegistryRc,
    store: Store,
    events: PendingEvents,
) -> pipewire::registry::Listener {
    registry
        .add_listener_local()
        .global({
            let registry = registry.clone();
            let store = store.clone();
            let events = events.clone();
            move |object| {
                try_or_log!(
                    on_global_object_added(object, registry.clone(), store.clone(), events.clone()),
                    "failed to track new global object"
                )
            }
        })
        .global_remove(move |id| on_global_object_removed(id, store.clone()))
        .register()
}

fn on_global_object_added(
    object: &GlobalObject<&DictRef>,
    registry: RegistryRc,
    store: Store,
    events: PendingEvents,
) -> Result<()> {
    let Some(props) = object.props else {
        return Ok(());
    };

    if props.get("metadata.name") == Some("default") {
        let metadata: Metadata = registry.bind(object).context("not a Metadata")?;
        on_metadata_object_added(object.id, metadata, store.clone());
    }

    if props.get("media.class") == Some("Audio/Sink") {
        let node: Node = registry.bind(object).context("not a Node")?;
        let name = props.get("node.name").context("no node.name")?;
        on_audio_sink_added(object.id, node, name, store, events);
    }

    Ok(())
}

fn on_metadata_object_added(id: u32, metadata: Metadata, store: Store) {
    let listener = metadata
        .add_listener_local()
        .property({
            let store = store.clone();
            move |_subject, key, _type, value| {
                if let Some((key, value)) = key.zip(value) {
                    try_or_log!(
                        on_metadata_prop_changed(key, value, store.clone()),
                        "failed to process metadata prop change"
                    );
                }
                0
            }
        })
        .register();

    store.add_metadata(id, metadata);
    store.add_listener(id, Box::new(listener));
}

fn on_metadata_prop_changed(key: &str, value: &str, store: Store) -> Result<()> {
    if key == "default.audio.sink" {
        let name = jzon::parse(value)?
            .get("name")
            .context("no name")?
            .as_str()
            .context("name is not a string")?
            .to_string();

        log::info!("default sink changed: {name}");
        store.update_default_sink(name);
    }

    Ok(())
}

fn on_audio_sink_added(id: u32, node: Node, name: &str, store: Store, events: PendingEvents) {
    log::info!("audio sink added {id} {name}");

    node.subscribe_params(&[ParamType::Props]);
    let listener = node
        .add_listener_local()
        .param({
            let store = store.clone();
            move |_, _, _, _, param| {
                if let Some(param) = param {
                    try_or_log!(
                        on_audio_sink_prop_changed(id, param, store.clone(), events.clone()),
                        "failed to track sink property change"
                    )
                }
            }
        })
        .register();

    store.add_sink(id, node, name);
    store.add_listener(id, Box::new(listener));
}

fn on_audio_sink_prop_changed(
    id: u32,
    param: &Pod,
    store: Store,
    events: PendingEvents,
) -> Result<()> {
    if !store.is_default_sink(id) {
        return Ok(());
    }

    let sink_props = SinkProps::try_from(param)?;

    if let Some(volume) = sink_props.volume {
        events.push(Event::Volume(volume));
    }
    if let Some(muted) = sink_props.muted {
        events.push(Event::Mute(muted));
    }

    Ok(())
}

fn on_global_object_removed(id: u32, store: Store) {
    store.remove(id);
}
