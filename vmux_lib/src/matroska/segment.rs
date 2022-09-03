//use super::track::*;
use super::ebml::*;

pub const MATROSKA_SEGMENT_EBML: u64 = 0x18538067;

/*
pub fn write_matroska_file_fully<F: FnOnce(&mut Vec<u8>)>(
    tracks: Vec<Track>,
    duration: f64,
    write_clusters: F,
) -> Vec<u8> {
    let mut veca = Vec::new();
    write_ebml_head(&mut veca);

    write_segment(tracks, duration, write_clusters, &mut veca);
    veca
}
 */

pub struct MatroskaInSegmentInfo {
    pub timestamp_scale: u64,
    pub multiplex_app: String,
    pub writing_app: String,
    pub segment_uid: Option<Vec<u8>>,
    pub duration: f64,
}
/*
pub fn write_segment<F: FnOnce(&mut Vec<u8>)>(
    tracks: Vec<Track>,
    duration: f64,
    write_cluster: F,
    veca: &mut impl Write,
) {
    let mut segment_info: Vec<u8> = Vec::new();

    matroska_write_segment_info(
        &MatroskaInSegmentInfo {
            timestamp_scale: 1000,
            multiplex_app: "VMUX MUX".to_string(),
            writing_app: "VMUX Write".to_string(),
            segment_uid: None,
            duration,
        },
        &mut segment_info,
    );
    let mut tracks_bytes: Vec<u8> = Vec::new();

    matroska_write_tracks(tracks, &mut tracks_bytes);

    let mut segment: Vec<u8> = Vec::new();
    segment.append(&mut segment_info);
    segment.append(&mut tracks_bytes);

    write_cluster(&mut segment);

    //Segment
    ebml_write_binary(MATROSKA_SEGMENT_EBML as _, &segment, veca);
}
*/

pub fn matroska_write_segment_info(seginfo: &MatroskaInSegmentInfo, veca: &mut impl Write) {
    let mut inner_veca: Vec<u8> = Vec::new();

    //TimestampScale
    ebml_write_u64thingy(0x2AD7B1, seginfo.timestamp_scale, &mut inner_veca);

    //MuxingApp
    ebml_write_utf8(0x4D80, &seginfo.multiplex_app, &mut inner_veca);

    //WritingApp
    ebml_write_utf8(0x5741, &seginfo.writing_app, &mut inner_veca);

    if let Some(uid) = &seginfo.segment_uid {
        //SegmentUID
        ebml_write_binary(0x73A4, &uid, &mut inner_veca);
    }
    //Duration
    ebml_write_binary(0x4489, &seginfo.duration.to_be_bytes(), &mut inner_veca);

    //Segment info
    ebml_write_binary(0x1549A966 as _, &inner_veca, veca);
}

pub struct SimpleBlock {
    pub track_id: u64,
    pub relative_timestamp: i16,
    pub flags: u8,
    pub data: Vec<u8>,
}

pub fn matroska_write_cluster(timestamp: u64, blocks: &[SimpleBlock], veca: &mut impl Write) {
    let mut cluster_inner: Vec<u8> = Vec::new();

    //Timestamp
    ebml_write_u64thingy(0xE7, timestamp, &mut cluster_inner);

    for b in blocks {
        let mut block_inner: Vec<u8> = Vec::new();

        ebml_vint_size(b.track_id, &mut block_inner);
        block_inner
            .write(&b.relative_timestamp.to_be_bytes())
            .unwrap();
        block_inner.write(&[b.flags]).unwrap();
        block_inner.write(&b.data).unwrap();

        ebml_write_binary(0xA3, &block_inner, &mut cluster_inner);
    }

    //Cluster
    ebml_write_binary(0x1F43B675, &cluster_inner, veca);
}

