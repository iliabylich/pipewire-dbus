use pipewire::{metadata::Metadata, node::Node, proxy::Listener};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

struct InnerStore {
    metadata: HashMap<u32, Metadata>,
    nodes: HashMap<u32, Node>,
    listeners: HashMap<u32, Vec<Box<dyn Listener>>>,
    default_sink: Option<String>,
    sink_id_to_name: HashMap<u32, String>,
}

impl InnerStore {
    fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            nodes: HashMap::new(),
            listeners: HashMap::new(),
            default_sink: None,
            sink_id_to_name: HashMap::new(),
        }
    }

    fn add_metadata(&mut self, id: u32, metadata: Metadata) {
        self.metadata.insert(id, metadata);
    }

    fn add_sink(&mut self, id: u32, node: Node, name: impl Into<String>) {
        self.nodes.insert(id, node);
        self.sink_id_to_name.insert(id, name.into());
    }

    fn add_listener(&mut self, obj_id: u32, listener: Box<dyn Listener>) {
        self.listeners.entry(obj_id).or_default().push(listener);
    }

    fn update_default_sink(&mut self, name: String) {
        self.default_sink = Some(name);
    }

    fn remove(&mut self, obj_id: u32) {
        self.sink_id_to_name.remove(&obj_id);
        self.listeners.remove(&obj_id);
        self.nodes.remove(&obj_id);
        self.metadata.remove(&obj_id);
    }

    fn is_default_sink(&self, id: u32) -> bool {
        let Some(default_sink_name) = self.default_sink.as_deref() else {
            return false;
        };
        let Some(sink_name) = self.sink_id_to_name.get(&id) else {
            return false;
        };
        sink_name == default_sink_name
    }
}

#[derive(Clone)]
pub(crate) struct Store(Rc<RefCell<InnerStore>>);

impl Store {
    pub(crate) fn new() -> Self {
        Self(Rc::new(RefCell::new(InnerStore::new())))
    }

    pub(crate) fn add_metadata(&self, id: u32, metadata: Metadata) {
        self.0.borrow_mut().add_metadata(id, metadata);
    }
    pub(crate) fn add_sink(&self, id: u32, node: Node, name: impl Into<String>) {
        self.0.borrow_mut().add_sink(id, node, name);
    }
    pub(crate) fn add_listener(&self, obj_id: u32, listener: Box<dyn Listener>) {
        self.0.borrow_mut().add_listener(obj_id, listener);
    }
    pub(crate) fn update_default_sink(&self, name: String) {
        self.0.borrow_mut().update_default_sink(name);
    }
    pub(crate) fn remove(&self, obj_id: u32) {
        self.0.borrow_mut().remove(obj_id);
    }
    pub(crate) fn is_default_sink(&self, id: u32) -> bool {
        self.0.borrow().is_default_sink(id)
    }
}
