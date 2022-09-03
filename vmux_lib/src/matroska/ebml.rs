pub use std::io::Write;

struct ExtrU64 {
    //    msk0: u64,
    //    msk1: u64,
    //    msk2: u64,
    //    msk3: u64,
    //    msk4: u64,
    //    msk5: u64,
    b0: u8,
    b1: u8,
    b2: u8,
    b3: u8,
    b4: u8,
    b5: u8,
    b6: u8,
    b7: u8,
}
fn extract_u64(a: u64) -> ExtrU64 {
    let msk0: u64 = (1 << 8) - 1;
    let msk1 = msk0 << 8;
    let msk2 = msk1 << 8;
    let msk3 = msk2 << 8;
    let msk4 = msk3 << 8;
    let msk5 = msk4 << 8;
    let msk6 = msk5 << 8;
    let msk7 = msk6 << 8;

    let b0 = (a & msk0) as u8;
    let b1 = ((a & msk1) >> 8) as u8;
    let b2 = ((a & msk2) >> 16) as u8;
    let b3 = ((a & msk3) >> 24) as u8;
    let b4 = ((a & msk4) >> 32) as u8;
    let b5 = ((a & msk5) >> 40) as u8;
    let b6 = ((a & msk6) >> 48) as u8;
    let b7 = ((a & msk7) >> 56) as u8;

    ExtrU64 {
        //   msk0,
        //   msk1,
        //   msk2,
        //   msk3,
        //   msk4,
        //   msk5,
        b0,
        b1,
        b2,
        b3,
        b4,
        b5,
        b6,
        b7,
    }
}

pub fn ebml_predict_vint_size(a: u64) -> usize {
    let mut wrt = Vec::new();
    ebml_vint_size(a, &mut wrt);
    wrt.len()
}

pub fn ebml_vint_size(a: u64, wrt: &mut impl Write) {
    /*
    let msk0: u64 = (1 << 8) - 1;
    let msk1 = msk0 << 8;
    let msk2 = msk1 << 8;
    let msk3 = msk2 << 8;
    let msk4 = msk3 << 8;
    let msk5 = msk4 << 8;

    let b0 = (a & msk0) as u8;
    let b1 = ((a & msk1) >> 8) as u8;
    let b2 = ((a & msk2) >> 16) as u8;
    let b3 = ((a & msk3) >> 24) as u8;
    let b4 = ((a & msk4) >> 32) as u8;
    let b5 = ((a & msk5) >> 40) as u8;
    */
    let aaa = a.to_le_bytes();
    let (b0, b1, b2, b3, b4, b5, _b6, _b7) = (
        aaa[0], aaa[1], aaa[2], aaa[3], aaa[4], aaa[5], aaa[6], aaa[7],
    );

    if
    /*   */
    a < (1 << 7) {
        let frst_byte_msk = !0b10000000u8;
        wrt.write(&[0b10000000 | (b0 & frst_byte_msk)]).unwrap();
    } else if a < (1 << 13) {
        let frst_byte_msk = !0b11000000u8;
        wrt.write(&[0b01000000 | (b1 & frst_byte_msk), b0]).unwrap();
    } else if a < (1 << 20) {
        let frst_byte_msk = !0b11100000u8;
        wrt.write(&[0b00100000 | (b2 & frst_byte_msk), b1, b0])
            .unwrap();
    } else if a < (1 << 27) {
        let frst_byte_msk = !0b11110000u8;
        wrt.write(&[0b00010000 | (b3 & frst_byte_msk), b2, b1, b0])
            .unwrap();
    } else if a < (1 << 34) {
        let frst_byte_msk = !0b11111000u8;
        wrt.write(&[0b00001000 | (b4 & frst_byte_msk), b3, b2, b1, b0])
            .unwrap();
    } else if a < (1 << 41) {
        let frst_byte_msk = !0b11111100u8;
        wrt.write(&[0b00000100 | (b5 & frst_byte_msk), b4, b3, b2, b1, b0])
            .unwrap();
    } else {
        panic!("toobig");
    }
}

