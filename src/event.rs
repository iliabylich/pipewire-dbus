#[derive(Debug)]
pub(crate) struct Event {
    pub(crate) volume_changed: Option<f32>,
    pub(crate) muted_changed: Option<bool>,
}
