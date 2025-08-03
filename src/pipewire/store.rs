use pipewire::{metadata::Metadata, node::Node, proxy::Listener};
use std::{collections::HashMap, mem::MaybeUninit};

pub(crate) struct Store {
    metadata: HashMap<u32, Metadata>,
    nodes: HashMap<u32, Node>,
    listeners: HashMap<u32, Vec<Box<dyn Listener>>>,
    default_sink: Option<String>,
    sink_id_to_name: HashMap<u32, String>,
}

static mut STORE: MaybeUninit<Store> = MaybeUninit::zeroed();

pub(crate) fn store() -> &'static mut Store {
    unsafe {
        #[expect(static_mut_refs)]
        STORE.assume_init_mut()
    }
}

impl Store {
    pub(crate) fn init() {
        let store = Store {
            metadata: HashMap::new(),
            nodes: HashMap::new(),
            listeners: HashMap::new(),
            default_sink: None,
            sink_id_to_name: HashMap::new(),
        };

        unsafe {
            #[expect(static_mut_refs)]
            STORE.write(store);
        }
    }

    pub(crate) fn add_metadata(&mut self, id: u32, metadata: Metadata) {
        self.metadata.insert(id, metadata);
    }

    pub(crate) fn add_sink(&mut self, id: u32, node: Node, name: impl Into<String>) {
        self.nodes.insert(id, node);
        self.sink_id_to_name.insert(id, name.into());
    }

    pub(crate) fn add_listener(&mut self, obj_id: u32, listener: Box<dyn Listener>) {
        self.listeners.entry(obj_id).or_default().push(listener);
    }

    pub(crate) fn update_default_sink(&mut self, name: String) {
        self.default_sink = Some(name);
    }

    pub(crate) fn remove(&mut self, obj_id: u32) {
        self.sink_id_to_name.remove(&obj_id);
        self.listeners.remove(&obj_id);
        self.nodes.remove(&obj_id);
        self.metadata.remove(&obj_id);
    }

    pub(crate) fn is_default_sink(&self, id: u32) -> bool {
        let Some(default_sink_name) = self.default_sink.as_ref() else {
            return false;
        };
        let Some(sink_name) = self.sink_id_to_name.get(&id) else {
            return false;
        };
        sink_name == default_sink_name
    }
}
