use serde::{Deserialize, Serialize};

use super::ids::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum AudioMode {
    Auto,
    Single(u64),
    Multi(Vec<u64>),
}

impl AudioMode {
    pub fn brief_name(&self) -> &'static str {
        match self {
            AudioMode::Auto => "Auto",
            AudioMode::Single(_) => "Single",
            AudioMode::Multi(_) => "Multi",
        }
    }
}

impl Default for AudioMode {
    fn default() -> Self {
        AudioMode::Auto
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct TitleClipIndex {
    pub title: TitleId,
    pub clip: u64,

    pub audio_mode: AudioMode,
}

impl TitleClipIndex {
    pub fn new(t: TitleId, idx: u64) -> TitleClipIndex {
        TitleClipIndex {
            title: t,
            clip: idx,
            audio_mode: Default::default(), //       extra: std::collections::HashMap::new(),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct PlaylistClipIndex {
    pub playlist: PlaylistId,
    pub clip: u64,

    pub audio_mode: AudioMode,
    //#[serde(flatten)]
    //extra: std::collections::HashMap<String, serde_json::Value>,
}

impl PlaylistClipIndex {
    pub fn new(p: PlaylistId, idx: u64) -> PlaylistClipIndex {
        PlaylistClipIndex {
            playlist: p,
            clip: idx,
            audio_mode: Default::default(), //       extra: std::collections::HashMap::new(),
        }
    }
    pub fn full(p: PlaylistId, idx: u64, audio_mode: AudioMode) -> PlaylistClipIndex {
        PlaylistClipIndex {
            playlist: p,
            clip: idx,
            audio_mode,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum BlurayExtract {
    PlaylistFull(PlaylistId),
    PlaylistFromToChap(PlaylistId, u64, u64), // including encluding
    PlaylistClipIndex(PlaylistClipIndex),
}

impl BlurayExtract {
    pub fn brief_name(&self) -> &'static str {
        match self {
            BlurayExtract::PlaylistFull(_) => "PlaylistFull",
            BlurayExtract::PlaylistFromToChap(_, _, _) => "PlaylistFromToChap",
            BlurayExtract::PlaylistClipIndex(_) => "PlaylistClipIndex",
        }
    }
}
impl Default for BlurayExtract {
    fn default() -> Self {
        BlurayExtract::PlaylistFull(PlaylistId::default())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct SingularRemuxMatroskaFile {
    pub name: String,
    pub src: String,

    pub extract: BlurayExtract,
}

impl SingularRemuxMatroskaFile {
    pub fn new(name: String, src: String, new_extract: BlurayExtract) -> Self {
        Self {
            name,
            src,
            extract: new_extract,
        }
    }
    pub fn flatten_only(name: String, src: String, new_extract: BlurayExtract) -> Self {
        Self {
            name,
            src,
            extract: new_extract,
        }
    }
}

impl Default for SingularRemuxMatroskaFile {
    fn default() -> Self {
        Self {
            name: Default::default(),
            src: Default::default(),
            extract: BlurayExtract::PlaylistFull(PlaylistId::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct ClipSplit {
    pub name: String,
    pub src: String,

    pub playlist: PlaylistId,

    pub format_start: u64,

    pub format_minwidth: u64,

    pub max_cnt: u64,
}

impl Default for ClipSplit {
    fn default() -> Self {
        Self {
            format_start: 1,
            format_minwidth: 0,
            max_cnt: 0,
            name: Default::default(),
            src: Default::default(),
            playlist: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum RemuxFolderEntrie {
    SingularFile(SingularRemuxMatroskaFile),
    MultipleFilePlaylistClipSplit(ClipSplit),
}
impl RemuxFolderEntrie {
    pub fn src(&self) -> &str {
        match self {
            RemuxFolderEntrie::SingularFile(e) => &e.src,
            RemuxFolderEntrie::MultipleFilePlaylistClipSplit(e) => &e.src,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            RemuxFolderEntrie::SingularFile(e) => &e.name,
            RemuxFolderEntrie::MultipleFilePlaylistClipSplit(e) => &e.name,
        }
    }
    pub fn set_name(&mut self, name: String) {
        match self {
            RemuxFolderEntrie::SingularFile(e) => e.name = name,
            RemuxFolderEntrie::MultipleFilePlaylistClipSplit(e) => e.name = name,
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Deserialize, Debug, Hash)]
pub struct RemuxFolder {
    pub name: String,
    pub entries: Vec<RemuxFolderEntrie>,

    pub file_prefix: String,

    pub show: bool,
    pub full_load: bool,
}

impl Default for RemuxFolder {
    fn default() -> Self {
        Self {
            full_load: false,
            show: false,
            name: String::new(),
            entries: Default::default(),
            file_prefix: "[DUMMY]".to_string(),
        }
    }
}

impl RemuxFolder {
    pub fn sort_entries(&mut self) {
        self.entries
            .sort_by(|a, b| a.src().partial_cmp(b.src()).unwrap());
    }
    pub fn sort_entries_name(&mut self) {
        self.entries
            .sort_by(|a, b| a.name().partial_cmp(b.name()).unwrap());
    }

    pub fn migrate(&mut self) {}
}