pub fn ebml_type(a: u64, wrt: &mut impl Write) {
    let msk0: u64 = (1 << 8) - 1;
    let msk1 = msk0 << 8;
    let msk2 = msk1 << 8;
    let msk3 = msk2 << 8;
    let msk4 = msk3 << 8;
    let msk5 = msk4 << 8;

    let b0 = (a & msk0) as u8;
    let b1 = ((a & msk1) >> 8) as u8;
    let b2 = ((a & msk2) >> 16) as u8;
    let b3 = ((a & msk3) >> 24) as u8;
    let b4 = ((a & msk4) >> 32) as u8;
    let b5 = ((a & msk5) >> 40) as u8;

    if a < (1 << 8) {
        wrt.write(&[b0]).unwrap();
    } else if a < (1 << 16) {
        wrt.write(&[b1, b0]).unwrap();
    } else if a < (1 << 24) {
        wrt.write(&[b2, b1, b0]).unwrap();
    } else if a < (1 << 32) {
        wrt.write(&[b3, b2, b1, b0]).unwrap();
    } else if a < (1 << 40) {
        wrt.write(&[b4, b3, b2, b1, b0]).unwrap();
    } else if a < (1 << 48) {
        wrt.write(&[b5, b4, b3, b2, b1, b0]).unwrap();
    } else {
        panic!("toobig");
    }
}

pub fn ebml_write_u8(t: u64, dataa: u8, veca: &mut impl Write) {
    //EBML readversion
    ebml_type(t, veca);
    ebml_vint_size(1, veca);
    veca.write(&[dataa]).unwrap();
}
pub fn ebml_write_u64thingy(t: u64, dataa: u64, veca: &mut impl Write) {
    //EBML readversion
    ebml_type(t, veca);
    ebml_vint_size(8, veca);

    let extr = extract_u64(dataa);
    veca.write(&[
        extr.b7, extr.b6, extr.b5, extr.b4, extr.b3, extr.b2, extr.b1, extr.b0,
    ])
    .unwrap();
}
pub fn ebml_write_utf8(t: u64, dataa: &str, veca: &mut impl Write) {
    let str_bytes: Vec<u8> = dataa.bytes().collect();
    //EBML readversion
    ebml_type(t, veca);
    ebml_vint_size(str_bytes.len() as _, veca);

    veca.write(&str_bytes).unwrap();
}
pub fn ebml_write_f64(t: u64, dataa: f64, veca: &mut impl Write) {
    ebml_write_binary(t, &dataa.to_be_bytes(), veca);
}

pub fn ebml_write_ascii(t: u64, dataa: &str, veca: &mut impl Write) {
    //yolo
    ebml_write_utf8(t, dataa, veca);
}

pub fn ebml_write_binary(t: u64, dataa: &[u8], veca: &mut impl Write) {
    //EBML readversion
    ebml_type(t, veca);
    ebml_vint_size(dataa.len() as _, veca);

    veca.write(&dataa).unwrap();
}

pub fn write_ebml_head(veca: &mut impl Write) {
    let mut header_inner: Vec<u8> = Vec::new();
    //EBML version
    ebml_write_u8(0x4286, 1, &mut header_inner);
    //EBML readversion
    ebml_write_u8(0x42F7, 1, &mut header_inner);

    //EBML EBMLMaxIDLength
    ebml_write_u8(0x42F2, 4, &mut header_inner);

    //EBML EBMLMaxSizeLength
    ebml_write_u8(0x42F3, 8, &mut header_inner);

    //EBML doctype
    ebml_write_utf8(0x4282, "matroska", &mut header_inner);

    //EBML DocTypeVersion
    ebml_write_u8(0x4287, 4, &mut header_inner);
    //EBML DocTypeReadVersion
    ebml_write_u8(0x4285, 2, &mut header_inner);

    //EBML header
    ebml_write_binary(0x1a45dfa3, &header_inner, veca)
}
