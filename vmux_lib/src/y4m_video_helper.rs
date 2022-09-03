use crate::*;

pub struct Y4mVideoHelper {
    pub frame_offset: u64,
    pub num_frames: u64,

    pub y4m_header_size: u64,
    pub y4m_frame_size: u64,
    pub y4m_total_file_size: u64,
}

pub fn read_video(av: &mut VSource, rc: ClipRecut) -> Y4mVideoHelper {
    //let vsa = &mut av.vs;
    //let vpa = &mut av.vp;

    //let frame = Frame::GetFrame(vsa, 0).unwrap();
    //let fr = frame.get_frame_resolution();
    let frame = av.frame(0);

    let (header_size, frame_size, _) = crate::y4my::do_one_frame(
        y4m::Colorspace::C420,
        av.width,
        av.height,
        av.framerate_n,
        av.framerate_d,
        av.sar_n,
        av.sar_d,
        &frame,
    );
    let num_framesa = av.num_frames;

    let frame_offset = ((rc.offset / 90_000) as f64
        * (av.framerate_n as f64 / av.framerate_d as f64))
        .round() as u64;

    let num_framesa = u64::min(
        if let Some(dur) = rc.duration {
            ((dur / 90_000) as f64 * (av.framerate_n as f64 / av.framerate_d as f64)).round() as u64
        } else {
            u64::MAX
        },
        num_framesa - frame_offset,
    );
    let total_file_size = header_size + num_framesa * frame_size;

    println!("{}/{}", av.framerate_n, av.framerate_d);
    println!("{}/{}", av.width, av.height);
    println!("{}", num_framesa);

    Y4mVideoHelper {
        frame_offset,
        num_frames: num_framesa,

        y4m_header_size: header_size,
        y4m_frame_size: frame_size,
        y4m_total_file_size: total_file_size,
    }
}
impl Y4mVideoHelper {
    pub fn read_frame(&mut self, v: &mut VSource, frame: u64) -> (usize, Vec<u8>) {
        //        let frame = Frame::GetFrame(&mut av.vs, (self.frame_offset + frame) as _).unwrap();
        let frame = v.frame(frame as _);

        //let start = Instant::now();

        let (header_size, _, data) = crate::y4my::do_one_frame(
            y4m::Colorspace::C420,
            v.width,
            v.height,
            v.framerate_n,
            v.framerate_d,
            v.sar_n,
            v.sar_d,
            &frame,
        );
        //let end = Instant::now();
        //  println!("fram took {}ms",(end-start).as_millis());
        //data[header_size as usize..].to_vec()
        (header_size as usize, data)
    }
    pub fn get_header(&mut self, v: &mut VSource) -> Vec<u8> {
        let frame = v.frame(0 as _);
        let (header_size, _, data) = crate::y4my::do_one_frame(
            y4m::Colorspace::C420,
            v.width,
            v.height,
            v.framerate_n,
            v.framerate_d,
            v.sar_n,
            v.sar_d,
            &frame,
        );
        data[0..header_size as usize].to_vec()
    }
}
