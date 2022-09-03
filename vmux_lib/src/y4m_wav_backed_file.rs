use crate::frame_cache::*;
use crate::wavy::WavProps;
use crate::y4m_video_helper::*;
use std::io::*;

use crate::*;

pub struct VideoBackedFile {
    pub av: VSource,
    pub vy: Y4mVideoHelper,
    pub fc: FrameCache,
}
pub struct AudioBackedFile {
    pub av: ASource,
    pub audio_idx: usize,
    pub wavy: WavProps,
}

pub fn backed_y4m_read(
    av: &mut VSource,
    vy: &mut Y4mVideoHelper,
    video_fc: &mut FrameCache,
    offset: u64,
    buffer: &mut [u8],
) -> usize {
    let mut out_data = Cursor::new(buffer);

    //Write header if offset in header
    let wrote_header = if offset < vy.y4m_header_size as _ {
        let ll = vy.get_header(av);
        let mut hdr = ll[offset as usize..ll.len() - offset as usize].to_vec();
        out_data.write(&mut hdr).unwrap(); //read could be smaller than header
        true
    } else {
        false
    };

    //let frames_byte_offset = ((out_data.stream_position().unwrap() as usize + offset as usize) - self.vy.y4m_header_size as usize) as u64;
    let frames_byte_offset = if wrote_header {
        0
    } else {
        offset as u64 - vy.y4m_header_size as u64
    };
    let frames_byte_lower = frames_byte_offset - (frames_byte_offset % vy.y4m_frame_size);

    if offset >= vy.y4m_total_file_size {
        return out_data.stream_position().unwrap() as usize; //always 0
    }
    //println!("{}", too_much);
    let mut frame_add = 0;
    loop {
        let too_much = if frame_add == 0 {
            frames_byte_offset - frames_byte_lower
        } else {
            0
        };
        let frame = (frames_byte_lower / vy.y4m_frame_size) + frame_add;

        let frame_cached = video_fc.contains(frame as _);

        if frame_cached {
            let e = video_fc.find(frame as _).unwrap();
            //fastpath
            let bf = &e[too_much as usize..];

            let rslt = out_data.write(&bf).ok().unwrap();
            if rslt != bf.len() {
                return out_data.stream_position().unwrap() as _;
            }
        } else {
            let (a, b) = vy.read_frame(av, frame as _);
            let frame_data = &b[a..];
            video_fc.push(frame as _, frame_data.to_vec());
            continue;
        };

        frame_add += 1;
    }
}

pub fn backed_wav_read(
    audio_source: &mut ASource,
    ay: &WavProps,
    offset: u64,
    buffer: &mut [u8],
) -> usize {
    let read_siuze = buffer.len() as u64;

    let mut out_data = Cursor::new(buffer);
    let wav_total_header_size = 12 + 24 + 4 + 4;

    let wrote_header = if offset < wav_total_header_size {
        let mut hdr = crate::wavy::custom_wav_header(&audio_source, &ay).unwrap();
        out_data.write(&mut hdr).unwrap(); //read could be smaller than header
        true
    } else {
        false
    };
    let audio_sample_byteidx = if wrote_header {
        0
    } else {
        offset as u64 - wav_total_header_size as u64
    };

    let size = if wrote_header {
        read_siuze - wav_total_header_size
    } else {
        read_siuze
    };

    let asdasd = audio_sample_byteidx % ay.bytes_per_sample as u64;
    let smpl_byteidx_loer = audio_sample_byteidx - asdasd;

    let sanple = smpl_byteidx_loer / ay.bytes_per_sample as u64;

    let to_much = audio_sample_byteidx - smpl_byteidx_loer;

    let uproadsize =
        size as u64 - (size as u64 % ay.bytes_per_sample as u64) + ay.bytes_per_sample as u64;

    let snpsps = uproadsize as u64 / ay.bytes_per_sample as u64;

    if sanple + snpsps >= ay.num_samples - 1 {
        return out_data.stream_position().unwrap() as _;
    }
    //let asds: Vec<u8> = ffms2::audio::AudioSource::GetAudio(
    //    &audio_source,
    //    (ay.sample_offset + sanple) as _,
    //    (snpsps) as usize,
    //)
    //.unwrap();
    let asds = audio_source.samples(sanple as _, snpsps as _);
    out_data
        .write(&asds[to_much as usize..to_much as usize + size as usize])
        .unwrap();
    return out_data.stream_position().unwrap() as _;
}
