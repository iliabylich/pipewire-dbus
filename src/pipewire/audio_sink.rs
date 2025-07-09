use crate::{Event, pipewire::Store};
use anyhow::{Context as _, Result, anyhow, bail, ensure};
use pipewire::{
    node::Node,
    spa::{
        param::ParamType,
        pod::{Pod, Value, ValueArray, deserialize::PodDeserializer},
        sys::{SPA_PROP_channelVolumes, SPA_PROP_mute},
        utils::dict::DictRef,
    },
};
use tokio::sync::mpsc::Sender;

pub(crate) struct AudioSink;

impl AudioSink {
    pub(crate) fn added(
        id: u32,
        props: &DictRef,
        node: Node,
        event_tx: Sender<Event>,
    ) -> Result<()> {
        let sink_name = props.get("node.name").context("no sink.name")?;
        let device_id = props
            .get("device.id")
            .context("no device.id")?
            .parse::<u32>()
            .context("device.id is not a number")?;

        node.subscribe_params(&[ParamType::Props]);
        let listener = node
            .add_listener_local()
            .param(move |_, _, _, _, param| {
                if let Some(param) = param {
                    if let Err(err) = Self::prop_changed(param, event_tx.clone()) {
                        log::error!("Failed to track sink prop change: {:?}", err);
                    }
                } else {
                    // ignore
                }
            })
            .register();

        Store::register_sink(id, sink_name, device_id, node).context("failed to register sink")?;
        Store::register_listener(id, Box::new(listener)).context("failed to register listener")?;

        Ok(())
    }

    fn prop_changed(param: &Pod, event_tx: Sender<Event>) -> Result<()> {
        let (_, value) = PodDeserializer::deserialize_any_from(param.as_bytes())
            .map_err(|err| anyhow!("Failed to parse sink node's route param: {:?}", err))?;

        let Value::Object(object) = value else {
            bail!("Pod is not an Object");
        };

        for prop in object.properties {
            if prop.key == SPA_PROP_channelVolumes {
                if let Value::ValueArray(ValueArray::Float(floats)) = prop.value {
                    ensure!(
                        floats.len() == 2,
                        "channelVolumes must contain exactly two elements"
                    );
                    let volume = (floats[0] + floats[1]) / 2.0;
                    // convert to linear
                    let volume = volume.powf(1.0 / 3.0);
                    // round
                    let volume = (volume * 100.0) as u32;
                    Self::send(&event_tx, Event::Volume(volume));
                } else {
                    bail!("channelVolumes must be an Array of Floats");
                }
            } else if prop.key == SPA_PROP_mute {
                if let Value::Bool(muted) = prop.value {
                    Self::send(&event_tx, Event::Mute(muted));
                } else {
                    bail!("mute must be Bool");
                }
            }
        }

        Ok(())
    }

    fn send(tx: &Sender<Event>, event: Event) {
        if let Err(err) = tx.blocking_send(event) {
            log::error!("Failed to send event, channel is closed: {:?}", err);
        }
    }
}
