/*
EBML HEAD

SEGMENT (SZ) {
SEGMENT_INFO
TRACKS
(CLUSTERS...)
}
*/
pub struct MatroskaBacked {
    pub vmap: VMap,
    pub total_size: u64,
    /* clusters */
}
impl Drop for MatroskaBacked {
    fn drop(&mut self) {
        println!("MBACKED DROPPED");
    }
}

use crate::matroska::*;
use crate::*;

use crate::frame_cache::FrameCache;

use std::sync::*;

fn collect_track_metadata(v: &VSource, a: &[ASource], fram_time: f64) -> Vec<Track> {
    let mut tracks = Vec::new();

    let vt = MatroskaVideoTrack {
        ns_per_frame: (1.0 / (fram_time / 1_000_000_000.0)) as u64,
        width: v.width as _,
        height: v.height as _,
        //TODO:
        color_space: vec![0x49, 0x34, 0x32, 0x30], // I420
        color_range: 1,
        hor_chroma_siting: 1,
        vert_chroma_siting: 2,
    };
    tracks.push(Track {
        track_id: 1,
        track_uid: 2323,
        track_type: TrackType::Video(vt),
        codec: "V_UNCOMPRESSED".to_string(),
    });

    let mut audio_i = 0;
    for audio in a {
        let at = MatroskaAudioTrack {
            channels: audio.channels as _,
            sampling_freq: audio.sample_rate as _,
            bit_depth: audio.bits_per_sample as _,
        };
        let codec = {
            let frmt = match audio.format {
                AFormat::INT => "A_PCM/INT/LIT".to_string(),
                AFormat::FLOAT => "A_PCM/FLOAT/IEEE".to_string(),
                //  _ => panic!("invalid format"),
            };
            frmt
        };
        tracks.push(Track {
            track_id: 2 + audio_i,
            track_uid: 123213123 + audio_i as u64,
            track_type: TrackType::Audio(at),
            codec,
        });
        audio_i += 1;
    }

    tracks
}

struct PacketAudio {
    tmstmp: u64,
    packet_start: u64,
    packet_size: u64,

    track_id: u64,
    //smpl_start: u64,
    //smpl_cnt: u64,
}

fn packetize_audio(audio: &ASource, track_id: u64, tmstmp_scale_f64: f64) -> Vec<PacketAudio> {
    let mut packetized = Vec::new();

    let bps = audio.bits_per_sample;
    let chnls = audio.channels;
    let smplr = audio.sample_rate;

    //let audio_samples_per_cluster = 128;
    //4 packets every second
    let audio_samples_per_cluster = (8192 * 8) / (bps * audio.channels);

    let clstr =
        align_up(audio.num_samples as u64, audio_samples_per_cluster) / audio_samples_per_cluster;

    let mut remain =
        audio.num_samples as u64 * ((audio.bits_per_sample / 8) as u64) * audio.channels as u64;

    for c in 0..clstr {
        let max_cluster_size = ((bps / 8) as u64) * chnls as u64 * audio_samples_per_cluster;

        let cluster_size = if max_cluster_size > remain {
            remain
        } else {
            max_cluster_size
        };
        let tmstmp = (((c * audio_samples_per_cluster) as f64 * (1.0 / smplr as f64))
            * 1_000_000_000.0)
            / tmstmp_scale_f64;

        remain -= cluster_size;

        //let smpl_start = c * audio_samples_per_cluster;
        //let smpl_cnt = cluster_size;
        packetized.push(PacketAudio {
            tmstmp: tmstmp as u64,
            packet_start: (c * audio_samples_per_cluster) * (bps as u64 / 8) * chnls as u64,
            packet_size: cluster_size,

            track_id,
            // smpl_start,
            // smpl_cnt,
        })
    }
    assert_eq!(remain, 0);
    packetized
}

struct PacketVideo {
    tmstmp: u64,
    frm_idx: u64,
}
fn packetize_video(src: &VSource, fram_time: f64, tmstmp_scale_f64: f64) -> Vec<PacketVideo> {
    let mut asd = Vec::with_capacity(src.num_frames as _);

    for f in 0..src.num_frames {
        let fr_time = (((1.0 / fram_time) * f as f64 * 1_000_000_000.0) / tmstmp_scale_f64) as u64;

        asd.push(PacketVideo {
            tmstmp: fr_time,
            frm_idx: f,
        })
    }
    asd
}

