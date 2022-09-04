//use simple_error::*;
use std::error::Error;
use std::ffi::OsStr;

mod fuse_fs;

use vmux_lib;
use vmux_lib::fs::hellofs_build_from_config;

use crate::fuse_fs::HelloFsRuntimeBD;
use fs::emu_folder_builder::HelloFSFolderBuilder;
use fs::ino_allocator::InoAllocator;

use vmux_lib::bd_cache::BDsCache;
use vmux_lib::handling::Config;
use vmux_lib::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = vmux_lib::CmdLineArgs::new();
    let allow_other = args.has_key("allow_other");
    let cfg_path = args
        .get_key("config")
        .or(Some(Config::default_config_path().display().to_string()))
        .unwrap();
    let mnt_path = args.get_key("mount").or(Some("fuse".to_string())).unwrap();

    println!("Initialising ffms2");
    init_ffms2();

    println!("Loading Config");
    let cfg = Config::new(cfg_path.to_owned());

    let mut builder = HelloFSFolderBuilder::new();
    let mut ino = InoAllocator::new();
    let mut bdbd = BDsCache::new();

    builder = hellofs_build_from_config(&cfg, &mut ino, &mut bdbd, builder);

    let runtime = HelloFsRuntimeBD { cfg, bdbd };

    let mountpoint = &mnt_path;

    let mut optns = vec![/* "-o", "ro",*/ "-o", "fsname=vmux_fs"];

    if allow_other {
        optns.append(&mut vec!["-o", "allow_other"]);
    }
    let options = optns.iter().map(|o| o.as_ref()).collect::<Vec<&OsStr>>();

    println!("FUSE up and running!");
    match fuse::mount(
        fuse_fs::HelloFS::new(cfg_path.to_owned(), builder, runtime, ino),
        mountpoint,
        &options,
    ) {
        Ok(_) => println!("Ok!"),
        Err(err) => {
            dbg!(err);
            println!("Does the fuse folder exist ?");
        }
    }
    Ok(())
}
