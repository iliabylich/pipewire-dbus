use anyhow::{anyhow, bail, ensure};
use pipewire::spa::{
    pod::{Pod, Value, ValueArray, deserialize::PodDeserializer},
    sys::{SPA_PROP_channelVolumes, SPA_PROP_mute},
};

pub(crate) struct SinkProps {
    pub(crate) volume: Option<u32>,
    pub(crate) muted: Option<bool>,
}

impl TryFrom<&Pod> for SinkProps {
    type Error = anyhow::Error;

    fn try_from(param: &Pod) -> Result<Self, Self::Error> {
        let (_, value) = PodDeserializer::deserialize_any_from(param.as_bytes())
            .map_err(|err| anyhow!("Failed to parse sink node's route param: {:?}", err))?;

        let Value::Object(object) = value else {
            bail!("Pod is not an Object");
        };

        let mut volume = None;
        let mut muted = None;

        for prop in object.properties {
            if prop.key == SPA_PROP_channelVolumes {
                if let Value::ValueArray(ValueArray::Float(floats)) = prop.value {
                    ensure!(
                        floats.len() == 2,
                        "channelVolumes must contain exactly two elements"
                    );
                    let value = (floats[0] + floats[1]) / 2.0;
                    // convert to linear
                    let value = value.powf(1.0 / 3.0);
                    // round
                    let value = (value * 100.0) as u32;
                    volume = Some(value);
                } else {
                    bail!("channelVolumes must be an Array of Floats");
                }
            } else if prop.key == SPA_PROP_mute {
                if let Value::Bool(value) = prop.value {
                    muted = Some(value)
                } else {
                    bail!("mute must be Bool");
                }
            }
        }

        Ok(Self { volume, muted })
    }
}
