use crate::{
    bd_stream_av_cache::BDAVStreamCache,
    handling::{Bdrom, PlaylistId, TitleId},
};
use bluray_support::{TitleInfo, BD};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::*};

pub trait TitleInfoProvider {
    fn clone_titleinfo(&self) -> Vec<TitleInfo>;
    fn depre_pis(&self, pi: PlaylistId) -> &TitleInfo;
    fn tis_from_pis(&self, pis: PlaylistId) -> Option<TitleId>;
    fn get_titleinfo(&self, ti: TitleId) -> Option<&TitleInfo>;
    fn get_titleinfo_playlist(&self, pi: PlaylistId) -> Option<&TitleInfo>;
}

impl TitleInfoProvider for Vec<TitleInfo> {
    fn clone_titleinfo(&self) -> Vec<TitleInfo> {
        self.clone()
    }

    /*
    Deperected
    Same as get_titleinfo_playlist but with unwrap
    */
    fn depre_pis(&self, pi: PlaylistId) -> &TitleInfo {
        self.get_titleinfo_playlist(pi).unwrap()
    }

    fn tis_from_pis(&self, pis: PlaylistId) -> Option<TitleId> {
        for f in self {
            if pis.acual_title_pis() == f.playlist as u64 {
                return Some(TitleId::from_title_id(f.idx as u64));
            }
        }
        None
    }

    fn get_titleinfo(&self, ti: TitleId) -> Option<&TitleInfo> {
        if (ti.acual_title_id() as usize) < self.len() {
            Some(&self[ti.acual_title_id() as usize])
        } else {
            None
        }
    }

    fn get_titleinfo_playlist(&self, pi: PlaylistId) -> Option<&TitleInfo> {
        if let Some(e) = self.tis_from_pis(pi) {
            self.get_titleinfo(e)
        } else {
            None
        }
    }
}

pub struct CachedBD {
    //bd: BD,
    tis: Vec<TitleInfo>,
    pub bdav_helper: BDAVStreamCache,
}

impl CachedBD {
    pub fn new(bdrom: &Bdrom, bd_index_dir: &str, tis: Vec<TitleInfo>) -> CachedBD {
        CachedBD {
            //bd,
            bdav_helper: BDAVStreamCache::new(bdrom, bd_index_dir),
            tis,
        }
    }
}

impl TitleInfoProvider for CachedBD {
    fn clone_titleinfo(&self) -> Vec<TitleInfo> {
        self.tis.clone_titleinfo()
    }

    fn depre_pis(&self, pi: PlaylistId) -> &TitleInfo {
        self.tis.depre_pis(pi)
    }

    fn tis_from_pis(&self, pis: PlaylistId) -> Option<TitleId> {
        self.tis.tis_from_pis(pis)
    }

    fn get_titleinfo(&self, ti: TitleId) -> Option<&TitleInfo> {
        self.tis.get_titleinfo(ti)
    }

    fn get_titleinfo_playlist(&self, pi: PlaylistId) -> Option<&TitleInfo> {
        self.tis.get_titleinfo_playlist(pi)
    }
}

#[derive(Default, Serialize, Deserialize)]
struct BDTisDiskCache {
    map: std::collections::HashMap<String, Vec<TitleInfo>>,
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

pub struct BDsCache {
    //         bdrom ,   CachedBD
    cache: BDTisDiskCache,
    veca: Vec<(String, Arc<Mutex<CachedBD>>)>,
}

impl BDsCache {
    pub fn new() -> BDsCache {
        BDsCache {
            cache: BDTisDiskCache::new(),
            veca: Vec::new(),
        }
    }

    pub fn save(&self) {
        self.cache.save()
    }
    pub fn clear_disk(&mut self) {
        self.cache.clear();
        self.cache.save();
    }
    pub fn clear_for(&mut self, asd: &str) {
        if self.cache.map.contains_key(asd) {
            self.cache.map.remove(asd);
        }
    }
    pub fn get_tis(&mut self, bdrom: &Bdrom) -> Option<Vec<TitleInfo>> {
        if let Some(e) = self.cache.find(&bdrom.internal_id) {
            Some(e.clone())
        } else {
            let bdd = BD::open(&bdrom.path);
            if let Some(e) = bdd {
                let tis = Self::get_tis_from_bd(&e);
                self.cache.put(&bdrom.internal_id, tis);
                self.get_tis(bdrom)
            } else {
                None
            }
        }
    }

    pub fn get_full(&mut self, bdrom: &Bdrom, index_path: &str) -> Option<Arc<Mutex<CachedBD>>> {
        for f in &self.veca {
            if &bdrom.path == &f.0 {
                return Some(f.1.clone());
            }
        }

        //Pushing code
        {
            let tis = self.get_tis(bdrom);
            if tis.is_none() {
                return None;
            }
            let tis = tis.unwrap();
            self.veca.push((
                bdrom.path.to_owned(),
                Arc::new(Mutex::new(CachedBD::new(bdrom, index_path, tis))),
            ));
        }
        self.get_full(bdrom, index_path)
    }

    fn get_tis_from_bd(bd: &BD) -> Vec<TitleInfo> {
        bd.get_titles();
        let mut tis = Vec::new();
        for f in 0..bd.get_titles() {
            let t = bd.get_title_info(f, 0).unwrap();
            tis.push(t);
        }
        tis
    }
}