pub fn matroska_predict_cluster_size(blocks: &[(u64, usize)]) -> u64 {
    let mut junk = Vec::new();
    let mut sz = 0;
    //Timestamp
    sz += 1 + 1 + 8;

    for b in blocks {
        //Trackid
        assert!(b.0 < 100);
        sz += 1;
        //relative timestamp
        sz += 2;
        //flags
        sz += 1;
        sz += b.1;

        ebml_vint_size(b.1 as _, &mut junk);
        sz += 1 + junk.len();
        junk.clear();
    }
    ebml_vint_size(sz as _, &mut junk);
    sz += junk.len();
    sz += 4;
    sz as _
}

pub fn matroska_write_cluster_1block_nodata(
    timestamp: u64,
    b: SimpleBlock,
    data_len: u64,
    veca: &mut impl Write,
) {
    let mut cluster_inner: Vec<u8> = Vec::new();

    //Timestamp
    ebml_write_u64thingy(0xE7, timestamp, &mut cluster_inner);

    let mut block_inner: Vec<u8> = Vec::new();

    ebml_vint_size(b.track_id, &mut block_inner);
    block_inner
        .write(&b.relative_timestamp.to_be_bytes())
        .unwrap();
    block_inner.write(&[b.flags]).unwrap();
    //block_inner.write(&b.data).unwrap();

    ebml_type(0xA3, &mut cluster_inner);
    ebml_vint_size((block_inner.len() as u64) + data_len, &mut cluster_inner);
    cluster_inner.write(&block_inner).unwrap();

    ebml_type(0x1F43B675, veca);
    ebml_vint_size((cluster_inner.len() + data_len as usize) as _, veca);
    veca.write(&cluster_inner).unwrap();
}

pub fn matroska_write_cluster_nblock(
    timestamp: u64,
    blocks: &[(SimpleBlock, u64)],
) -> Vec<Vec<u8>> {
    let mut blocksyy = Vec::new();

    let mut latest_buffer: Vec<u8> = Vec::new();

    let mut all_block_len = 0;

    for (b, szsz) in blocks {
        let mut block_inner: Vec<u8> = Vec::new();

        ebml_vint_size(b.track_id, &mut block_inner);
        block_inner
            .write(&b.relative_timestamp.to_be_bytes())
            .unwrap();
        block_inner.write(&[b.flags]).unwrap();
        //block_inner.write(&b.data).unwrap();

        ebml_type(0xA3, &mut latest_buffer);
        ebml_vint_size((block_inner.len() as u64) + szsz, &mut latest_buffer);
        latest_buffer.write(&block_inner).unwrap();

        all_block_len += latest_buffer.len() as u64 + *szsz as u64;

        let mut push = Vec::new();
        push.append(&mut latest_buffer);
        latest_buffer.clear();
        blocksyy.push(Some(push));
    }

    let mut tmstp = Vec::new();

    //Timestamp
    ebml_write_u64thingy(0xE7, timestamp, &mut tmstp);

    let mut final_buffer = Vec::new();

    //Cluster
    ebml_type(0x1F43B675, &mut final_buffer);
    ebml_vint_size(tmstp.len() as u64 + all_block_len as u64, &mut final_buffer);
    final_buffer.write(&tmstp).unwrap();
    final_buffer.write(&blocksyy[0].take().unwrap()).unwrap();

    let mut ret = Vec::new();
    ret.push(final_buffer);
    for f in 1..blocksyy.len() {
        ret.push(blocksyy[f].take().unwrap());
    }
    ret
}

pub fn matroska_write_cue_track_pos_inner(
    track: u64,
    cluster: u64,
    // relative: u64,
    veca: &mut impl Write,
) {
    ebml_write_u64thingy(0xF7, track, veca);
    ebml_write_u64thingy(0xF1, cluster, veca);
    //   ebml_write_u64thingy(0xF0, relative, veca);
}

pub struct Cue {
    pub time: u64,
    pub track: u64,
    pub cluster: u64,
    pub relative: Option<u64>,
}

pub fn matroska_write_cue_point_inner(cue: &Cue, veca: &mut impl Write) {
    ebml_write_u64thingy(0xB3, cue.time, veca);
    let mut tpos = Vec::new();

    matroska_write_cue_track_pos_inner(cue.track, cue.cluster /*,cue.relative*/, &mut tpos);

    ebml_write_binary(0xB7, &tpos, veca)
}

