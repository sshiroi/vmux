mod ids;
pub use ids::*;

mod bdrom;
pub use bdrom::*;

mod remux_folder;
pub use remux_folder::*;

use serde::*;
use std::path::PathBuf;

mod exporter;
pub use exporter::*;

mod v1;

pub type InteralBDROMId = String;

fn dflt_mpvraw_exportlocation() -> String {
    match home::home_dir() {
        Some(e) => PathBuf::from(e).join("Videos").display().to_string(),
        None => "".to_string(),
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Hash)]
pub struct Config {
    pub blurays: Vec<Bdrom>,
    pub folders: Vec<RemuxFolder>,
    pub bd_index_dir: String,

    pub ftp_port: u16,

    #[serde(default = "dflt_mpvraw_exportlocation")]
    pub mpvraw_exportlocation: String,

    #[serde(skip_serializing, skip_deserializing)]
    pathh: String,
}

impl Config {
    pub fn vmux_home_path() -> PathBuf {
        let home = home::home_dir().unwrap();
        PathBuf::from(home).join(".vmux/")
    }
    pub fn default_config_path() -> PathBuf {
        Self::vmux_home_path().join("config.json")
    }

    pub fn dflt() -> Config {
        Config::new(Config::default_config_path().display().to_string())
    }
    pub fn new(pthth: String) -> Config {
        if let Some(stra) = std::fs::read_to_string(&pthth).ok() {
            let mut vasd: Config = serde_json::from_str(&stra).unwrap();
            vasd.pathh = pthth;
            for bd in &mut vasd.blurays {
                bd.migrate();
            }

            for f in &mut vasd.folders {
                f.migrate();
            }

            vasd
        } else {
            if let Some(ee) = std::fs::read_link(&pthth).ok() {
                panic!(
                    "Not overriding config because its a symlink {}",
                    ee.display()
                );
            }

            let parent = PathBuf::from(&pthth).parent().unwrap().to_owned();
            let indexx = parent.join("ffms2_index");
            //Cursed
            let _ = std::fs::create_dir(&parent).ok();
            let _ = std::fs::create_dir(&indexx).ok();

            Config {
                blurays: Vec::new(),
                folders: Vec::new(),
                bd_index_dir: indexx.display().to_string(),
                pathh: pthth.clone(),
                ftp_port: 2121,
                mpvraw_exportlocation: dflt_mpvraw_exportlocation(),
            }
            .save();
            Config::new(pthth)
        }
    }

    pub fn save(&self) {
        let stra = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(&self.pathh, stra).unwrap();
    }

    //pub fn bluray(&self, interal_id: &InteralBDROMId) -> &Bdrom {
    pub fn bluray(&self, interal_id: &str) -> Option<&Bdrom> {
        let res: Option<&Bdrom> = self
            .blurays
            .iter()
            .find(|e| (*e).internal_id == *interal_id);
        res
    }
    pub fn exists_bd(&self, interal_id: &str) -> bool {
        self.bluray(interal_id).is_some()
    }
    pub fn exists_folder(&self, name: &str) -> bool {
        for f in &self.folders {
            if f.name == name {
                return true;
            }
        }
        false
    }

    pub fn new_bd(
        &mut self,
        internal_id: InteralBDROMId,
        path: &str,
    ) -> std::result::Result<(), simple_error::SimpleError> {
        if internal_id.len() == 0 {
            simple_error::bail!("id to short");
        }
        if !std::path::PathBuf::new().join(path).exists() {
            simple_error::bail!("path does not exists");
        }

        if self
            .blurays
            .iter()
            .find(|e| (*e).internal_id == *internal_id)
            .is_some()
        {
            simple_error::bail!("id already exists");
        }

        let bdrom = Bdrom {
            internal_id,
            path: path.to_owned(),
            ..Default::default()
        };

        self.blurays.push(bdrom);
        Ok(())
    }

    pub fn new_folder(
        &mut self,
        name: &str,
        show: bool,
    ) -> std::result::Result<(), simple_error::SimpleError> {
        for f in &self.folders {
            if f.name == name {
                simple_error::bail!("fodler already exists");
            }
        }
        if name.trim().len() == 0 {
            simple_error::bail!("invalid foldername");
        }
        if name.trim().len() != name.len() {
            simple_error::bail!("Foldername would couse problems on windows (trailing whitespace)");
        }
        self.folders.push(RemuxFolder {
            name: name.to_owned(),
            file_prefix: format!("{} ", name),
            show,
            ..Default::default()
        });
        Ok(())
    }
    pub fn bluray_mut<F: FnOnce(&mut Bdrom)>(&mut self, interal_id: &InteralBDROMId, fna: F) {
        let bdrom = self
            .blurays
            .iter_mut()
            .find(|e| (*e).internal_id == *interal_id)
            .unwrap();
        fna(bdrom);
    }
}

pub fn could_be_bdrom_at_path<A: AsRef<std::path::Path>>(a: A) -> bool {
    let mut aasd = a.as_ref().to_path_buf();
    aasd.push("BDMV/STREAM");
    aasd.exists()
}
