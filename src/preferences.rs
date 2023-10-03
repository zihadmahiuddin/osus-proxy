use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum BeatmapMirror {
    ServerDefault,
    #[default]
    Chimu,
    BeatConnect,
    Nerinyan,
    // catboy.best
}

impl BeatmapMirror {
    pub fn direct_download_link(&self, set_id: u32, with_video: bool) -> String {
        match self {
            BeatmapMirror::ServerDefault => unreachable!("This function should not be called on the server default variant"),
            BeatmapMirror::Chimu => format!("https://api.chimu.moe/d/{}", set_id),
            BeatmapMirror::BeatConnect => format!("https://beatconnect.io/b/{}", set_id),
            BeatmapMirror::Nerinyan => format!("https://api.nerinyan.moe/d/{}", set_id),
        }
    }
}

impl Display for BeatmapMirror {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BeatmapMirror::ServerDefault => {
                f.write_str("Server Default")
            }
            BeatmapMirror::Chimu => {
                f.write_str("chimu.moe")
            }
            BeatmapMirror::BeatConnect => {
                f.write_str("BeatConnect")
            }
            BeatmapMirror::Nerinyan => {
                f.write_str("nerinyan.moe")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Preferences {
    pub server_address: String,
    pub fake_supporter: bool,
    pub beatmap_mirror: BeatmapMirror,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            #[cfg(debug_assertions)]
            server_address: "cmyui.xyz".to_owned(),
            #[cfg(not(debug_assertions))]
            server_address: "ppy.sh".to_owned(),
            fake_supporter: true,
            beatmap_mirror: Default::default(),
        }
    }
}
