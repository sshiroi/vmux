use crate::egui;
use crate::egui_docking;
use bluray_support::TitleInfo;
use egui::{Color32, Frame, Widget};
use vmux_lib::{bd_cache::BDsCache, handling::*};

pub struct BDDisplayInfo {
    // bd: bluray_support::BD,
    pub legacy_title_list: Vec<TitleInfo>,

    pub path: String,

    //                  s   path     indexed
    pub strms: Vec<(String, PathBuf, bool)>,
}

impl BDDisplayInfo {
    pub fn new(
        path: &str,
        bdrom: &Bdrom,
        bdbd: &mut BDsCache,
        bd_index_dir: &str,
    ) -> Option<BDDisplayInfo> {
        //let bd = bluray_support::BD::open(path);
        let bd = bdbd.get_tis(bdrom);
        let tis = match bd {
            Some(e) => e,
            None => return None,
        };

        let mut strms = Vec::new();
        for (s, _) in bdrom.find_streams() {
            let indexed = bdrom.index_for_stream(&s, bd_index_dir).exists();
            let strm_file = bdrom.find_stream_file(&s);

            strms.push((s, strm_file, indexed));
        }

        Some(BDDisplayInfo {
            //bd,
            legacy_title_list: tis,
            path: path.to_string(),
            strms,
        })
    }
}

use std::path::PathBuf;

use crate::gui_impl;

use std::sync::*;

#[derive(Debug, PartialEq)]
pub enum BdInspectSortMode {
    Title,
    Playlist,
}

pub struct GuiGlobalState {
    pub ondisk_vmux_config: Config,

    pub vmux_config: Config,
    pub inspect_bd: Option<(InteralBDROMId, BDDisplayInfo)>,
    pub highlighted_bd: Option<(InteralBDROMId, bool)>,
    pub selected_folder: Option<usize>,

    pub bdmvs_filter: String,
    pub folders_filter: String,

    pub tmp_add_path: String,
    pub tmp_internal_id: String,
    pub tmp_bdmvs_refacor_rename_src: String,

    pub tmp_add_is_bd_addable1: bool,
    pub tmp_add_is_bd_addable2: bool,

    pub tmp_add_folder_path: String,

    pub tmp_folder_entrie: RemuxFolderEntrie,

    pub panel_state: u32,

    pub can_exit: bool,
    pub is_exiting: bool,

    pub vlc_output: Arc<Mutex<Vec<String>>>,
    pub bd_inspect_sortmode: BdInspectSortMode,

    pub current_msg_errors: Vec<(usize, String)>,
    pub current_throw_err_cnt: usize,

    pub bdsc: BDsCache,
    pub ftp: Option<(u16, Arc<Mutex<bool>>, Vec<String>)>,
    pub ftp_search: String,

    pub mpv_raw: Option<Vec<RemuxFolder>>,
    pub mpv_raw_search: String,

    pub folders_selection_idx: Option<(usize, bool)>,

    pub remux_folder_flattend: Option<RemuxFolder>,
    pub remux_folder_lasthash: u64,

    pub edl_fix_offset: f64,

    pub remux_folder_last_hash_all: u64,
    pub remux_folder_errors: Vec<(usize, String)>,
    pub bdmvs_longpath: bool,
    pub inspect_longpath: bool,

    pub folder_export: Option<RemuxFolder>,
    pub folder_export_other: bool,
    pub folder_export_string: Option<String>,
    pub folder_export_yaml: bool,

    pub import_textbox: String,
    pub import_error: Option<String>,
    pub import_result: Option<(Vec<(String, bool, Bdrom)>, Vec<RemuxFolder>)>,
}

