use egui::{FontData, FontDefinitions, FontFamily};

pub use eframe::egui;

mod gui_common;
use gui_common::*;
//use vmux_lib::{handling::{Config, BlurayExtract, NewBlurayExtract, PlaylistId, PlaylistClipIndex, TitleId}, bd_cache::BDsCache};
use vmux_lib::handling::Config;
mod egui_docking;
mod global_state;
mod gui_impl;
mod mpv_raw;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    if cfg!(windows) {
    } else {
        native_options.initial_window_size = Some(egui::Vec2::new(1280.0, 720.0));
        native_options.hardware_acceleration = eframe::HardwareAcceleration::Off;
    }

    //use vmux_lib::{handling::{Config, NewBlurayExtract, PlaylistId, PlaylistClipIndex, TitleId}, bd_cache::BDsCache};
    //let mut cfg = Config::dflt();
    //let mut bdbd = BDsCache::new();
    //let mut new_bdroms = cfg.blurays.clone();
    //for f in &mut new_bdroms {
    //    let bd = bdbd.get(f,&cfg.bd_index_dir).unwrap();
    //    let bd = bd.lock().unwrap();
    //    for t in &mut f.title_comments {
    //        t.index = PlaylistId::from_pis(bd.get_ti(TitleId::from_title_id(t.index.acual_title_pis())).unwrap().playlist as u64);
    //    }
    //}
    //cfg.blurays = new_bdroms;
    //cfg.save();
    //return;
    //    use std::{hash::Hash,hash::Hasher, collections::hash_map::DefaultHasher};
    //
    //    let cfg = Config::dflt();
    //    let mut exp = vmux_lib::config::Exporter::new();
    //    let mut blr = cfg.blurays.clone();
    //    let mut fld = cfg.folders.clone();
    //    for a in &mut blr {
    //        a.path = String::new();
    //    }
    //    for a in &mut fld {
    //        a.show = true;
    //        a.full_load = false;
    //    }
    //    let mut hasher = DefaultHasher::new();
    //    blr.hash(&mut hasher);
    //    fld.hash(&mut hasher);
    //    let hash_before = hasher.finish();
    //
    //
    //    exp.add_folders(&fld);
    //    exp.add_bdroms(&blr);
    //    let out = exp.string_out();
    //    println!("{}",out);
    //    println!("{}",out.len());
    //
    //    let reparse = vmux_lib::config::Exporter::from_string(&out).unwrap();
    //
    //    let mut hasher = DefaultHasher::new();
    //    reparse.blurays.hash(&mut hasher);
    //    reparse.folders.hash(&mut hasher);
    //    let hash_after = hasher.finish();
    //    println!("{}",hash_before);
    //    println!("{}",hash_after);
    //let cfg = Config::dflt();
    //let mut exp = vmux_lib::handling::Exporter::new();
    //    exp.add_folders(&cfg.folders);
    //exp.add_bdroms(&cfg.blurays);
    //let strng = exp.string_out_txt_uncompressed();
    //    println!("{}",strng);
    //return;

    eframe::run_native(
        "vmux",
        native_options,
        Box::new(|cc| {
            let mut fonts = FontDefinitions::default();

            // Install my own font (maybe supporting non-latin characters):
            fonts.font_data.insert(
                "my_font".to_owned(),
                FontData::from_static(include_bytes!("../../ipagp.ttf")),
            ); // .ttf and .otf supported

            // Put my font first (highest priority):
            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "my_font".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Box::new(EguiApp::new(Config::dflt()))
        }),
    );
}