fn packetize_allaudio(src: &[ASource], tmstmp_scale_f64: f64) -> Vec<PacketAudio> {
    let mut t = Vec::new();
    for (i, a) in src.iter().enumerate() {
        t.append(&mut packetize_audio(&a, (2 + i) as _, tmstmp_scale_f64));
    }
    t.sort_by(|a, b| a.tmstmp.partial_cmp(&b.tmstmp).unwrap());

    t
}

impl MatroskaBacked {
    pub fn vread(&mut self, addr: u64, size: u64, buf: &mut [u8]) -> u64 {
        let mut cr = std::io::Cursor::new(buf);
        self.vmap.vread(addr, size, &mut cr)
    }

    pub fn new(vsrc: &mut VSource, asrc: &[ASource]) -> MatroskaBacked {
        MatroskaBacked::new_ex(vsrc, asrc, None)
    }

    pub fn new_ex(
        vsrc: &mut VSource,
        asrc: &[ASource],
        chpts: Option<&[MatroksChapter]>,
    ) -> MatroskaBacked {
        println!("MatroskaBacked::new");
        let fram_time = vsrc.framerate_n as f64 / vsrc.framerate_d as f64;

        let mut tmstmp_scale = 100_000;
        let mut tmstmp_scale_f64 = tmstmp_scale as f64;

        let mut succ = false;
        for _ in 0..10 {
            let frame_time_ns = 1_000_000_000.0 * fram_time;
            let min_aviable_duration = tmstmp_scale_f64 * (i16::MAX) as f64;

            println!("{} {}", frame_time_ns, min_aviable_duration);
            if frame_time_ns > min_aviable_duration {
                tmstmp_scale *= 10;
                tmstmp_scale_f64 = tmstmp_scale as f64
            } else {
                succ = true;
            }
        }
        //tmstmp_scale = 100_000_000;
        //tmstmp_scale_f64 = tmstmp_scale as f64;
        if !succ {
            panic!("Could not find suitable tmstmsp_scale");
        } else {
            println!("tmstmsp_scale: {}", tmstmp_scale);
        }

        let mut ebml_head = Vec::new();
        write_ebml_head(&mut ebml_head);

        let mut segment_info_tracks_bytes = Vec::new();

        let duration =
            ((1.0 / (fram_time)) * vsrc.num_frames as f64 * 1_000_000_000.0) / tmstmp_scale_f64;

        matroska_write_segment_info(
            &MatroskaInSegmentInfo {
                timestamp_scale: tmstmp_scale,
                multiplex_app: "VMUX Profi mpleeplel".to_string(),
                writing_app: "FMMUJX Write".to_string(),
                segment_uid: None,
                duration,
            },
            &mut segment_info_tracks_bytes,
        );
        let tracks = collect_track_metadata(&vsrc, &asrc, fram_time);
        matroska_write_tracks(tracks, &mut segment_info_tracks_bytes);

        let mut chapters_bytes = Vec::new();

        if chpts.is_some() {
            matroksa_write_chapters(chpts.unwrap(), &mut chapters_bytes);
        }

        let raw_frame_size = vsrc.frame(0).len();

        let mut cues = Vec::new();

        let clusters_start = chapters_bytes.len() + segment_info_tracks_bytes.len();

        let cluster_vmap = {
            println!("Packetizing audio");
            let packetize_audio = packetize_allaudio(&asrc, tmstmp_scale_f64);
            println!("Packetizing video");
            let packetize_video = packetize_video(&vsrc, fram_time, tmstmp_scale_f64);

            let fcsss = Arc::new(Mutex::new(FrameCache::new(raw_frame_size, 40)));

            /*
            let audios: Vec<Arc<Mutex<AudioSource>>> = src
                .audio
                .into_iter()
                .map(|e| Arc::new(Mutex::new(e)))
                .collect();
                */
            let audios: Vec<ASource> = asrc.into_iter().map(|e| e.clone()).collect();

            println!("building clusters");

            let mut last_packetized_audio_off = 0;

            //  let mut last_vid_cue = 0;
            //  let mut last_aud_cue = 0;
            let mut cluster_vmap = VMap::new_ex(packetize_video.len() * 4);

            cues.reserve(packetize_video.len() + packetize_audio.len());
            let mut blocks = Vec::with_capacity(5);

            for i in 0..packetize_video.len() {
                let pkt = &packetize_video[i];
                let until = if i == packetize_video.len() - 1 {
                    //TODO: fix
                    packetize_video[i].tmstmp + 1000000
                } else {
                    packetize_video[i + 1].tmstmp
                };
                blocks.push((
                    SimpleBlock {
                        track_id: 1 as _,
                        relative_timestamp: 0,
                        flags: 1 << 7,
                        data: Vec::new(),
                    },
                    raw_frame_size as u64,
                ));

                let mut audio_getters =
                    Vec::with_capacity(packetize_audio.len() - last_packetized_audio_off);

                let rnng = last_packetized_audio_off..packetize_audio.len();

                for iia in rnng {
                    let e = &packetize_audio[iia];

                    //Sometimes this still failes (because way more audio than video ???) TOOD: Fix
                    if e.tmstmp > until {
                        //continue;
                        break;
                    } else if e.tmstmp >= pkt.tmstmp {
                        let relative_timestamp_exact = e.tmstmp as i64 - pkt.tmstmp as i64;
                        let relative_timestamp = relative_timestamp_exact as i16;
                        if relative_timestamp_exact != relative_timestamp as i64 {
                            println!(
                                "{}/{} timestamp difference error {} != {}",
                                i,
                                packetize_video.len() as f32,
                                relative_timestamp_exact,
                                relative_timestamp
                            );
                            //tweak timestamp scale
                        }
                        blocks.push((
                            SimpleBlock {
                                track_id: e.track_id,
                                relative_timestamp,
                                flags: 1 << 7,
                                data: Vec::new(),
                            },
                            e.packet_size as _,
                        ));

                        //    if (e.tmstmp - last_aud_cue) > 1_000_000
                        {
                            cues.push(Cue {
                                time: e.tmstmp,
                                track: e.track_id,
                                cluster: clusters_start as u64 + cluster_vmap.total_size(),
                                relative: None,
                            });
                            //        last_aud_cue = e.tmstmp;
                        }

                        let mut audio = audios[e.track_id as usize - 2].clone();

                        let packet_start = e.packet_start;

                        let fna =
                            Box::new(move |addr: u64, size: u64, wrt: &mut dyn Write| -> u64 {
                                //   let audio = audio.lock().unwrap();

                                audio_source_memory_read(packet_start + addr, size, &mut audio, wrt)
                            });
                        audio_getters.push(Some((e.packet_size, fna)));
                        last_packetized_audio_off = (iia + 1) as usize;
                    } else {
                        panic!("Something went wrong");
                    }
                }

                let xx = pkt.frm_idx;
                // let vss = vsss.clone();
                let fcss = fcsss.clone();
                let mut vs = vsrc.clone();
                let video_fna = Box::new(move |addr: u64, size: u64, wrt: &mut dyn Write| -> u64 {
                    let mut fc = fcss.lock().unwrap();

                    if fc.contains(xx) {
                        let frm = fc.find(xx).unwrap();
                        let mm = &frm[addr as usize..(addr + size) as usize];
                        wrt.write(mm).unwrap();
                        mm.len() as _
                    } else {
                        let frame = vs.frame(xx as _);
                        let hsh = bad_hash(&frame);
                        let _ = hsh;
                        //println!("{} {}",xx,hsh);

                        let mm = &frame[addr as usize..(addr + size) as usize];
                        wrt.write(mm).unwrap();
                        let ll = mm.len();
                        fc.push(xx, frame);
                        ll as _
                    }
                });

                //    if (pkt.tmstmp - last_vid_cue) > 1_000_000 {
                cues.push(Cue {
                    time: pkt.tmstmp,
                    track: 1,
                    cluster: clusters_start as u64 + cluster_vmap.total_size(),
                    relative: None,
                });
                //       last_vid_cue = pkt.tmstmp;
                //   }

                let cluster_rslt = matroska_write_cluster_nblock(pkt.tmstmp, &blocks);
                cluster_vmap.add_vec(cluster_rslt[0].clone());
                cluster_vmap.add(raw_frame_size as _, Box::new(VRead::Custom(video_fna)));
                for f in 1..cluster_rslt.len() {
                    let audio_idx = f - 1;

                    cluster_vmap.add_vec(cluster_rslt[f].clone());
                    let gtr = audio_getters[audio_idx].take().unwrap();
                    cluster_vmap.add(gtr.0, Box::new(VRead::Custom(gtr.1)));
                }
                blocks.clear();
            }
            cluster_vmap
        };
        println!("cluster finished");

        /*
        ebml head
        Segment {
        seginfo
        tracks
        <cue>
        clusters
        }
        */
        let cluster_size = cluster_vmap.total_size();

        let mut bytes_cues = Vec::with_capacity(16 * 3 * cues.len());
        {
            println!("Wrigin cues!");
            matroska_write_cues(&cues, &mut bytes_cues);
            let leny = bytes_cues.len();
            for f in &mut cues {
                f.cluster += leny as u64;
            }
            println!("Wrigin cues again!");
            bytes_cues.clear();
            matroska_write_cues(&cues, &mut bytes_cues);
            assert_eq!(leny, bytes_cues.len());
        }
        println!(
            "cues finsihed finished {}   -> {}",
            cues.len(),
            bytes_cues.len()
        );

        let inner_segment_len = segment_info_tracks_bytes.len() as u64
            + chapters_bytes.len() as u64
            + bytes_cues.len() as u64
            + cluster_size as u64;

        let mut segmnt_header = Vec::new();
        ebml_type(MATROSKA_SEGMENT_EBML, &mut segmnt_header);
        ebml_vint_size(inner_segment_len as _, &mut segmnt_header);

        let mut vmap = VMap::new();

        vmap.add_vec(ebml_head);
        vmap.add_vec(segmnt_header);
        vmap.add_vec(segment_info_tracks_bytes);
        vmap.add_vec(chapters_bytes);
        vmap.add_vec(bytes_cues);
        vmap.insert_consume_other(cluster_vmap);
        println!("vmap finsihed finished");
        println!("memory: {}", vmap.memory_footprint());
        println!(
            "memory: {:.2} MiB",
            (vmap.memory_footprint() as f64) / (1024.0 * 1024.0)
        );

        let total_size = vmap.total_size();
        MatroskaBacked { vmap, total_size }
    }
}

