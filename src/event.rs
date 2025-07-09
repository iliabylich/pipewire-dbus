#[derive(Debug)]
pub(crate) struct Event {
    pub(crate) volume_changed: Option<u32>,
    pub(crate) muted_changed: Option<bool>,
}
