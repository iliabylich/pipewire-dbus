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
use std::{cell::RefCell, rc::Rc, time::Duration};
use tokio::sync::mpsc::{error::TryRecvError, Receiver, Sender};

mod audio_device;
mod audio_sink;
mod command;
mod metadata_node;
mod store;

use store::Store;

use crate::{Event, Request};

pub(crate) fn start(event_tx: Sender<Event>, request_rx: Receiver<Request>) {
    std::thread::spawn(|| {
        if let Err(err) = start_pw_mainloop(event_tx, request_rx) {
            log::error!("Failed to start PW loop: {:?}", err);
        }
    });
}

fn start_pw_mainloop(event_tx: Sender<Event>, request_rx: Receiver<Request>) -> Result<()> {
    let mainloop = MainLoop::new(None).context("failed to instantiate PW loop")?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    Store::init_for_current_thread();
    let registry: &'static Registry = Box::leak(Box::new(core.get_registry()?));
    let request_rx = Rc::new(RefCell::new(request_rx));

    let _listener = start_pw_listener(registry, event_tx);

    let timer = mainloop.loop_().add_timer(move |_| {
        let rx = Rc::clone(&request_rx);
        loop {
            match rx.borrow_mut().try_recv() {
                Ok(Request::SetMuted(muted)) => command::dispatch(None, Some(muted)),
                Ok(Request::SetVolume(volume)) => command::dispatch(Some(volume), None),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("request receiver is closed"),
            }
        }
    });

    timer
        .update_timer(
            Some(Duration::from_millis(100)),
            Some(Duration::from_millis(100)),
        )
        .into_result()
        .context("invalid timer")?;

    mainloop.run();

    Ok(())
}

fn start_pw_listener(registry: &'static Registry, event_tx: Sender<Event>) -> Listener {
    registry
        .add_listener_local()
        .global(move |obj| {
            if let Err(err) = on_global_object_added(registry, obj, event_tx.clone()) {
                log::error!("Failed to track new global object: {:?}", err);
            }
        })
        .global_remove(on_global_object_removed)
        .register()
}

fn on_global_object_added(
    registry: &Registry,
    obj: &GlobalObject<&DictRef>,
    event_tx: Sender<Event>,
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
        audio_sink::AudioSink::added(obj.id, props, node, event_tx.clone())?;
    }

    Ok(())
}

fn on_global_object_removed(id: u32) {
    if let Err(err) = Store::remove(id) {
        log::error!("Failed to remove device: {:?}", err);
    }
}
