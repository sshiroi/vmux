use std::io::Write;
use std::sync::*;
use y4m::Colorspace;

fn get_plane_sizes(width: usize, height: usize, colorspace: Colorspace) -> (usize, usize, usize) {
    let y_plane_size = width * height * colorspace.get_bytes_per_sample();

    let c420_chroma_size =
        ((width + 1) / 2) * ((height + 1) / 2) * colorspace.get_bytes_per_sample();
    let c422_chroma_size = ((width + 1) / 2) * height * colorspace.get_bytes_per_sample();

    let c420_sizes = (y_plane_size, c420_chroma_size, c420_chroma_size);
    let c422_sizes = (y_plane_size, c422_chroma_size, c422_chroma_size);
    let c444_sizes = (y_plane_size, y_plane_size, y_plane_size);

    match colorspace {
        Colorspace::Cmono => (y_plane_size, 0, 0),
        Colorspace::C420
        | Colorspace::C420p10
        | Colorspace::C420p12
        | Colorspace::C420jpeg
        | Colorspace::C420paldv
        | Colorspace::C420mpeg2 => c420_sizes,
        Colorspace::C422 | Colorspace::C422p10 | Colorspace::C422p12 => c422_sizes,
        Colorspace::C444 | Colorspace::C444p10 | Colorspace::C444p12 => c444_sizes,
    }
}

#[derive(Clone)]
struct Shit {
    buf: Arc<Mutex<Option<Vec<u8>>>>,
}

impl Shit {
    pub fn len(&self) -> usize {
        let lck = self.buf.lock().unwrap();

        lck.as_ref().unwrap().len()
    }
    pub fn buf(&self) -> Vec<u8> {
        let mut lck = self.buf.lock().unwrap();

        lck.take().unwrap()
    }
}
impl Write for Shit {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lck = self.buf.lock().unwrap();

        lck.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut lck = self.buf.lock().unwrap();

        lck.as_mut().unwrap().flush()
    }
}

pub fn do_one_frame(
    colspace: y4m::Colorspace,
    width: u64,
    height: u64,
    fps_n: u64,
    fps_d: u64,
    sar_n: u64,
    sar_d: u64,
    frame_d: &[u8],
) -> (u64, u64, Vec<u8>) {
    let shit = Shit {
        buf: Arc::new(Mutex::new(Some(Vec::new()))),
    };
    let (y_len, u_len, v_len) = get_plane_sizes(width as _, height as _, y4m::Colorspace::C420);

    /*
    let colspace = if frame.ColorSpace == 1 || frame.ColorSpace == 2 {
        y4m::Colorspace::C420
    } else {
        //y4m::Colorspace::C420
        panic!("EROROR {}", 2);
    };
    */
    let mut encoder = y4m::EncoderBuilder::new(
        width as _,
        height as _,
        y4m::Ratio::new(fps_n as _, fps_d as _),
    )
    .with_pixel_aspect(y4m::Ratio::new(sar_n as _, sar_d as _))
    .with_colorspace(colspace)
    .write_header(shit.clone())
    .unwrap();

    // let pd = frame.get_pixel_data();
    let out_frame = y4m::Frame::new(
        [
            &frame_d[0..y_len],
            &frame_d[y_len..y_len + u_len],
            &frame_d[y_len + u_len..y_len + u_len + v_len],
        ],
        None,
    );

    let header_size = shit.len();
    encoder.write_frame(&out_frame).unwrap();
    let frame_size = shit.len() - header_size;

    (header_size as _, frame_size as _, shit.buf())
}
