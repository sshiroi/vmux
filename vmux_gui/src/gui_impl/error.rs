/*
use crate::egui;
use vmux_lib::{
    bd_cache::BDsCache,
    handling::{Config, RemuxFolderEntrie, TitleId},
};

pub enum FoundError {
    BdmvNotInFolder(String, String),

    //folder name, src
    RemuxFolderBdmvDoesNotExist(String, String),
    //folder name, bdmv, title
    RemuxFolderBdmvTitleDoesNotExist(String, String, TitleId),
}

pub struct GuiErrorsState {
    pub errors: Vec<FoundError>,
}

impl Default for GuiErrorsState {
    fn default() -> Self {
        Self {
            errors: Default::default(),
        }
    }
}

impl GuiErrorsState {
    fn check_bdmvs_paths(&mut self, cfga: &Config) {
        for a in &cfga.blurays {
            let ok = Self::check_seems_like_bdrom(&a.path);
            if !ok {
                self.errors.push(FoundError::BdmvNotInFolder(
                    a.internal_id.to_owned(),
                    a.path.to_owned(),
                ));
            }
        }
    }

    fn check_all_other(&mut self, bda: &mut BDsCache, cfga: &Config) {
        //for f in &cfga.folders {
        //    for j in &f.entries {
        //        match j {
        //            RemuxFolderEntrie::SingularFile(_) => {
        //                //TODO: implement
        //            }
        //            RemuxFolderEntrie::MultipleFileTitleClipSplit(e) => {
        //                let bdbd = cfga.bluray(j.src());
        //                if let Some(bdbd) = bdbd {
        //                    let bdd = bda.get(bdbd, &cfga.bd_index_dir);
        //                    if bdd.is_none() {
        //                        //TODO: push error
        //                        continue;
        //                    }
        //                    let bdd = bdd.unwrap();
        //                    let bdd = bdd.lock().unwrap();
        //
        //                    if e.title as u32 >= bdd.bd.get_titles() {
        //                        self.errors
        //                            .push(FoundError::RemuxFolderBdmvTitleDoesNotExist(
        //                                e.name.to_owned(),
        //                                e.src.to_owned(),
        //                                e.title,
        //                            ));
        //                    }
        //                } else {
        //                    self.errors.push(FoundError::RemuxFolderBdmvDoesNotExist(
        //                        e.name.to_owned(),
        //                        e.src.to_owned(),
        //                    ));
        //                }
        //            }
        //        }
        //    }
        //}
    }

    pub fn check_all(&mut self, cfga: &Config) {
        let mut bda = BDsCache::new();
        let bda = &mut bda;

        self.check_bdmvs_paths(cfga);
        self.check_all_other(bda, cfga);
    }

    fn check_seems_like_bdrom<A: AsRef<std::path::Path>>(path: A) -> bool {
        if !path.as_ref().join("BDMV").exists() {
            return false;
        }
        if !path.as_ref().join("BDMV").join("index.bdmv").exists() {
            return false;
        }

        true
    }
}

pub fn gui_check_errors(ui: &mut egui::Ui, asd: &mut GuiErrorsState, vmux_config: &Config) {
    ui.label("Not really usefull does not chcek alot yet TODO: merge with RemuxFolders errors");
    if ui.button("CheckAll").clicked() {
        asd.errors.clear();
        asd.check_all(&vmux_config);
    }

    ui.separator();
    for (_, e) in asd.errors.iter().enumerate() {
        let sta = match e {
            FoundError::BdmvNotInFolder(e, f) => format!("At {} is no BDMV for {}", f, e),
            FoundError::RemuxFolderBdmvDoesNotExist(a, b) => {
                format!("For folder {} {} src does not exists", a, b)
            }
            FoundError::RemuxFolderBdmvTitleDoesNotExist(a, b, c) => {
                format!(
                    "For folder {} {} src does title {} not existzs",
                    a,
                    b,
                    c.acual_title_id()
                )
            }
        };
        ui.label(sta);
    }
}
*/
