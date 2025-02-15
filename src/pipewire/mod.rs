use anyhow::{Context as _, Result};
use pipewire::{
    context::Context,
    device::Device,
    main_loop::MainLoop,
    metadata::Metadata,
    node::Node,
    registry::{GlobalObject, Listener, Registry},
    spa::utils::dict::DictRef,
};
use tokio::sync::mpsc::Sender;

mod audio_device;
mod audio_sink;
mod metadata_node;
mod store;

use store::Store;

use crate::Event;

pub(crate) fn start(tx: Sender<Event>) {
    std::thread::spawn(|| {
        if let Err(err) = start_pw_mainloop(tx) {
            log::error!("Failed to start PW loop: {:?}", err);
        }
    });
}

fn start_pw_mainloop(tx: Sender<Event>) -> Result<()> {
    let mainloop = MainLoop::new(None).context("failed to instantiate PW loop")?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    Store::init_for_current_thread();
    let registry: &'static Registry = Box::leak(Box::new(core.get_registry()?));

    let _listener = start_pw_listener(registry, tx);

    mainloop.run();

    Ok(())
}

fn start_pw_listener(registry: &'static Registry, tx: Sender<Event>) -> Listener {
    registry
        .add_listener_local()
        .global(move |obj| {
            if let Err(err) = on_global_object_added(registry, obj, tx.clone()) {
                log::error!("Failed to track new global object: {:?}", err);
            }
        })
        .global_remove(on_global_object_removed)
        .register()
}

fn on_global_object_added(
    registry: &Registry,
    obj: &GlobalObject<&DictRef>,
    tx: Sender<Event>,
) -> Result<()> {
    let Some(props) = obj.props else {
        // ignore empty objects
        return Ok(());
    };

    if props.get("metadata.name") == Some("default") {
        let metadata: Metadata = registry.bind(obj).context("not a Metadata")?;
        metadata_node::MetadataNode::added(obj.id, metadata)?;
    }

    if props.get("media.class") == Some("Audio/Device") {
        let device: Device = registry.bind(obj).context("not a Device")?;
        audio_device::AudioDevice::added(obj.id, device)?;
    }

    if props.get("media.class") == Some("Audio/Sink") {
        let node: Node = registry.bind(obj).context("not a Node")?;
        audio_sink::AudioSink::added(obj.id, props, node, tx.clone())?;
    }

    Ok(())
}

fn on_global_object_removed(id: u32) {
    if let Err(err) = Store::remove(id) {
        log::error!("Failed to remove device: {:?}", err);
    }
}