fn bad_hash(asd: &[u8]) -> u64 {
    let mut vv = 0u64;
    for f in asd {
        if vv == 0 || vv == 1 {
            vv = 2;
        }
        vv = (vv as u64 * (*f) as u64) % (255 * 255 * 255);
    }
    vv
}

/*
fn frame_to_raw(frame: &ffms2::frame::Frame) -> Vec<u8> {
    let mut vid_frame2 = Vec::new();
    let pix = frame.get_pixel_data();

    let mut i = 0;
    for f in pix {
        if let Some(e) = f {
            if i == 0 {
                vid_frame2.append(&mut e.to_owned());
            } else {
                vid_frame2.append(&mut e[0..e.len() / 2].to_owned());
            }
        }
        i += 1;
    }

    return vid_frame2;
}

fn frame_to_cluster(timestamp: u64, frame: &[u8]) -> Vec<u8> {
    //let mut vid_frame2 = frame_to_raw(frame);
    let vid_frame2 = frame;
    let video_block = SimpleBlock {
        track_id: 1,
        relative_timestamp: 0,
        flags: 1 << 7,
        data: vid_frame2.to_owned(),
    };
    let mut segment = Vec::new();
    matroska_write_cluster(timestamp as _, &vec![video_block], &mut segment);
    segment
}
*/

