use crate::egui;
use std::sync::*;
use vmux_lib::{
    handling::{Config, InteralBDROMId},
    FFMS2IndexedFile,
};

pub struct IndexQueuEntry {
    pub bdrom: InteralBDROMId,
    pub streama: String,
    pub prog: Arc<Mutex<f32>>,
    pub started: bool,
    pub finished: Arc<Mutex<bool>>,
}

impl IndexQueuEntry {
    pub fn new(bdrom: &InteralBDROMId, clip: String) -> IndexQueuEntry {
        IndexQueuEntry {
            bdrom: bdrom.to_owned(),
            streama: clip,
            prog: Arc::new(Mutex::new(0.0)),
            started: false,
            finished: Arc::new(Mutex::new(false)),
        }
    }
}

#[derive(Default)]
pub struct GuiIndexQueue {
    pub index_queue: Vec<IndexQueuEntry>,
}

pub fn check_trigger_indexing(asd: &mut GuiIndexQueue, vmux_config: &Config) {
    for f in &mut asd.index_queue {
        if !f.started {
            let bd = vmux_config.bluray(&f.bdrom).unwrap();
            let clp_file = bd.find_stream_file(&f.streama);
            let idx_file = bd.index_for_stream(&f.streama, &vmux_config.bd_index_dir);

            let finished = f.finished.clone();
            let prog = f.prog.clone();
            std::thread::spawn(move || {
                FFMS2IndexedFile::new_ex(clp_file, idx_file, prog);
                *finished.lock().unwrap() = true;
            });
            f.started = true;
        }
    }
}

pub fn gui_index_queue(ui: &mut egui::Ui, asd: &mut GuiIndexQueue, vmux_config: &Config) {
    if ui.button("Remove finished").clicked() {
        asd.index_queue.retain(|e| !(*e.finished.lock().unwrap()));
    }
    check_trigger_indexing(asd, vmux_config);
    for f in &mut asd.index_queue {
        let prog = {
            let ll = f.prog.lock().unwrap();
            *ll
        };
        let finsh = {
            let ll = f.finished.lock().unwrap();
            *ll
        };
        ui.label(format!(
            "{} {} {} prog: {:0.2}",
            f.started,
            finsh,
            f.streama,
            prog * 100.0
        ));
    }
}
