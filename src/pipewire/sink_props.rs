use anyhow::anyhow;
use pipewire::spa::{
    pod::{
        Pod, Value, ValueArray,
        deserialize::{
            DeserializeError, DeserializeSuccess, ObjectPodDeserializer, PodDeserialize,
            PodDeserializer, Visitor,
        },
    },
    sys::{SPA_PROP_channelVolumes, SPA_PROP_mute},
};
use std::convert::Infallible;

pub(crate) struct SinkProps {
    pub(crate) volume: Option<u32>,
    pub(crate) muted: Option<bool>,
}

impl TryFrom<&Pod> for SinkProps {
    type Error = anyhow::Error;

    fn try_from(param: &Pod) -> Result<Self, Self::Error> {
        let (_, props) = PodDeserializer::deserialize_from(param.as_bytes())
            .map_err(|err| anyhow!("Failed to parse sink node's route param: {:?}", err))?;

        Ok(props)
    }
}

impl<'de> PodDeserialize<'de> for SinkProps {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<(Self, DeserializeSuccess<'de>), DeserializeError<&'de [u8]>>
    where
        Self: Sized,
    {
        deserializer.deserialize_object(SinkPropsVisitor)
    }
}

struct SinkPropsVisitor;

impl<'de> Visitor<'de> for SinkPropsVisitor {
    type Value = SinkProps;
    type ArrayElem = Infallible;

    fn visit_object(
        &self,
        object: &mut ObjectPodDeserializer<'de>,
    ) -> Result<Self::Value, DeserializeError<&'de [u8]>> {
        let mut volume = None;
        let mut muted = None;

        while let Some((value, key, _flags)) = object.deserialize_property::<Value>()? {
            if key == SPA_PROP_channelVolumes {
                let Value::ValueArray(ValueArray::Float(floats)) = value else {
                    return Err(DeserializeError::UnsupportedType);
                };
                if floats.len() != 2 {
                    return Err(DeserializeError::InvalidType);
                }

                let value = (floats[0] + floats[1]) / 2.0;
                // convert to linear
                let value = value.powf(1.0 / 3.0);
                // round
                let value = (value * 100.0) as u32;
                volume = Some(value);
            } else if key == SPA_PROP_mute {
                let Value::Bool(value) = value else {
                    return Err(DeserializeError::UnsupportedType);
                };
                muted = Some(value);
            }
        }

        Ok(SinkProps { volume, muted })
    }
}
