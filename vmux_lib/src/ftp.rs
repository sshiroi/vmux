use std::io::*;
use std::net::{TcpListener, TcpStream};

use crate::{
    bd_cache::AVBDsCache, fs::HelloFSFile, handling::Config,
    matroska_hellofs::build_singular_matroska_backed,
};

use std::sync::mpsc;

#[derive(Debug)]
pub enum Comd {
    Exists(String, mpsc::Sender<bool>),
    GetFileData(String, u64, u64, TcpStream, mpsc::Sender<(u64, TcpStream)>),
}

pub fn build_fls(bdbd: &mut AVBDsCache, cfg: &Config) -> Vec<HelloFsEntry> {
    use crate::*;

    let mut ino = fs::InoAllocator::new();
    let mut builder = fs::HelloFSFolderBuilder::new();

    init_ffms2();

    builder = fs::hellofs_build_from_config(cfg, &mut ino, bdbd, builder);

    builder.build()
}

fn find_entry_by_path<'a>(pth: &str, a: &'a [HelloFsEntry]) -> Option<&'a HelloFSFile> {
    for e in a {
        match e {
            HelloFsEntry::HelloFile(e) => {
                if pth == e.name {
                    return Some(e);
                }
            }
            HelloFsEntry::HelloFolder(fld) => {
                if pth.starts_with(&fld.name) {
                    let mut stripp = pth.chars();
                    for _ in 0..fld.name.chars().count() {
                        stripp.next();
                    }
                    stripp.next();
                    let new = stripp.as_str();
                    return find_entry_by_path(new, &fld.inner);
                }
            }
        }
    }
    return None;
}

fn find_entry_by_path_mut<'a>(pth: &str, a: &'a mut [HelloFsEntry]) -> Option<&'a mut HelloFSFile> {
    for e in a {
        match e {
            HelloFsEntry::HelloFile(e) => {
                if pth == e.name {
                    return Some(e);
                }
            }
            HelloFsEntry::HelloFolder(fld) => {
                if pth.starts_with(&fld.name) {
                    let mut stripp = pth.chars();
                    for _ in 0..fld.name.chars().count() {
                        stripp.next();
                    }
                    stripp.next();
                    let new = stripp.as_str();
                    return find_entry_by_path_mut(new, &mut fld.inner);
                }
            }
        }
    }
    return None;
}
use crate::fs::EmuFile;

pub fn spawn_media_processing(fls: Vec<HelloFsEntry>, cfg: Config) -> mpsc::Sender<Comd> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut fls = fls;
        let mut bdbd = AVBDsCache::new();
        loop {
            match rx.recv() {
                Ok(e) => {
                    //  println!("rcvd {:?}",e);
                    match e {
                        Comd::GetFileData(file, offset, max_size_want, mut whr, txt) => {
                            let entry = find_entry_by_path_mut(&file, &mut fls);
                            if let Some(e) = entry {
                                match &mut e.backed {
                                    crate::fs::EmuFile::WavFile(_) => todo!(),
                                    crate::fs::EmuFile::Y4MFile(_) => todo!(),
                                    crate::fs::EmuFile::Matroska(mm) => {
                                        //   let mut buf = vec![0u8; max_size_want as usize];
                                        //mm.vread(offset,max_size_want,&mut buf);

                                        struct Proxy {
                                            strm: TcpStream,
                                            error: bool,
                                        }
                                        impl Write for Proxy {
                                            fn write(&mut self, buf: &[u8]) -> Result<usize> {
                                                match self.strm.write(buf) {
                                                    Ok(e) => Ok(e),
                                                    Err(_) => {
                                                        self.error = true;
                                                        Ok(0)
                                                    }
                                                }
                                            }

                                            fn flush(&mut self) -> Result<()> {
                                                match self.strm.flush() {
                                                    Ok(_) => Ok(()),
                                                    Err(_) => {
                                                        self.error = true;
                                                        Ok(())
                                                    }
                                                }
                                            }
                                        }
                                        let mut proxy = Proxy {
                                            strm: whr,
                                            error: false,
                                        };
                                        let wrttt =
                                            mm.vmap.vread(offset, max_size_want, &mut proxy);
                                        let wrttt = if proxy.error { 0 } else { wrttt };
                                        txt.send((wrttt, proxy.strm)).unwrap();
                                        continue;
                                    }
                                    crate::fs::EmuFile::UnloadedMatroska(unloaded) => {
                                        println!("Buildin gmatroska");
                                        let mut mm = build_singular_matroska_backed(
                                            unloaded, &cfg, &mut bdbd,
                                        );
                                        let red = mm.vmap.vread(offset, max_size_want, &mut whr);
                                        txt.send((red, whr)).unwrap();

                                        //put back
                                        e.size = mm.total_size;
                                        e.backed = EmuFile::Matroska(mm);
                                    }
                                    crate::fs::EmuFile::TxtFile(_) => todo!(),
                                };
                            } else {
                                println!("Invalid file");
                            }
                        }
                        Comd::Exists(file, retr) => {
                            let has = find_entry_by_path(&file, &fls);
                            retr.send(has.is_some()).unwrap();
                        }
                    }
                }
                Err(_) => break,
            }
        }
        println!("Meida processor exited");
    });
    println!("Thread init up");
    tx
}

