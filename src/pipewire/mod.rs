use crate::Event;
use anyhow::{Context as _, Result};
use pipewire::{
    context::Context,
    core::Core,
    main_loop::MainLoop,
    metadata::Metadata,
    node::Node,
    registry::{GlobalObject, Registry},
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
        if let Err(err) = Pipewire::start(tx) {
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
    mainloop: MainLoop,
    #[expect(dead_code)]
    context: Context,
    #[expect(dead_code)]
    core: Core,
    registry: Registry,
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
    pub(crate) fn start(tx: Sender<Event>) -> Result<()> {
        Store::init();
        Self::init(tx)?;

        let _listener = add_global_listener();

        pw().mainloop.run();

        Ok(())
    }

    fn init(tx: Sender<Event>) -> Result<()> {
        let pipewire = Self::try_new(tx)?;
        unsafe {
            #[expect(static_mut_refs)]
            PIPEWIRE.write(pipewire);
        }
        Ok(())
    }

    fn try_new(tx: Sender<Event>) -> Result<Self> {
        let mainloop = MainLoop::new(None).context("failed to instantiate PW loop")?;
        let context = Context::new(&mainloop).context("failed to construct context")?;
        let core = context.connect(None).context("failed to get core")?;
        let registry = core.get_registry().context("failed to get registry")?;

        Ok(Self {
            mainloop,
            context,
            core,
            registry,
            tx,
        })
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
