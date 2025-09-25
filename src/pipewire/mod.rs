use crate::Event;
use anyhow::{Context as _, Result};
use pipewire::{
    context::ContextRc,
    main_loop::MainLoopRc,
    metadata::Metadata,
    node::Node,
    registry::{GlobalObject, RegistryBox},
    spa::{param::ParamType, pod::Pod, utils::dict::DictRef},
};
use sink_props::SinkProps;
use std::mem::MaybeUninit;
use store::{Store, store};
use tokio::sync::mpsc::Sender;

mod sink_props;
mod store;

pub(crate) fn start(tx: Sender<Event>) {
    std::thread::spawn(|| {
        let Ok(mainloop) = MainLoopRc::new(None)
            .inspect_err(|err| log::error!("failed to instantiate PW loop: {err:?}"))
        else {
            return;
        };

        let Ok(context) = ContextRc::new(&mainloop, None)
            .inspect_err(|err| log::error!("failed to construct context: {err:?}"))
        else {
            return;
        };

        let Ok(core) = context
            .connect(None)
            .inspect_err(|err| log::error!("failed to get core: {err:?}"))
        else {
            return;
        };

        let Ok(registry) = core
            .get_registry()
            .inspect_err(|err| log::error!("failed to get registry: {err:?}"))
        else {
            return;
        };

        let registry: RegistryBox<'static> =
            unsafe { std::mem::transmute::<_, RegistryBox<'static>>(registry) };

        if let Err(err) = Pipewire::start(tx, mainloop, registry) {
            log::error!("Failed to start PW loop: {:?}", err);
        }
    });
}

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

struct Pipewire {
    registry: RegistryBox<'static>,
    tx: Sender<Event>,
}

static mut PIPEWIRE: MaybeUninit<Pipewire> = MaybeUninit::zeroed();

fn pw() -> &'static mut Pipewire {
    unsafe {
        #[expect(static_mut_refs)]
        PIPEWIRE.assume_init_mut()
    }
}

impl Pipewire {
    pub(crate) fn start(
        tx: Sender<Event>,
        mainloop: MainLoopRc,
        registry: RegistryBox<'static>,
    ) -> Result<()> {
        Store::init();
        Self::init(tx, registry)?;

        let _listener = add_global_listener();

        mainloop.run();

        Ok(())
    }

    fn init(tx: Sender<Event>, registry: RegistryBox<'static>) -> Result<()> {
        let pipewire = Self::try_new(tx, registry)?;
        unsafe {
            #[expect(static_mut_refs)]
            PIPEWIRE.write(pipewire);
        }
        Ok(())
    }

    fn try_new(tx: Sender<Event>, registry: RegistryBox<'static>) -> Result<Self> {
        Ok(Self { registry, tx })
    }
}

fn add_global_listener() -> pipewire::registry::Listener {
    pw().registry
        .add_listener_local()
        .global(|object| {
            try_or_log!(
                on_global_object_added(object),
                "failed to track new global object"
            )
        })
        .global_remove(on_global_object_removed)
        .register()
}

fn on_global_object_added(object: &GlobalObject<&DictRef>) -> Result<()> {
    let Some(props) = object.props else {
        return Ok(());
    };

    if props.get("metadata.name") == Some("default") {
        let metadata: Metadata = pw().registry.bind(object).context("not a Metadata")?;
        on_metadata_object_added(object.id, metadata);
    }

    if props.get("media.class") == Some("Audio/Sink") {
        let node: Node = pw().registry.bind(object).context("not a Node")?;
        let name = props.get("node.name").context("no node.name")?;
        on_audio_sink_added(object.id, node, name);
    }

    Ok(())
}

fn on_metadata_object_added(id: u32, metadata: Metadata) {
    let listener = metadata
        .add_listener_local()
        .property(|_subject, key, _type, value| {
            if let Some((key, value)) = key.zip(value) {
                try_or_log!(
                    on_metadata_prop_changed(key, value),
                    "failed to process metadata prop change"
                );
            }
            0
        })
        .register();

    store().add_metadata(id, metadata);
    store().add_listener(id, Box::new(listener));
}

fn on_metadata_prop_changed(key: &str, value: &str) -> Result<()> {
    if key == "default.audio.sink" {
        #[derive(serde::Deserialize)]
        struct Value {
            name: String,
        }

        let Value { name } =
            serde_json::from_str(value).context("malformed JSON in default.audio.sink.value")?;
        log::info!("default sink changed: {name}");
        store().update_default_sink(name);
    }

    Ok(())
}

fn on_audio_sink_added(id: u32, node: Node, name: &str) {
    log::info!("audio sink added {id} {name}");

    node.subscribe_params(&[ParamType::Props]);
    let listener = node
        .add_listener_local()
        .param(move |_, _, _, _, param| {
            if let Some(param) = param {
                try_or_log!(
                    on_audio_sink_prop_changed(id, param),
                    "failed to track sink property change"
                )
            }
        })
        .register();

    store().add_sink(id, node, name);
    store().add_listener(id, Box::new(listener));
}

fn on_audio_sink_prop_changed(id: u32, param: &Pod) -> Result<()> {
    if !store().is_default_sink(id) {
        return Ok(());
    }

    let sink_props = SinkProps::try_from(param)?;

    if let Some(volume) = sink_props.volume {
        send(Event::Volume(volume))?;
    }
    if let Some(muted) = sink_props.muted {
        send(Event::Mute(muted))?;
    }

    Ok(())
}

fn send(event: Event) -> Result<()> {
    pw().tx
        .blocking_send(event)
        .context("failed to send event: channel is closed")
}

fn on_global_object_removed(id: u32) {
    store().remove(id);
}
