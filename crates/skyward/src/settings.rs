use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use skyward_ui::app::{State, SCREEN_OFF, TIME_ZONES};
use skyward_ui::Lang;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::Path;

const DIRECTORY: &str = "/metadata/skyward";
const FILE: &str = "/metadata/skyward/settings.toml";
const TEMP: &str = "/metadata/skyward/settings.toml.new";

#[derive(Deserialize, Serialize, PartialEq)]
pub struct Settings {
    language: String,
    dark: bool,
    brightness: f32,
    screen_off: usize,
    time_zone: usize,
    vibrate: bool,
    vibration: f32,
    file_transfer: bool,
}

impl Settings {
    pub fn from_state(s: &State) -> Self {
        Self {
            language: s.lang.pick(["zh-CN", "zh-TW", "en"]).into(),
            dark: s.dark,
            brightness: s.brightness,
            screen_off: s.screen_off,
            time_zone: s.time_zone,
            vibrate: s.vibrate,
            vibration: s.vibration,
            file_transfer: s.file_transfer,
        }
    }

    pub fn load(s: &mut State) -> Result<()> {
        let text = match fs::read_to_string(FILE) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        let saved: Self = toml::from_str(&text)?;
        ensure!(saved.screen_off < SCREEN_OFF.len() && saved.time_zone < TIME_ZONES.len(), "invalid settings selection");
        ensure!((0.0..=1.0).contains(&saved.brightness) && (0.0..=1.0).contains(&saved.vibration), "invalid slider position");
        s.lang = Lang::for_locale(&saved.language)?;
        s.dark = saved.dark;
        s.brightness = saved.brightness;
        s.screen_off = saved.screen_off;
        s.time_zone = saved.time_zone;
        s.vibrate = saved.vibrate;
        s.vibration = saved.vibration;
        s.file_transfer = saved.file_transfer;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        ensure!(skyward_block::mount::is_mount_point(Path::new("/metadata"))?, "/metadata is not mounted");
        let text = toml::to_string(self)?;
        skyward_vab::metadata_writable(true)?;
        let saved = (|| -> Result<()> {
            fs::DirBuilder::new().recursive(true).mode(0o700).create(DIRECTORY)?;
            let mut file = OpenOptions::new().write(true).create(true).truncate(true)
                .mode(0o600).custom_flags(libc::O_NOFOLLOW).open(TEMP)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            fs::rename(TEMP, FILE)?;
            File::open(DIRECTORY)?.sync_all()?;
            File::open("/metadata")?.sync_all()?;
            Ok(())
        })();
        let read_only = skyward_vab::metadata_writable(false);
        read_only.and(saved)
    }
}