/*
see and AudioSource as blob of bytes
read size at address
*/
fn audio_source_memory_read(addr: u64, size: u64, audio: &mut ASource, wrt: &mut dyn Write) -> u64 {
    let bytes_per_singular_sample = (audio.bits_per_sample / 8) as u64;
    let bytes_per_sample: u64 = bytes_per_singular_sample * audio.channels as u64;
    let num_smps = audio.num_samples as u64;

    let addr_rounddown = addr - (addr % bytes_per_sample);

    let end = addr + size;
    let end_more = if (end % bytes_per_sample) == 0 {
        end
    } else {
        end + (bytes_per_sample - (addr % bytes_per_sample))
    };

    assert!(end_more / bytes_per_sample <= num_smps);

    let statr_snpsps = addr_rounddown / bytes_per_sample;
    let snpsps = (end_more - addr_rounddown) / bytes_per_sample;

    let start_too_much = addr - addr_rounddown;

    if statr_snpsps >= num_smps - 1 {
        panic!("SHoul never happen {} {}", statr_snpsps, num_smps);
    }

    //    let asds: Vec<u8> =
    //        ffms2::audio::AudioSource::GetAudio(&audio, statr_snpsps as _, snpsps as usize).unwrap();

    let asds = audio.samples(statr_snpsps as _, snpsps as _);

    let acual_red = snpsps * bytes_per_sample;
    let to_want = u64::min(acual_red, (start_too_much) + size);

    let bb = &asds[start_too_much as usize..to_want as usize];
    wrt.write(bb).unwrap();
    to_want - start_too_much
}
