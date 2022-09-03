use super::ino_allocator::*;
use crate::handling::SingularRemuxMatroskaFile;
use crate::{matroska_backed::MatroskaBacked, y4m_wav_backed_file::*};

pub enum EmuFile {
    WavFile(AudioBackedFile),
    Y4MFile(VideoBackedFile),
    Matroska(MatroskaBacked),
    UnloadedMatroska(SingularRemuxMatroskaFile),
    TxtFile((String, bool)),
}

impl core::fmt::Debug for EmuFile {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("Dont debug");
    }
}

#[derive(Debug)]
pub struct HelloFSFile {
    pub name: String,
    pub ino: u64,
    pub size: u64,

    pub backed: EmuFile,
}

#[derive(Debug)]
pub struct HelloFSFolder {
    pub name: String,
    pub ino: u64,

    pub inner: Vec<HelloFsEntry>,
}

#[derive(Debug)]
pub enum HelloFsEntry {
    HelloFile(HelloFSFile),
    HelloFolder(HelloFSFolder),
}
impl HelloFsEntry {
    pub fn ino(&self) -> u64 {
        match self {
            HelloFsEntry::HelloFile(e) => e.ino,
            HelloFsEntry::HelloFolder(e) => e.ino,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            HelloFsEntry::HelloFile(e) => &e.name,
            HelloFsEntry::HelloFolder(e) => &e.name,
        }
    }
}

impl Into<HelloFsEntry> for HelloFSFile {
    fn into(self) -> HelloFsEntry {
        HelloFsEntry::HelloFile(self)
    }
}
impl Into<HelloFsEntry> for HelloFSFolder {
    fn into(self) -> HelloFsEntry {
        HelloFsEntry::HelloFolder(self)
    }
}

pub struct HelloFSFolderBuilder {
    pub files: Vec<HelloFsEntry>,
}

impl HelloFSFolderBuilder {
    pub fn new() -> HelloFSFolderBuilder {
        HelloFSFolderBuilder { files: Vec::new() }
    }

    pub fn audio(
        mut self,
        ino_src: &mut InoAllocator,
        name: &str,
        audio: AudioBackedFile,
    ) -> HelloFSFolderBuilder {
        self.files.push(
            HelloFSFile {
                name: name.to_string(),
                ino: ino_src.allocate(),
                size: audio.wavy.total_file_size as u64,
                backed: EmuFile::WavFile(audio),
            }
            .into(),
        );
        self
    }

    pub fn video(
        mut self,
        ino_src: &mut InoAllocator,
        name: &str,
        video: VideoBackedFile,
    ) -> HelloFSFolderBuilder {
        self.files.push(
            HelloFSFile {
                name: name.to_string(),
                ino: ino_src.allocate(),
                size: video.vy.y4m_total_file_size as u64,
                backed: EmuFile::Y4MFile(video),
            }
            .into(),
        );
        self
    }

    pub fn script(
        self,
        ino_src: &mut InoAllocator,
        name: &str,
        script: String,
    ) -> HelloFSFolderBuilder {
        self.file(ino_src, name, script, true)
    }

    pub fn file(
        mut self,
        ino_src: &mut InoAllocator,
        name: &str,
        script: String,
        is_sciprt: bool,
    ) -> HelloFSFolderBuilder {
        self.files.push(
            HelloFSFile {
                name: name.to_string(),
                ino: ino_src.allocate(),
                size: script.as_bytes().len() as u64,
                backed: EmuFile::TxtFile((script, is_sciprt)),
            }
            .into(),
        );
        self
    }

    pub fn matroska(
        mut self,
        ino_src: &mut InoAllocator,
        name: &str,
        matroksa: MatroskaBacked,
    ) -> HelloFSFolderBuilder {
        self.files.push(
            HelloFSFile {
                name: name.to_string(),
                ino: ino_src.allocate(),
                size: matroksa.total_size as u64,
                backed: EmuFile::Matroska(matroksa),
            }
            .into(),
        );
        self
    }

    pub fn unfinished_matroska(
        mut self,
        ino_src: &mut InoAllocator,
        name: &str,
        e: SingularRemuxMatroskaFile,
    ) -> HelloFSFolderBuilder {
        self.files.push(
            HelloFSFile {
                name: name.to_string(),
                ino: ino_src.allocate(),
                size: 999999999999 as u64,
                backed: EmuFile::UnloadedMatroska(e),
            }
            .into(),
        );
        self
    }

    pub fn folder(
        mut self,
        ino_src: &mut InoAllocator,
        name: &str,
        entries: HelloFSFolderBuilder,
    ) -> HelloFSFolderBuilder {
        //TODO: check for ino overlap
        self.files.push(HelloFsEntry::HelloFolder(HelloFSFolder {
            ino: ino_src.allocate(),
            name: name.to_string(),
            inner: entries.build(),
        }));

        self
    }
    pub fn build(self) -> Vec<HelloFsEntry> {
        self.files
    }
}
