use super::{Bdrom, RemuxFolder};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct V1Export {
    blurays: Vec<super::v1::V1Bdrom>,
    folders: Vec<super::v1::V1RemuxFolder>,
}

pub struct Exporter {
    out: V1Export,
}
pub struct ExporterOutput {
    pub blurays: Vec<Bdrom>,
    pub folders: Vec<RemuxFolder>,
}
impl Exporter {
    pub fn new() -> Exporter {
        Exporter {
            out: V1Export {
                blurays: Vec::new(),
                folders: Vec::new(),
            },
        }
    }

    pub fn add_bdroms(&mut self, bdrom: &[Bdrom]) {
        for a in bdrom {
            self.add_bdrom(a);
        }
    }
    pub fn add_folders(&mut self, fld: &[RemuxFolder]) {
        for a in fld {
            self.add_folder(a);
        }
    }
    pub fn add_bdrom(&mut self, bdrom: &Bdrom) {
        self.out.blurays.push(bdrom.clone().into());
    }
    pub fn add_folder(&mut self, fld: &RemuxFolder) {
        self.out.folders.push(fld.clone().into());
    }

    pub fn string_out(&self) -> String {
        let bts: Vec<u8> = rmp_serde::to_vec(&self.out).unwrap();
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&bts).unwrap();
        let compressed_bytes = e.finish().unwrap();
        let out = base64::encode(compressed_bytes);
        format!("v1_vmux_export_{}", out)
    }
    pub fn string_out_txt_uncompressed(&self) -> String {
        let bts: String = serde_yaml::to_string(&self.out).unwrap();
        bts
    }
    pub fn from_string(a: &str) -> Option<ExporterOutput> {
        let bts: serde_yaml::Result<V1Export> = serde_yaml::from_str(a);
        if let Ok(e) = bts {
            return Some(ExporterOutput {
                blurays: e.blurays.into_iter().map(|e| e.into()).collect(),
                folders: e.folders.into_iter().map(|e| e.into()).collect(),
            });
        }

        let strt: Option<usize> = a.find("v1_vmux_export_");
        if strt.is_none() {
            return None;
        }
        let trgt = &a[strt.unwrap()..];
        if trgt.starts_with("v1_vmux_export_") {
            let c: String = trgt.chars().skip(15).collect();

            let out = base64::decode(c);
            if !out.is_ok() {
                println!("Base64 not ok");
                return None;
            }
            let out = out.unwrap();
            let d = flate2::read::ZlibDecoder::new(std::io::Cursor::new(out));
            let bts: Result<V1Export, _> = rmp_serde::from_read(d);
            if !bts.is_ok() {
                println!("V1Export not ok");
                return None;
            }
            let bts = bts.unwrap();
            Some(ExporterOutput {
                blurays: bts.blurays.into_iter().map(|e| e.into()).collect(),
                folders: bts.folders.into_iter().map(|e| e.into()).collect(),
            })
        } else {
            None
        }
    }
}
