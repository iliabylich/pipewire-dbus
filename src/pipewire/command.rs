use crate::pipewire::Store;
use anyhow::{Context, Result};
use pipewire::spa::{
    param::ParamType,
    pod::{serialize::PodSerializer, Object, Pod, Property, PropertyFlags, Value, ValueArray},
    sys::{
        SPA_PARAM_ROUTE_device, SPA_PARAM_ROUTE_index, SPA_PARAM_ROUTE_props, SPA_PARAM_Route,
        SPA_PROP_channelVolumes, SPA_PROP_mute,
    },
};

pub(crate) fn dispatch(volume: Option<f32>, muted: Option<bool>) {
    if let Err(err) = try_dispatch(volume, muted) {
        log::error!("Failed to dispatch to PW: {:?}", err);
    }
}

fn try_dispatch(volume: Option<f32>, muted: Option<bool>) -> Result<()> {
    Store::with_default_device(|device, route| {
        let mut props = vec![];

        if let Some(volume) = volume {
            // convert to cubic
            let volume = volume.powf(3.0);
            props.push(Property {
                key: SPA_PROP_channelVolumes,
                flags: PropertyFlags::empty(),
                value: Value::ValueArray(ValueArray::Float(vec![volume, volume])),
            });
        }

        if let Some(muted) = muted {
            props.push(Property {
                key: SPA_PROP_mute,
                flags: PropertyFlags::empty(),
                value: Value::Bool(muted),
            });
        }

        let values: Vec<u8> = PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &Value::Object(Object {
                type_: pipewire::spa::utils::SpaTypes::ObjectParamRoute.as_raw(),
                id: SPA_PARAM_Route,
                properties: vec![
                    Property {
                        key: SPA_PARAM_ROUTE_index,
                        flags: PropertyFlags::empty(),
                        value: Value::Int(route.0),
                    },
                    Property {
                        key: SPA_PARAM_ROUTE_device,
                        flags: PropertyFlags::empty(),
                        value: Value::Int(route.1),
                    },
                    Property {
                        key: SPA_PARAM_ROUTE_props,
                        flags: PropertyFlags::empty(),
                        value: Value::Object(Object {
                            type_: pipewire::spa::utils::SpaTypes::ObjectParamProps.as_raw(),
                            id: SPA_PARAM_Route,
                            properties: props,
                        }),
                    },
                ],
            }),
        )
        .context("invalid pod value")?
        .0
        .into_inner();
        let param = Pod::from_bytes(&values).context("invalid pod value")?;
        device.set_param(ParamType::Route, 0, param);

        Ok(())
    })
}
