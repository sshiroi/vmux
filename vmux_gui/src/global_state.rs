use crate::gui_impl;
use crate::gui_impl::VlcOutput;
use std::sync::*;
use vmux_lib::{bd_cache::RGBDsCache, handling::*};

pub struct GuiGlobalState {
    pub ondisk_vmux_config: Config,

    pub vmux_config: Config,
    pub inspect_bd: Option<(InteralBDROMId, gui_impl::BDDisplayInfo)>,
    pub highlighted_bd: Option<(InteralBDROMId, bool)>,
    pub selected_folder: Option<usize>,

    pub bdmvs_filter: String,
    pub folders_filter: String,

    pub tmp_add_path: String,
    pub tmp_internal_id: String,
    pub tmp_bdmvs_refacor_rename_src: String,

    pub bdmvs_bd_addable: bool,

    pub bdmvs_addmanager: Option<Vec<(String, String, bool)>>,
    pub bdmvs_addmanager_tmp_srch: String,

    pub tmp_add_folder_path: String,

    pub tmp_folder_entrie: RemuxFolderEntrie,

    pub panel_state: u32,

    pub can_exit: bool,
    pub is_exiting: bool,

    pub vlc_filter_already: bool,
    pub vlc_output: Arc<Mutex<Option<VlcOutput>>>,
    pub bd_inspect_sortmode: gui_impl::BdInspectSortMode,

    pub current_msg_errors: Vec<(usize, String)>,
    pub current_throw_err_cnt: usize,

    pub bdsc: RGBDsCache,
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
    pub fn new(cfga: Config) -> GuiGlobalState {
        GuiGlobalState {
            vmux_config: cfga.clone(),
            ondisk_vmux_config: cfga.clone(),
            inspect_bd: None,

            tmp_add_path: Default::default(),
            tmp_internal_id: Default::default(),
            tmp_add_folder_path: Default::default(),

            tmp_folder_entrie: RemuxFolderEntrie::SingularFile(SingularRemuxMatroskaFile::default()),
            selected_folder: None,
            highlighted_bd: None,
            panel_state: 0,

            can_exit: false,
            is_exiting: false,
            vlc_output: Arc::new(Mutex::new(None)),
            vlc_filter_already: true,

            bd_inspect_sortmode: gui_impl::BdInspectSortMode::Playlist,
            current_msg_errors: Vec::new(),
            current_throw_err_cnt: 0,

            bdmvs_filter: String::new(),
            bdmvs_bd_addable: false,
            folders_filter: String::new(),
            bdsc: RGBDsCache::new(),
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

            bdmvs_addmanager: None,
            bdmvs_addmanager_tmp_srch: String::new(),
        }
    }
}
