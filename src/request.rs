#[derive(Debug)]
pub(crate) enum Request {
    SetVolume(f32),
    SetMuted(bool),
}
