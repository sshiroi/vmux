use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
    Request,
};
use libc::{ENOENT, ENOSYS};
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use vmux_lib::handling::Config;

use crate::y4m_wav_backed_file::{backed_wav_read, backed_y4m_read};
use vmux_lib::bd_cache::AVBDsCache;
use vmux_lib::fs::ino_allocator::InoAllocator;
use vmux_lib::fs::{emu_folder_builder::*, hellofs_build_from_config};

const TTL: Duration = Duration::from_secs(1); // 1 second

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

fn file_attr(inoy: u64, sz: u64, exec: bool) -> FileAttr {
    FileAttr {
        ino: inoy,
        size: sz,
        blocks: 1,
        atime: UNIX_EPOCH, // 1970-01-01 00:00:00
        mtime: UNIX_EPOCH,
        ctime: UNIX_EPOCH,
        crtime: UNIX_EPOCH,
        kind: FileType::RegularFile,
        perm: if exec { 0o744 } else { 0o644 },
        nlink: 1,
        uid: 1000,
        gid: 100,
        rdev: 0,
        flags: 0,
    }
}
fn dir_attr(inoy: u64) -> FileAttr {
    FileAttr {
        ino: inoy,
        size: 23,
        blocks: 1,
        atime: UNIX_EPOCH, // 1970-01-01 00:00:00
        mtime: UNIX_EPOCH,
        ctime: UNIX_EPOCH,
        crtime: UNIX_EPOCH,
        kind: FileType::Directory,
        perm: 0o644,
        nlink: 1,
        uid: 1000,
        gid: 100,
        rdev: 0,
        flags: 0,
    }
}

pub struct HelloFsRuntimeBD {
    pub cfg: Config,
    pub bdbd: AVBDsCache,
}

pub struct HelloFS {
    pub files: Vec<HelloFsEntry>,

    pub runtime: HelloFsRuntimeBD,

    pub ino: InoAllocator,

    reading: bool,
    reload_ino: u64,

    pub out_data_video: Vec<u8>,
    pub out_data_matroska: Vec<u8>,
    pub out_data_audio: Vec<u8>,

    pub conf_path: String,
}

impl HelloFS {
    pub fn new(
        conf_path: String,
        a: HelloFSFolderBuilder,
        runtime: HelloFsRuntimeBD,
        ino: InoAllocator,
    ) -> HelloFS {
        let mut ino = ino;
        HelloFS {
            reload_ino: ino.allocate(),
            files: a.build(),
            ino,
            runtime,
            out_data_matroska: Vec::new(),
            out_data_video: Vec::new(),
            out_data_audio: Vec::new(),
            conf_path,
            reading: false,
        }
    }
    pub fn reload_config(&mut self) {
        for f in &self.files {
            handle_file_dealloc(&mut self.ino, f);
        }
        self.files.clear();
        self.runtime.bdbd = AVBDsCache::new();

        vmux_lib::deint_ffms2();
        vmux_lib::init_ffms2();

        let cfg = Config::new(self.conf_path.clone());
        let builder = hellofs_build_from_config(
            &cfg,
            &mut self.ino,
            &mut self.runtime.bdbd,
            HelloFSFolderBuilder::new(),
        );
        self.files = builder.build();
    }
}

