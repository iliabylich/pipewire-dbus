use anyhow::{Context as _, Result, bail};
use tokio::process::Command;

#[cfg(debug_assertions)]
const SILENCE_WAV: &str = "data/silence.wav";

#[cfg(not(debug_assertions))]
const SILENCE_WAV: &str = "/usr/share/pipewire-dbus/silence.wav";

pub(crate) async fn play_silence() -> Result<()> {
    log::info!("starting pipewire warmup");

    let output = Command::new("/usr/bin/pw-play")
        .arg(SILENCE_WAV)
        .output()
        .await
        .context("failed to warmup pipewire")?;

    if !output.status.success() {
        log::error!("pw-play has exited with non-zero status code");

        let stdout = String::from_utf8_lossy(&output.stdout);
        log::error!("stdout: {stdout}");

        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("stderr: {stderr}");

        bail!("failed to warmup pipewire")
    }

    log::info!("finished pipewire warmup");

    Ok(())
}
