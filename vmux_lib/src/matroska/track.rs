use super::ebml::*;
pub struct MatroskaVideoTrack {
    pub ns_per_frame: u64,
    pub width: u16,
    pub height: u16,
    pub color_space: Vec<u8>,

    pub color_range: u8,
    pub hor_chroma_siting: u8,
    pub vert_chroma_siting: u8,
}
pub struct MatroskaAudioTrack {
    pub channels: u8,
    pub sampling_freq: f64,
    pub bit_depth: u8,
}

pub enum TrackType {
    Video(MatroskaVideoTrack),
    Audio(MatroskaAudioTrack),
}

pub struct Track {
    pub track_id: u8,
    pub track_uid: u64,
    pub track_type: TrackType,
    pub codec: String,
}

pub fn matroska_write_tracks(tracks: Vec<Track>, veca: &mut impl Write) {
    let mut tracks_inner = Vec::new();

    for t in &tracks {
        let mut track_buf: Vec<u8> = Vec::new();

        //TrackNumber
        ebml_write_u8(0xD7, t.track_id, &mut track_buf);

        //TrackUID
        ebml_write_u64thingy(0x73C5, t.track_uid, &mut track_buf);

        //FlagLacgin
        ebml_write_u8(0x9C, 0, &mut track_buf);

        //CodecID
        ebml_write_ascii(0x86, &t.codec, &mut track_buf);

        match &t.track_type {
            TrackType::Video(v) => {
                //TrackType
                ebml_write_u8(0x83, 1, &mut track_buf);

                //DefaultDuration
                ebml_write_u64thingy(0x23E383, v.ns_per_frame, &mut track_buf);
            }
            TrackType::Audio(_) => {
                //TrackType
                ebml_write_u8(0x83, 2, &mut track_buf);
            }
        }
        match &t.track_type {
            TrackType::Video(e) => {
                matroska_write_video_track(e, &mut track_buf);
            }
            TrackType::Audio(e) => {
                matroska_write_audio_track(&e, &mut track_buf);
            }
        }

        //TrackEntry
        ebml_write_binary(0xAE as _, &track_buf, &mut tracks_inner);
    }

    //Tracks
    ebml_write_binary(0x1654AE6B as _, &tracks_inner, veca);
}

pub fn matroska_write_video_track(t: &MatroskaVideoTrack, veca: &mut impl Write) {
    let mut video_inner: Vec<u8> = Vec::new();

    //PixelWidth
    ebml_write_u64thingy(0xB0, t.width as _, &mut video_inner);
    //PixelHeight
    ebml_write_u64thingy(0xBA, t.height as _, &mut video_inner);

    //0 - undetermined,
    //1 - interlaced,
    //2 - progressive
    //FlagInterlaced
    ebml_write_u8(0x9A, 2, &mut video_inner);

    //0 - progressive,
    //1 - tff,
    //2 - undetermined,
    //6 - bff,
    //9 - bff(swapped),
    //14 - tff(swapped)
    //FieldOrder
    //ebml_write_u8(0x9D, 1, &mut video_inner);

    ebml_write_binary(0x2EB524, &t.color_space, &mut video_inner);

    let mut color_inner: Vec<u8> = Vec::new();
    //VideoColour
    {
        //ChromaSitingHorz
        ebml_write_u8(0x55B7, t.hor_chroma_siting, &mut color_inner);

        //ChromaSitingVert
        ebml_write_u8(0x55B8, t.vert_chroma_siting, &mut color_inner);

        //Range
        ebml_write_u8(0x55B9, t.color_range, &mut color_inner);
    }
    ebml_write_binary(0x55b0, &color_inner, &mut video_inner);

    //Video
    ebml_write_binary(0xE0, &video_inner, veca);
}

pub fn matroska_write_audio_track(a: &MatroskaAudioTrack, veca: &mut impl Write) {
    let mut audio_inner: Vec<u8> = Vec::new();

    //SamplingFrequency
    ebml_write_f64(0xB5, a.sampling_freq, &mut audio_inner);

    //BitDepth
    ebml_write_u8(0x6264, a.bit_depth, &mut audio_inner);

    //Channels
    ebml_write_u8(0x9F, a.channels, &mut audio_inner);

    //Video
    ebml_write_binary(0xE1, &audio_inner, veca);
}