pub fn matroska_write_cues(cue: &Vec<Cue>, veca: &mut impl Write) {
    let mut inn = Vec::new();
    let mut innz = Vec::new();

    //rough estimate
    inn.reserve(cue.len() * (16 + 16 + 16 + 16 + 8));
    for f in cue {
        matroska_write_cue_point_inner(f, &mut innz);
        ebml_write_binary(0xBB, &innz, &mut inn);
        innz.clear();
    }
    ebml_write_binary(0x1C53BB6B, &inn, veca);
}

pub struct MatroksChapter {
    pub title: Option<String>,
    pub time_start: u64,
}
fn matroksa_write_chapter_display_inner(stra: &str, veca: &mut impl Write) {
    //ChapString
    ebml_write_utf8(0x85, stra, veca);
    //ChapLanguage
    ebml_write_utf8(0x437C, "eng", veca);
    //ChapLanguageIETF
    ebml_write_utf8(0x437D, "en", veca);
}

fn matroksa_write_chapter_atom(cha: &MatroksChapter, uid: u64, veca: &mut impl Write) {
    //ChapterUID
    ebml_write_u64thingy(0x73C4, 634643 + uid, veca);
    //ChapterTimeStart
    ebml_write_u64thingy(0x91, cha.time_start, veca);
    //ChapterFlagHidden
    ebml_write_u8(0x98, 0, veca);
    //ChapterFlagEnabled
    ebml_write_u8(0x4598, 0, veca);

    let te = if let Some(te) = &cha.title { te } else { "" };

    {
        let mut chpater_display = Vec::new();
        matroksa_write_chapter_display_inner(te, &mut chpater_display);
        //ChapterDisplay
        ebml_write_binary(0x80, &chpater_display, veca)
    }
}

fn matroksa_write_chapter_edition_inner(cha: &[MatroksChapter], veca: &mut impl Write) {
    //EditionFlagHidden
    ebml_write_u8(0x45BD, 0, veca);
    //EditionFlagDefault
    ebml_write_u8(0x45DB, 0, veca);
    //EditionUID
    ebml_write_u64thingy(0x45BC, 0x782732737, veca);
    for (i, f) in cha.iter().enumerate() {
        let mut inna = Vec::new();
        matroksa_write_chapter_atom(f, (88 * i) as u64, &mut inna);
        //ChapterAtom
        ebml_write_binary(0xB6, &mut inna, veca);
    }
}
fn matroksa_write_chapter_edition(cha: &[MatroksChapter], veca: &mut impl Write) {
    let mut ed_entry = Vec::new();
    matroksa_write_chapter_edition_inner(cha, &mut ed_entry);
    ebml_write_binary(0x45B9, &ed_entry, veca)
}
pub fn matroksa_write_chapters(cha: &[MatroksChapter], veca: &mut impl Write) {
    let mut asdasd = Vec::new();
    matroksa_write_chapter_edition(cha, &mut asdasd);
    ebml_write_binary(0x1043A770, &asdasd, veca)
}

#[test]
fn asd() {
    let blocks = vec![
        SimpleBlock {
            track_id: 3,
            relative_timestamp: 1,
            flags: 0,
            data: vec![0; 123],
        },
        SimpleBlock {
            track_id: 1,
            relative_timestamp: 1,
            flags: 0,
            data: vec![0; 456],
        },
        SimpleBlock {
            track_id: 1,
            relative_timestamp: 1,
            flags: 0,
            data: vec![0; 4456],
        },
    ];
    let mut veca = Vec::new();
    matroska_write_cluster(0, &blocks, &mut veca);
    let rmp: Vec<(u64, usize)> = blocks.iter().map(|e| (e.track_id, e.data.len())).collect();
    assert_eq!(veca.len() as u64, matroska_predict_cluster_size(&rmp));
}
