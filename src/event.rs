#[derive(Debug)]
pub(crate) enum Event {
    VolumeChanged(f32),
    MuteChanged(bool),
}