impl GuiGlobalState {
    pub fn throw_error(&mut self, asd: String) {
        self.current_msg_errors
            .push((self.current_throw_err_cnt, asd));
        self.current_throw_err_cnt += 1;
    }
    fn new(cfga: Config) -> GuiGlobalState {
        GuiGlobalState {
            vmux_config: cfga.clone(),
            ondisk_vmux_config: cfga.clone(),
            inspect_bd: None,

            tmp_add_path: Default::default(),
            tmp_internal_id: Default::default(),
            tmp_add_folder_path: Default::default(),
            tmp_add_is_bd_addable1: false,
            tmp_add_is_bd_addable2: false,

            tmp_folder_entrie: RemuxFolderEntrie::SingularFile(SingularRemuxMatroskaFile::default()),
            selected_folder: None,
            highlighted_bd: None,
            panel_state: 0,

            can_exit: false,
            is_exiting: false,
            vlc_output: Arc::new(Mutex::new(Vec::new())),

            bd_inspect_sortmode: BdInspectSortMode::Playlist,
            current_msg_errors: Vec::new(),
            current_throw_err_cnt: 0,

            bdmvs_filter: String::new(),
            folders_filter: String::new(),
            bdsc: BDsCache::new(),
            ftp: None,
            ftp_search: String::new(),

            mpv_raw: None,
            mpv_raw_search: String::new(),

            folders_selection_idx: None,
            remux_folder_flattend: None,
            remux_folder_lasthash: 0,

            tmp_bdmvs_refacor_rename_src: String::new(),

            edl_fix_offset: -0.034,

            remux_folder_last_hash_all: 0,
            remux_folder_errors: Vec::new(),

            bdmvs_longpath: false,
            inspect_longpath: false,

            folder_export: None,
            folder_export_string: None,

            import_result: None,
            import_textbox: String::new(),
            import_error: None,
            folder_export_other: false,
            folder_export_yaml: false,
        }
    }
}

pub struct EguiAppState {
    pub global: GuiGlobalState,
    pub index_queue: gui_impl::GuiIndexQueue,
    // pub errors: gui_impl::GuiErrorsState,
}

pub struct EguiApp {
    pub state: EguiAppState,
    pub egtre: egui_docking::Tree<EguiAppState>,
}

impl EguiApp {
    pub fn new(cfg: Config) -> Self {
        use egui_docking::NodeIndex;
        let _ = PlaceholderTab::MpvRaw;
        //let _ = PlaceholderTab::Errors;
        let mut egtre = egui_docking::Tree::new(vec![
            Box::new(PlaceholderTab::Inspect),
            //          Box::new(PlaceholderTab::Errors),
            //        Box::new(PlaceholderTab::Index),
            Box::new(PlaceholderTab::Ftp),
            //    Box::new(PlaceholderTab::MpvRaw),
            Box::new(PlaceholderTab::Clips),
            Box::new(PlaceholderTab::Import),
            Box::new(PlaceholderTab::Config),
        ]);
        let [a, b] = egtre.split_left(
            NodeIndex::root(),
            0.16,
            vec![Box::new(PlaceholderTab::Bdvms)],
        );
        egtre.split_below(b, 0.5, vec![Box::new(PlaceholderTab::Folders)]);
        let [c, _] = egtre.split_right(a, 0.5, vec![Box::new(PlaceholderTab::RemuxFolders)]);
        egtre.split_below(c, 0.5, vec![Box::new(PlaceholderTab::Vlc)]);
        Self {
            state: EguiAppState::new(cfg),
            egtre,
        }
    }
}

impl EguiAppState {
    fn new(cfg: Config) -> Self {
        Self {
            global: GuiGlobalState::new(cfg),
            index_queue: Default::default(),
            //      errors: Default::default(),
        }
    }
}

