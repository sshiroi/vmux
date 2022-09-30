use bluray_support::TitleInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct BDTisDiskCache {
    pub map: std::collections::HashMap<String, Vec<TitleInfo>>,
}

impl BDTisDiskCache {
    fn target_path() -> PathBuf {
        crate::config::Config::vmux_home_path().join("bd_tis_cache.bin")
    }

    pub fn new() -> BDTisDiskCache {
        if let Some(stra) = std::fs::File::open(Self::target_path()).ok() {
            let slf: std::result::Result<BDTisDiskCache, _> = rmp_serde::from_read(&stra);
            match slf {
                Ok(e) => e,
                Err(_) => Default::default(),
            }
        } else {
            Default::default()
        }
    }

    pub fn save(&self) {
        let buf = rmp_serde::to_vec(self).unwrap();
        let _ = std::fs::write(Self::target_path(), buf);
    }

    pub fn find(&self, k: &str) -> Option<&Vec<TitleInfo>> {
        self.map.get(k)
    }

    pub fn put(&mut self, k: &str, a: Vec<TitleInfo>) {
        self.map.insert(k.to_string(), a);
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}
