pub mod emu_folder_builder;
pub mod ino_allocator;

pub use emu_folder_builder::*;
pub use ino_allocator::*;

use crate::bd_cache::BDsCache;
use crate::handling::Config;

pub fn hellofs_build_from_config(
    cfg: &Config,
    ino: &mut InoAllocator,
    bdbd: &mut BDsCache,
    builder: HelloFSFolderBuilder,
) -> HelloFSFolderBuilder {
    let mut main_builder = builder;

    for (_, f) in cfg.folders.iter().enumerate() {
        if !f.show {
            continue;
        }
        let mut folder_builder = HelloFSFolderBuilder::new();

        if f.full_load {
            folder_builder =
                crate::matroska_hellofs::mkvs_from_remux_folder(ino, f, &cfg, folder_builder);
            main_builder = main_builder.folder(ino, &f.name, folder_builder);
        } else {
            let f2 =
                crate::matroska_hellofs::prepare_check_entries(&cfg, &f.name, &f.entries, bdbd);

            for e in f2 {
                let nam =
                    crate::matroska_hellofs::build_singular_remux_file_name(&f.file_prefix, &e);

                folder_builder = folder_builder.unfinished_matroska(ino, &nam, e);
            }
            main_builder = main_builder.folder(ino, &f.name, folder_builder);
        }
    }
    main_builder
}