impl eframe::App for EguiApp {
    fn on_exit_event(&mut self) -> bool {
        let glob = &mut self.state.global;
        if glob.vmux_config != glob.ondisk_vmux_config {
            glob.is_exiting = true;
            glob.can_exit
        } else {
            true
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        {
            let glob = &mut self.state.global;
            // crate::gui_impl::gui_bdmvs_window(ctx, glob);
            // crate::gui_impl::gui_bd_folders_window(ctx, glob);

            // crate::gui_impl::gui_bd_inspection_window(ctx, glob);
            // crate::gui_impl::gui_remux_folder_window(ctx, glob);
            // crate::gui_impl::gui_config_window(ctx, glob);

            if glob.is_exiting {
                egui::Window::new("quitw?")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.heading("Do you want to quit? There are changes that aren't saved.");
                        ui.horizontal(|ui| {
                            let close = egui::Button::new("Close").fill(Color32::RED).ui(ui);
                            let cancel = egui::Button::new("Cancel").ui(ui);
                            let save = egui::Button::new("Save").fill(Color32::GREEN).ui(ui);
                            if close.clicked() {
                                glob.can_exit = true;
                                frame.quit();
                            }
                            if cancel.clicked() {
                                glob.is_exiting = false;
                            }

                            if save.clicked() {
                                //Config
                                glob.vmux_config.save();
                                //Cache
                                glob.bdsc.save();

                                glob.can_exit = true;
                                frame.quit();
                            }
                        });
                    });
            }

            let mut cntrr = 0;
            glob.current_msg_errors.retain(|e| {
                cntrr += 1;
                let mut asd = true;
                egui::Window::new(format!("Error? {}", e.0))
                    .collapsible(false)
                    .resizable(false)
                    .frame(Frame::window(&ctx.style()).fill(Color32::LIGHT_RED))
                    .show(ctx, |ui| {
                        ui.heading(&e.1);
                        if ui.button("Close").clicked() {
                            asd = false;
                        }
                    });
                asd
            });

            if glob.folder_export.is_some()
                || (glob.folder_export_other && glob.folder_export_string.is_some())
            {
                egui::Window::new(format!("Export?"))
                    .collapsible(false)
                    .resizable(false)
                    // .frame(Frame::window(&ctx.style()).fill(Color32::LIGHT_RED))
                    .show(ctx, |ui| {
                        if !glob.folder_export_other {
                            let name = glob.folder_export.as_ref().unwrap().name.clone();

                            ui.heading(format!("Export - {}", name));
                        } else {
                            ui.heading(format!("Export - Other"));
                        }

                        //ui.checkbox(&mut glob.folder_export_yaml, "yaml");
                        glob.folder_export_yaml = false;

                        if ui.button("Close").clicked() {
                            glob.folder_export = None;
                            glob.folder_export_string = None;
                            glob.folder_export_other = false;
                            return;
                        }

                        if let Some(e) = &glob.folder_export_string {
                            ui.text_edit_multiline(&mut e.clone());
                        } else {
                            let name = glob.folder_export.as_ref().unwrap().name.clone();

                            let e = glob.folder_export.as_ref().unwrap();

                            ui.horizontal(|ui| {
                                let mut exprt_some = None;
                                if ui.button("With Bdroms").clicked() {
                                    let mut exp = vmux_lib::config::Exporter::new();
                                    exp.add_folder(e);
                                    let strc = collect_bdrom_src(&e.entries);

                                    for e in &strc {
                                        if let Some(bdr) = glob.vmux_config.bluray(e) {
                                            exp.add_bdrom(bdr);
                                        }
                                    }
                                    exprt_some = Some((format!("[{}_bdroms]", name), exp));
                                }
                                if ui.button("Without Bdroms").clicked() {
                                    let mut exp = vmux_lib::config::Exporter::new();
                                    exp.add_folder(e);
                                    exprt_some = Some((format!("[{}_nobdroms]", name), exp));
                                }
                                if ui.button("Bdroms only").clicked() {
                                    let mut exp = vmux_lib::config::Exporter::new();
                                    let strc = collect_bdrom_src(&e.entries);

                                    for e in &strc {
                                        if let Some(bdr) = glob.vmux_config.bluray(e) {
                                            exp.add_bdrom(bdr);
                                        }
                                    }
                                    exprt_some = Some((format!("[{}_only_bdroms]", name), exp));
                                }
                                if let Some(e) = exprt_some {
                                    if !glob.folder_export_yaml {
                                        glob.folder_export_string =
                                            Some(format!("{}{}", e.0, e.1.string_out()));
                                    } else {
                                        glob.folder_export_string =
                                            Some(e.1.string_out_txt_uncompressed());
                                    }
                                }
                            });
                        }
                    });
            }
        }

