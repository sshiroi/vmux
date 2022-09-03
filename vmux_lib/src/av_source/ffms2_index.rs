use ffms2::{index::*, IndexErrorHandling};
use std::path::*;
use std::sync::{Arc, Mutex};

pub struct FFMS2IndexedFile {
    pub file_path: PathBuf,
    pub index: Index,
    pub indexer: Indexer,
}
impl FFMS2IndexedFile {
    pub fn new<A: AsRef<Path>, B: AsRef<Path>>(file_path: A, index_path: B) -> FFMS2IndexedFile {
        Self::new_ex(file_path, index_path, Arc::new(Mutex::new(0.0)))
    }
    pub fn new_ex<A: AsRef<Path>, B: AsRef<Path>>(
        file_path: A,
        index_path: B,
        prog: std::sync::Arc<std::sync::Mutex<f32>>,
    ) -> FFMS2IndexedFile {
        let indexer = Indexer::new(file_path.as_ref()).unwrap();

        let idx_file = std::fs::read(index_path.as_ref());
        let rslt = if let Ok(e) = idx_file {
            //println!("Index exists reading.. {}", e.len());
            let rslt = ffms2::index::Index::ReadIndexFromBuffer(&e);
            rslt.ok()
        } else {
            None
        };
        let ee = if let Some(i) = rslt {
            i
        } else {
            //Index everything
            println!("Num traacaks tracks {}", indexer.NumTracksI());
            for i in 0..indexer.NumTracksI() {
                indexer.TrackIndexSettings(i, 1);
            }
            let mut val = 0;
            let prog2 = prog.clone();
            indexer.ProgressCallback(
                move |current, total, _| {
                    let mut asd = prog2.lock().unwrap();
                    *asd = current as f32 / total as f32;
                    0
                },
                &mut val,
            );
            println!("Creating index..");

            std::fs::create_dir_all(index_path.as_ref().parent().unwrap()).unwrap();

            let rslt = indexer.DoIndexing2(IndexErrorHandling::IEH_IGNORE).unwrap();
            rslt.WriteIndex(index_path.as_ref()).unwrap();
            println!("wainting for index wrote");

            loop {
                if index_path.as_ref().exists() {
                    break;
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
            println!("Wrote index");
            return FFMS2IndexedFile::new_ex(file_path, index_path, prog.clone());
            //let idx_file = std::fs::read(index_path.as_ref());
            //let e = idx_file.unwrap();
            //let rslt = ffms2::index::Index::ReadIndexFromBuffer(&e);
            //rslt.unwrap()
        };
        FFMS2IndexedFile {
            file_path: file_path.as_ref().to_owned(),
            index: ee,
            indexer,
        }
    }
}
