#[derive(Debug)]
pub(crate) enum Event {
    Volume(u32),
    Mute(bool),
}