        let style = egui_docking::Style::from_egui(ctx.style().as_ref());
        // let id = egui::Id::new("some hashable string");
        // egui::Window::new("test22").show(ctx, |ui| {
        //     egui_docking::show(ui, id, &style, &mut self.egtre, &mut self.state);
        // });

        let id = egui::Id::new("some hashable string");
        let layer_id = egui::LayerId::background();
        let max_rect = ctx.available_rect();
        let clip_rect = ctx.available_rect();

        let mut ui = egui::Ui::new(ctx.clone(), layer_id, id, max_rect, clip_rect);
        egui_docking::show(&mut ui, id, &style, &mut self.egtre, &mut self.state);

        ctx.request_repaint();
    }
}

fn collect_bdrom_src(es: &[RemuxFolderEntrie]) -> Vec<String> {
    let mut strc: Vec<String> = Vec::new();
    for e in es {
        let aa = e.src().to_owned();
        if !strc.contains(&aa) {
            strc.push(aa);
        }
    }
    strc
}

pub enum PlaceholderTab {
    // Errors,
    Ftp,
    MpvRaw,
    //   Index,
    Clips,
    Bdvms,
    Folders,
    RemuxFolders,
    Config,
    Vlc,
    Inspect,
    Import,
}

impl egui_docking::Tab<EguiAppState> for PlaceholderTab {
    fn title(&self) -> &str {
        match self {
            //PlaceholderTab::Errors => "Errors",
            //        PlaceholderTab::Index => "Index",
            PlaceholderTab::MpvRaw => "MpvRaw",
            PlaceholderTab::Ftp => "Ftp",
            PlaceholderTab::Clips => "BdStreams", //TODO: also change clips everywhere to BdStreams
            PlaceholderTab::Bdvms => "Bdmvs",
            PlaceholderTab::Folders => "Folders",
            PlaceholderTab::RemuxFolders => "RemuxFolders",
            PlaceholderTab::Config => "Config",
            PlaceholderTab::Inspect => "Inspect",
            PlaceholderTab::Vlc => "Vlc",
            PlaceholderTab::Import => "Import",
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EguiAppState) {
        let margin = egui::style::Margin::same(4.0);

        egui::Frame::none()
            .inner_margin(margin)
            .show(ui, |ui| match self {
                //    PlaceholderTab::Errors => {
                //        crate::gui_impl::gui_check_errors(ui, &mut ctx.errors, &ctx.global.vmux_config)
                //    }

                //   PlaceholderTab::Index => {},
                PlaceholderTab::Clips => {
                    crate::gui_impl::gui_clips(ui, &mut ctx.global, &mut ctx.index_queue)
                }

                PlaceholderTab::Bdvms => crate::gui_impl::gui_bdmvs(ui, &mut ctx.global),

                PlaceholderTab::Ftp => crate::gui_impl::gui_ftp(ui, &mut ctx.global),

                PlaceholderTab::MpvRaw => crate::gui_impl::gui_mpv_raw(ui, &mut ctx.global),

                PlaceholderTab::Folders => crate::gui_impl::gui_bd_folders(ui, &mut ctx.global),

                PlaceholderTab::RemuxFolders => {
                    crate::gui_impl::gui_remux_folder(ui, &mut ctx.global)
                }

                PlaceholderTab::Config => crate::gui_impl::gui_config(ui, &mut ctx.global),
                PlaceholderTab::Inspect => crate::gui_impl::gui_bd_inspection(ui, &mut ctx.global),
                PlaceholderTab::Import => crate::gui_impl::gui_import(ui, &mut ctx.global),
                PlaceholderTab::Vlc => {
                    ui.heading("Vlc output");
                    let mut ls = ctx.global.vlc_output.lock().unwrap();

                    if ui.button("Clear").clicked() {
                        ls.clear();
                    }
                    for ln in ls.iter().rev() {
                        ui.label(ln);
                    }
                }
            });
    }
}
