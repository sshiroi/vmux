use std::path::PathBuf;

use super::ids::*;
use super::InteralBDROMId;
use serde::*;

#[derive(Serialize, PartialEq, Deserialize, Hash, Clone, Debug)]
pub struct TitleComment {
    pub index: PlaylistId,

    pub name: String,
    pub chapter_comments: Vec<(u64, String)>,
}

impl TitleComment {
    //Title comments
    pub fn get_chapter_comment(&self, chapt: u64) -> Option<String> {
        self.chapter_comments
            .iter()
            .find(|e| e.0 == chapt)
            .map(|e| e.1.clone())
    }

    pub fn set_chapter_comment(&mut self, chapt: u64, stra: &str) {
        if self.get_chapter_comment(chapt).is_none() {
            self.chapter_comments.push((chapt, stra.to_owned()));
        } else {
            self.chapter_comments
                .iter_mut()
                .find(|e| e.0 == chapt)
                .unwrap()
                .1 = stra.to_owned();
        }
    }
}

#[derive(Serialize, PartialEq, Deserialize, Hash, Clone, Debug)]
pub struct Bdrom {
    pub internal_id: InteralBDROMId,
    pub path: String,
    pub title_comments: Vec<TitleComment>,
    pub general_comment: String,
}
impl Default for Bdrom {
    fn default() -> Self {
        Self {
            internal_id: Default::default(),
            path: Default::default(),
            title_comments: Default::default(),
            general_comment: Default::default(),
        }
    }
}

impl Bdrom {
    pub fn migrate(&mut self) {}

    pub fn stream_folder(&self) -> PathBuf {
        let streams = PathBuf::from(&self.path).join("BDMV/STREAM");
        streams
    }

    pub fn find_stream_file(&self, cipp: &str) -> PathBuf {
        let bdcliop = PathBuf::from(&self.path)
            .join("BDMV/STREAM")
            .join(format!("{}{}", cipp, ".m2ts"));
        if !bdcliop.exists() {
            //  let mteta = std::fs::metadata(&bdcliop);
            //  dbg!(mteta);
            panic!("clip ##{}## does not exist", bdcliop.display());
        }

        bdcliop
    }

    pub fn index_for_stream(&self, cipp: &str, bd_index_dir: &str) -> PathBuf {
        let bdidx = PathBuf::from(bd_index_dir)
            .join(format!("{}", self.internal_id))
            .join(format!("{}{}", cipp, ".idx"));
        bdidx
    }

    pub fn find_streams(&self) -> Vec<(String, PathBuf)> {
        let streams = self.stream_folder();

        let rslt = std::fs::read_dir(streams).unwrap();

        let mut ret = Vec::new();
        for f in rslt {
            let f = f.unwrap();
            let p = f.path();

            if p.extension().unwrap() == "m2ts" {
                let lflf = p.file_stem().unwrap();

                ret.push((lflf.to_str().unwrap().to_string(), p.clone()));
            }
        }
        ret
    }

    pub fn getcreate_title_comment(&self, title: PlaylistId) -> TitleComment {
        let res = self
            .title_comments
            .iter()
            .find(|e| e.index == title)
            .map(|e| e.clone());

        if let Some(ee) = res {
            ee
        } else {
            TitleComment {
                index: title,
                name: "".to_string(),
                chapter_comments: Vec::new(),
            }
        }
    }

    pub fn set_title_comment(&mut self, title: PlaylistId, tcmnt: TitleComment) {
        let res = self
            .title_comments
            .iter()
            .find(|e| e.index == title)
            .map(|e| e.clone());

        if let Some(_) = res {
            *self
                .title_comments
                .iter_mut()
                .find(|e| e.index == title)
                .unwrap() = tcmnt;
        } else {
            self.title_comments.push(tcmnt);
        }
    }
}
