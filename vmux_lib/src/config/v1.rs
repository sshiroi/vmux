use super::ids::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, PartialEq, Deserialize, Hash, Clone, Debug)]
struct V1TitleComment {
    index: PlaylistId,
    name: String,
    chapter_comments: Vec<(u64, String)>,
}

#[derive(Serialize, PartialEq, Deserialize, Hash, Clone, Debug)]
pub struct V1Bdrom {
    internal_id: String,
    title_comments: Vec<V1TitleComment>,
    general_comment: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
enum V1AudioMode {
    Auto,
    Single(u64),
    Multi(Vec<u64>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
struct V1TitleClipIndex {
    pub title: TitleId,
    pub clip: u64,

    pub audio_mode: V1AudioMode,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
struct V1PlaylistClipIndex {
    pub playlist: PlaylistId,
    pub clip: u64,
    pub audio_mode: V1AudioMode,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
enum V1BlurayExtract {
    PlaylistFull(PlaylistId),
    PlaylistFromToChap(PlaylistId, u64, u64), // including encluding
    PlaylistClipIndex(V1PlaylistClipIndex),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
struct V1SingularRemuxMatroskaFile {
    pub name: String,
    pub src: String,
    pub extract: V1BlurayExtract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
struct V1ClipSplit {
    pub name: String,
    pub src: String,
    pub playlist: PlaylistId,
    pub format_start: u64,
    pub format_minwidth: u64,
    pub max_cnt: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
enum V1RemuxFolderEntrie {
    SingularFile(V1SingularRemuxMatroskaFile),
    MultipleFilePlaylistClipSplit(V1ClipSplit),
}

#[derive(Serialize, Clone, PartialEq, Deserialize, Debug, Hash)]
pub struct V1RemuxFolder {
    pub name: String,
    pub file_prefix: String,
    entries: Vec<V1RemuxFolderEntrie>,
}

//Boilerplate
impl From<super::TitleComment> for V1TitleComment {
    fn from(t: super::TitleComment) -> Self {
        Self {
            index: t.index,
            name: t.name,
            chapter_comments: t.chapter_comments,
        }
    }
}
impl Into<super::TitleComment> for V1TitleComment {
    fn into(self) -> super::TitleComment {
        super::TitleComment {
            index: self.index,
            name: self.name,
            chapter_comments: self.chapter_comments,
        }
    }
}
impl From<super::Bdrom> for V1Bdrom {
    fn from(a: super::Bdrom) -> Self {
        Self {
            internal_id: a.internal_id,
            title_comments: a.title_comments.into_iter().map(|e| e.into()).collect(),
            general_comment: a.general_comment,
        }
    }
}
impl Into<super::Bdrom> for V1Bdrom {
    fn into(self) -> super::Bdrom {
        super::Bdrom {
            internal_id: self.internal_id,
            path: String::new(),
            title_comments: self.title_comments.into_iter().map(|e| e.into()).collect(),
            general_comment: self.general_comment,
        }
    }
}
impl From<super::AudioMode> for V1AudioMode {
    fn from(a: super::AudioMode) -> Self {
        match a {
            super::AudioMode::Auto => V1AudioMode::Auto,
            super::AudioMode::Single(e) => V1AudioMode::Single(e),
            super::AudioMode::Multi(e) => V1AudioMode::Multi(e),
        }
    }
}
impl Into<super::AudioMode> for V1AudioMode {
    fn into(self) -> super::AudioMode {
        match self {
            V1AudioMode::Auto => super::AudioMode::Auto,
            V1AudioMode::Single(e) => super::AudioMode::Single(e),
            V1AudioMode::Multi(e) => super::AudioMode::Multi(e),
        }
    }
}

impl From<super::TitleClipIndex> for V1TitleClipIndex {
    fn from(i: super::TitleClipIndex) -> Self {
        Self {
            title: i.title,
            clip: i.clip,
            audio_mode: i.audio_mode.into(),
        }
    }
}
impl Into<super::TitleClipIndex> for V1TitleClipIndex {
    fn into(self) -> super::TitleClipIndex {
        super::TitleClipIndex {
            title: self.title,
            clip: self.clip,
            audio_mode: self.audio_mode.into(),
        }
    }
}

impl From<super::PlaylistClipIndex> for V1PlaylistClipIndex {
    fn from(a: super::PlaylistClipIndex) -> Self {
        Self {
            playlist: a.playlist,
            clip: a.clip,
            audio_mode: a.audio_mode.into(),
        }
    }
}
impl Into<super::PlaylistClipIndex> for V1PlaylistClipIndex {
    fn into(self) -> super::PlaylistClipIndex {
        super::PlaylistClipIndex {
            playlist: self.playlist,
            clip: self.clip,
            audio_mode: self.audio_mode.into(),
        }
    }
}

impl From<super::BlurayExtract> for V1BlurayExtract {
    fn from(a: super::BlurayExtract) -> Self {
        match a {
            super::BlurayExtract::PlaylistFull(a) => V1BlurayExtract::PlaylistFull(a),
            super::BlurayExtract::PlaylistFromToChap(a, b, c) => {
                V1BlurayExtract::PlaylistFromToChap(a, b, c)
            }
            super::BlurayExtract::PlaylistClipIndex(a) => {
                V1BlurayExtract::PlaylistClipIndex(a.into())
            }
        }
    }
}
impl Into<super::BlurayExtract> for V1BlurayExtract {
    fn into(self) -> super::BlurayExtract {
        match self {
            V1BlurayExtract::PlaylistFull(a) => super::BlurayExtract::PlaylistFull(a),
            V1BlurayExtract::PlaylistFromToChap(a, b, c) => {
                super::BlurayExtract::PlaylistFromToChap(a, b, c)
            }
            V1BlurayExtract::PlaylistClipIndex(a) => {
                super::BlurayExtract::PlaylistClipIndex(a.into())
            }
        }
    }
}

impl From<super::SingularRemuxMatroskaFile> for V1SingularRemuxMatroskaFile {
    fn from(a: super::SingularRemuxMatroskaFile) -> Self {
        Self {
            name: a.name,
            src: a.src,
            extract: a.extract.into(),
        }
    }
}
impl Into<super::SingularRemuxMatroskaFile> for V1SingularRemuxMatroskaFile {
    fn into(self) -> super::SingularRemuxMatroskaFile {
        super::SingularRemuxMatroskaFile {
            name: self.name,
            src: self.src,
            extract: self.extract.into(),
        }
    }
}

impl From<super::ClipSplit> for V1ClipSplit {
    fn from(a: super::ClipSplit) -> Self {
        Self {
            name: a.name,
            src: a.src,
            playlist: a.playlist,
            format_start: a.format_start,
            format_minwidth: a.format_minwidth,
            max_cnt: a.max_cnt,
        }
    }
}
impl Into<super::ClipSplit> for V1ClipSplit {
    fn into(self) -> super::ClipSplit {
        super::ClipSplit {
            name: self.name,
            src: self.src,
            playlist: self.playlist,
            format_start: self.format_start,
            format_minwidth: self.format_minwidth,
            max_cnt: self.max_cnt,
        }
    }
}
impl From<super::RemuxFolderEntrie> for V1RemuxFolderEntrie {
    fn from(a: super::RemuxFolderEntrie) -> Self {
        match a {
            super::RemuxFolderEntrie::SingularFile(e) => Self::SingularFile(e.into()),
            super::RemuxFolderEntrie::MultipleFilePlaylistClipSplit(e) => {
                Self::MultipleFilePlaylistClipSplit(e.into())
            }
        }
    }
}
impl Into<super::RemuxFolderEntrie> for V1RemuxFolderEntrie {
    fn into(self) -> super::RemuxFolderEntrie {
        match self {
            V1RemuxFolderEntrie::SingularFile(e) => {
                super::RemuxFolderEntrie::SingularFile(e.into())
            }
            V1RemuxFolderEntrie::MultipleFilePlaylistClipSplit(e) => {
                super::RemuxFolderEntrie::MultipleFilePlaylistClipSplit(e.into())
            }
        }
    }
}

impl From<super::RemuxFolder> for V1RemuxFolder {
    fn from(a: super::RemuxFolder) -> Self {
        Self {
            name: a.name,
            file_prefix: a.file_prefix,
            entries: a.entries.into_iter().map(|e| e.into()).collect(),
        }
    }
}
impl Into<super::RemuxFolder> for V1RemuxFolder {
    fn into(self) -> super::RemuxFolder {
        super::RemuxFolder {
            name: self.name,
            entries: self.entries.into_iter().map(|e| e.into()).collect(),
            file_prefix: self.file_prefix,
            show: true,
            full_load: false,
        }
    }
}