fn send_stra(num: u64, data: &str) -> Vec<u8> {
    format!("{} {}\r\n", num, data).as_bytes().to_vec()
}

use crate::fs::HelloFsEntry;

fn hanle_con(mut ftp_main_connection: TcpStream, asd: mpsc::Sender<Comd>) {
    println!("Got connection");

    ftp_main_connection
        .write(&send_stra(220, "FTP SERVER READY"))
        .unwrap();

    let mut pasv_lstt = None;

    // let mut thrd_tx: Option<Sender<i64>> = None;

    let mut start_stthin = 0;

    let amount_of_retr_threads = std::sync::Arc::new(std::sync::Mutex::new(0));

    loop {
        let mut buf = vec![0; 1024];
        let result = ftp_main_connection.read(&mut buf);

        match result {
            Ok(data) => {
                println!("red {}", data);
                if data != 0 {
                    let straa = std::str::from_utf8(&buf[0..data]).unwrap();

                    println!("stra: {}", straa);
                    if straa.starts_with("USER") {
                        println!("Goit user");
                        //     stream.write(&send_stra(331,"Password required")).unwrap();
                        //     stream.flush().unwrap();
                        ftp_main_connection
                            .write(&send_stra(230, "Password required"))
                            .unwrap();
                        ftp_main_connection.flush().unwrap();
                    } else if straa.starts_with("PASS") {
                        println!("Goit pass");
                        ftp_main_connection
                            .write(&send_stra(230, "olkok."))
                            .unwrap();
                        ftp_main_connection.flush().unwrap();
                    } else if straa.starts_with("PWD") {
                        println!("Goit pwd");
                        ftp_main_connection.write(&send_stra(257, "\"/\"")).unwrap();
                        ftp_main_connection.flush().unwrap();
                    } else if straa.starts_with("TYPE") {
                        println!("Goit type I");
                        ftp_main_connection.write(&send_stra(200, "pen")).unwrap();
                        ftp_main_connection.flush().unwrap();
                    } else if straa.starts_with("REST") {
                        println!("Got rest {}", straa);

                        let splt: Vec<&str> = straa.split(" ").collect();
                        let asd: u64 = splt[1].trim().parse().unwrap();

                        start_stthin = asd;

                        ftp_main_connection.write(&send_stra(350, "pen")).unwrap();
                        ftp_main_connection.flush().unwrap();
                    } else if straa.starts_with("SIZE") {
                        println!("Got size {}", straa);

                        //let mut splt = straa.chars();
                        //for x in 0.."SIZE /".len() {
                        //    splt.next();
                        //}
                        //let fname = splt.as_str().to_string();
                        //let fname = fname.trim();
                        //let mut pth = PathBuf::from("fuse");
                        //pth.push(fname);
                        //if pth.exists() {
                        //    let sz = std::fs::metadata(pth).unwrap();
                        //    println!("RETURN SIZE {}",sz.len());
                        //    stream.write(&send_stra(213,&format!("{}",sz.len()))).unwrap();
                        //    stream.flush().unwrap();
                        //   }else {
                        //        println!("FILE not exits");
                        //        stream.write(&send_stra(999,"eroror")).unwrap();
                        //        stream.flush().unwrap();
                        //}

                        ftp_main_connection
                            .write(&send_stra(213, &format!("{}", 99999999)))
                            .unwrap();
                        ftp_main_connection.flush().unwrap();
                    } else if straa.starts_with("RETR") {
                        println!("Got RETR");

                        let mut splt = straa.chars();
                        for _ in 0.."RETR /".len() {
                            splt.next();
                        }
                        let fname = splt.as_str().to_string();
                        let fname = fname.trim().to_string();

                        let (tx, rx) = mpsc::channel();
                        let req = Comd::Exists(fname.to_string(), tx);
                        asd.send(req).unwrap();

                        let exists = rx.recv().unwrap();
                        if exists {
                            println!("Path exists");
                            ftp_main_connection
                                .write(&send_stra(150, "RETR is ok"))
                                .unwrap();
                            ftp_main_connection.flush().unwrap();

                            let mut lstt: Option<TcpStream> = Some(pasv_lstt.take().unwrap());

                            //let (tx, lcl_rx) = channel();
                            //let lcl_rx: Receiver<i64> = lcl_rx;

                            //thrd_tx = Some(tx);
                            let start_stthin = start_stthin as u64;
                            let mut stream = ftp_main_connection.try_clone().unwrap();
                            let asd = asd.clone();
                            let amount_of_retr_threads = amount_of_retr_threads.clone();
                            std::thread::spawn(move || {
                                println!("read THread up");

                                {
                                    let mut asduuu = amount_of_retr_threads.lock().unwrap();
                                    *asduuu = *asduuu + 1;
                                }

                                //                                let mut f = File::open(pth).unwrap();
                                //                                f.seek(SeekFrom::Start(start_stthin)).unwrap();
                                let mut offst = start_stthin;

                                loop {
                                    //match lcl_rx.try_recv() {
                                    //    Ok(e) => {
                                    //        if e == -1 {
                                    //            println!("Broke thr hread");
                                    //            break;
                                    //        }
                                    //    },
                                    //    Err(asd) => {
                                    //        break;
                                    //    }
                                    //}
                                    let (tx, rx) = mpsc::channel();
                                    let req = Comd::GetFileData(
                                        fname.to_string(),
                                        offst,
                                        1024 * 64,
                                        lstt.take().unwrap(),
                                        tx,
                                    );
                                    asd.send(req).unwrap();
                                    let buf = rx.recv().unwrap();
                                    lstt = Some(buf.1);

                                    if buf.0 == 0 {
                                        println!("Exit because 0");
                                        let mut asduuu = amount_of_retr_threads.lock().unwrap();
                                        if !(*asduuu > 1) {
                                            stream.write(&send_stra(226, "Transfer ok")).unwrap();
                                            stream.flush().unwrap();
                                        } else {
                                            println!("Didnt sedd tranfer ok because the new on just got started");
                                        }
                                        *asduuu = *asduuu - 1;
                                        break;
                                    } else {
                                        //lstt.write(&buf).unwrap();
                                        offst += buf.0 as u64;
                                    }
                                    //                                    let mut buf = vec![0u8; 1024];
                                    //                                    let red = f.read(&mut buf).unwrap();
                                    //                                    if red == 0 {
                                    //                                        stream.write(&send_stra(226,"Transfer ok")).unwrap();
                                    //                                        stream.flush().unwrap();
                                    //                                        break;
                                    //                                    }else {
                                    //                                        lstt.write(&buf[0..red]).unwrap();
                                    //                                    }
                                }
                            });
                        } else {
                            println!("Path does not  exists");

                            ftp_main_connection.write(&send_stra(999, "ERRRR")).unwrap();
                            ftp_main_connection.flush().unwrap();
                        }
                    } else if straa.starts_with("PASV") {
                        println!("Got PASV {}", straa);

                        let lstt = TcpListener::bind("0.0.0.0:0").unwrap();

                        let pasv_port = lstt.local_addr().unwrap().port();
                        println!("port {}", pasv_port);

                        let octet = match ftp_main_connection.local_addr().unwrap().ip() {
                            std::net::IpAddr::V4(e) => e.octets(),
                            std::net::IpAddr::V6(_) => panic!("v6"),
                        };

                        let (a, b, c, d, e, f) = (
                            octet[0],
                            octet[1],
                            octet[2],
                            octet[3],
                            pasv_port >> 8,
                            pasv_port & 0xff,
                        );
                        ftp_main_connection
                            .write(&send_stra(
                                227,
                                &format!(
                                    "Enter passive mode ({},{},{},{},{},{})",
                                    a, b, c, d, e, f
                                ),
                            ))
                            .unwrap();
                        ftp_main_connection.flush().unwrap();

                        println!("Wait con");
                        let conn = lstt.accept().unwrap();
                        println!("Put con");
                        pasv_lstt = Some(conn.0);
                    } else if straa.starts_with("ABOR") {
                        //                        if let Some(e) = &mut thrd_tx {
                        //                            println!("Sent aborrt");
                        //                            let _ = e.send(-1);
                        //                        }
                        ftp_main_connection.write(&send_stra(226, "abort")).unwrap();
                        ftp_main_connection.flush().unwrap();
                    } else {
                        ftp_main_connection
                            .write(&send_stra(500, "gnot iht."))
                            .unwrap();
                        ftp_main_connection.flush().unwrap();
                    }
                }
                if data == 0 {
                    break;
                }
            }
            Err(e) => {
                println!("error reading: {}", e);
                break;
            }
        }
    }
}
use std::sync::*;

fn new_server(port: u16, asd2: mpsc::Sender<Comd>, exit_cond: Arc<Mutex<bool>>) {
    let asd = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

    let local_addr = asd.local_addr().unwrap();

    {
        let exit_cond = exit_cond.clone();
        std::thread::spawn(move || loop {
            let a = {
                let a = exit_cond.lock().unwrap();
                *a
            };
            if a {
                let _ = TcpStream::connect(local_addr).unwrap();
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(60));
        });
    }
    std::thread::spawn(move || {
        for f in asd.incoming() {
            let asd2 = asd2.clone();
            match f {
                Ok(s) => {
                    {
                        let asd22 = exit_cond.lock().unwrap();
                        if *asd22 {
                            break;
                        }
                    }
                    std::thread::spawn(|| {
                        hanle_con(s, asd2);
                    });
                }
                Err(_) => break,
            }
        }
    });
}

pub fn spawn_combined(cfg: Config, port: u16, fls: Vec<HelloFsEntry>) -> Arc<Mutex<bool>> {
    let amtx = Arc::new(Mutex::new(false));
    let main = spawn_media_processing(fls, cfg);
    new_server(port, main, amtx.clone());

    amtx
}