fn handle_file_dealloc(ino: &mut InoAllocator, f: &HelloFsEntry) {
    match f {
        HelloFsEntry::HelloFile(f) => {
            ino.free(f.ino);
        }
        HelloFsEntry::HelloFolder(ff) => {
            ino.free(ff.ino);
            for e in &ff.inner {
                handle_file_dealloc(ino, e)
            }
        }
    }
}

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        //println!("lookup {} {}", parent, name.to_str().unwrap());

        if parent == 1 && name == ".reload_cfg" {
            return reply.entry(&TTL, &file_attr(self.reload_ino, 0, false), 0);
        }
        let asd = {
            let shit = find_by_ino(&self.files, parent);

            if shit.is_some() {
                if let HelloFsEntry::HelloFolder(e) = shit.unwrap() {
                    &e.inner
                } else {
                    //Does not happend
                    &self.files
                }
            } else {
                &self.files
            }
        };
        for f in asd {
            let f_name = f.name();
            if name.to_str() == Some(f_name) {
                match f {
                    HelloFsEntry::HelloFile(f) => {
                        let exec = if let EmuFile::TxtFile(e) = &f.backed {
                            e.1
                        } else {
                            false
                        };
                        return reply.entry(&TTL, &file_attr(f.ino, f.size, exec), 0);
                    }
                    HelloFsEntry::HelloFolder(d) => {
                        return reply.entry(&TTL, &dir_attr(d.ino), 0);
                    }
                }
            }
        }

        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        //println!("getattr {}", ino);
        let ff = find_by_ino(&self.files, ino);
        if let Some(f) = ff {
            match f {
                HelloFsEntry::HelloFile(f) => {
                    let exec = if let EmuFile::TxtFile(e) = &f.backed {
                        e.1
                    } else {
                        false
                    };

                    return reply.attr(&TTL, &file_attr(f.ino, f.size, exec));
                }
                HelloFsEntry::HelloFolder(d) => {
                    return reply.attr(&TTL, &dir_attr(d.ino));
                }
            }
        }

        if ino == self.reload_ino {
            return reply.attr(&TTL, &file_attr(self.reload_ino, 0, false));
        }
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            //0 => reply.attr(&TTL, &file_attr_video(self.vb.vy.y4m_total_file_size)),
            // 3 => reply.attr(&TTL, &file_attr_audio(self.ab.ay.total_file_size)),
            _ => reply.error(ENOENT),
        }
    }

    /// Remove a file.
    fn unlink(&mut self, _req: &Request<'_>, _parent: u64, name: &OsStr, reply: ReplyEmpty) {
        println!("unlink {}", name.to_str().unwrap());
        if name == ".reload_cfg" {
            self.reload_config();
            reply.ok();
            println!("realdaoed ");
            return;
        }
        reply.error(ENOSYS);
    }
    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        assert_eq!(self.reading, false);
        self.reading = true;

        //println!("read {} off: {} size: {} fh: {}", ino, offset, size, fh);
        let f = find_by_ino(&self.files, ino);
        if f.is_none() {
            println!("Find by ino none {}", ino);
            self.reading = false;
            return;
        }
        let f = f.unwrap();

        let f = {
            let asd: *const HelloFsEntry = f;
            let asd: *mut HelloFsEntry = asd as *mut HelloFsEntry;
            asd
        };
        //TODO: why is there an unsafe here ?
        if let HelloFsEntry::HelloFile(f) = unsafe { &mut *f } {
            match &mut f.backed {
                EmuFile::WavFile(a) => {
                    let bfr = &mut self.out_data_audio;
                    bfr.resize(size as _, 0);

                    let wrt = backed_wav_read(&mut a.av, &a.wavy, offset as _, bfr);
                    reply.data(&bfr[0..wrt]);
                }
                EmuFile::Y4MFile(v) => {
                    let bfr = &mut self.out_data_video;
                    bfr.resize(size as _, 0);

                    let wrt = backed_y4m_read(&mut v.av, &mut v.vy, &mut v.fc, offset as _, bfr);
                    reply.data(&bfr[0..wrt]);
                }
                EmuFile::TxtFile(s) => {
                    reply.data(&s.0.as_bytes()[offset as usize..]);
                }

                EmuFile::Matroska(mm) => {
                    //Cpy
                    let mut bfr = &mut self.out_data_matroska;
                    bfr.resize(size as _, 0xEA);
                    let redd = mm.vread(offset as _, size as _, &mut bfr);

                    if redd as usize != size as usize {
                        println!("DIDN't rREAD ENOUGHH {} {}", size, redd);
                    }

                    reply.data(&bfr[0..redd as usize]);
                }
                EmuFile::UnloadedMatroska(unloaded) => {
                    let mut gen = crate::matroska_hellofs::build_singular_matroska_backed(
                        unloaded,
                        &self.runtime.cfg,
                        &mut self.runtime.bdbd,
                    );

                    let mut bfr = &mut self.out_data_matroska;
                    bfr.resize(size as _, 0xEA);
                    let redd = gen.vread(offset as _, size as _, &mut bfr);

                    if redd as usize != size as usize {
                        println!("DIDN't rREAD ENOUGHH {} {}", size, redd);
                    }
                    reply.data(&bfr[0..redd as usize]);

                    //put back
                    f.size = gen.total_size;
                    f.backed = EmuFile::Matroska(gen);
                }
            }
        } else {
            println!("Entrie is not file");
        }
        self.reading = false;
        return;
        //       }
        //   }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        //     println!("readdir {}", ino);
        let rr = if ino == 1 {
            let rr = find_shit(1, 1, &self.files, ino);
            rr.unwrap()
        } else {
            let asd = find_by_ino_ex(&self.files, ino, 1);
            let asd = asd.unwrap();
            let rr = find_shit(asd.0, asd.0, &self.files, ino);
            rr.unwrap()
        };

        let mut rr = rr;
        if ino == 1 {
            rr.push((
                self.reload_ino,
                FileType::RegularFile,
                ".reload_cfg".to_string(),
            ));
        }
        for (i, entry) in rr.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            //println!("reply {} {} ", entry.0, entry.2);
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        reply.ok();
        /*
        if ino == 1 {
            let mut entries = vec![
                (1, FileType::Directory, "."),
                (1, FileType::Directory, ".."),
            ];

            for f in &mut self.files {
                let tt = if let HelloFsEntry::HelloFile(ee) = f {
                    FileType::RegularFile
                } else {
                    FileType::Directory
                };
                entries.push((f.ino(), tt, f.name()));
            }

            for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
                // i + 1 means the index of the next entry
                reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
            return;
        }
        */
    }
}
fn find_shit(
    parent_ino: u64,
    current_ino: u64,
    ents: &[HelloFsEntry],
    target_ino: u64,
) -> Option<Vec<(u64, FileType, String)>> {
    if target_ino == current_ino {
        let mut entries = vec![
            (current_ino, FileType::Directory, ".".to_string()),
            (parent_ino, FileType::Directory, "..".to_string()),
        ];
        for f in ents {
            let tt = if let HelloFsEntry::HelloFile(_) = f {
                FileType::RegularFile
            } else {
                FileType::Directory
            };
            entries.push((f.ino(), tt, f.name().to_owned()));
        }
        return Some(entries);
    } else {
        for f in ents {
            match f {
                HelloFsEntry::HelloFile(_) => {}
                HelloFsEntry::HelloFolder(ff) => {
                    if ff.ino == target_ino {
                        if let Some(e) = find_shit(current_ino, f.ino(), &ff.inner, target_ino) {
                            return Some(e);
                        }
                    }
                }
            };
        }
    }
    None
}

fn find_by_ino(ents: &[HelloFsEntry], target_ino: u64) -> Option<&HelloFsEntry> {
    match find_by_ino_ex(ents, target_ino, 99) {
        Some(e) => Some(e.1),
        None => None,
    }
}

fn find_by_ino_ex(
    ents: &[HelloFsEntry],
    target_ino: u64,
    parent_ino: u64,
) -> Option<(u64, &HelloFsEntry)> {
    for f in ents {
        if f.ino() == target_ino {
            return Some((parent_ino, f));
        }
        match f {
            HelloFsEntry::HelloFile(_) => {}
            HelloFsEntry::HelloFolder(ff) => {
                if let Some(e) = find_by_ino_ex(&ff.inner, target_ino, ff.ino) {
                    return Some(e);
                }
            }
        };
    }
    return None;
}
