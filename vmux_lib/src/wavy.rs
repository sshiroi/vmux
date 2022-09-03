//use simple_error::*;
use crate::*;
use std::error::Error;

pub struct WavProps {
    pub data_format: u16,

    pub num_samples: u64,
    pub sample_rate: u64,

    pub bytes_per_singular_sample: u64,
    pub bytes_per_sample: u64,

    pub first_header_size: u64,
    pub bytes_per_second: u64,
    pub fmt_header_size: u64,
    pub smaple_data_size: u64,
    pub total_file_size: u64,
}

pub fn collect_wav_props(ass: &ASource) -> WavProps {
    let data_format = {
        let frmt = match ass.format {
            AFormat::INT => 1,
            AFormat::FLOAT => 3,
            //     _ => panic!("invalid format"),
        };
        frmt
    };

    let bytes_per_singular_sample = f32::floor((ass.bits_per_sample as f32 + 7.0) / 8.0) as u64;
    let bytes_per_sample: u64 = bytes_per_singular_sample * ass.channels as u64;

    let bytes_per_second = bytes_per_sample * ass.sample_rate as u64;

    let first_header_size = 12;
    let fmt_header_size = 24;

    let num_samples = ass.num_samples as u64 - 1;

    let smaple_data_size = bytes_per_sample * num_samples;

    let total_file_size = first_header_size + fmt_header_size + 8 + smaple_data_size;

    WavProps {
        sample_rate: ass.sample_rate as u64,
        data_format,
        bytes_per_singular_sample,
        bytes_per_sample,
        num_samples,
        first_header_size,
        fmt_header_size,
        smaple_data_size,
        total_file_size,
        bytes_per_second,
    }
}

pub fn custom_wav_header(ass: &ASource, inter: &WavProps) -> Result<Vec<u8>, Box<dyn Error>> {
    use byteorder::*;
    let mut veca = Vec::new();

    {
        veca.write_u8('R' as u8)?;
        veca.write_u8('I' as u8)?;
        veca.write_u8('F' as u8)?;
        veca.write_u8('F' as u8)?;
        veca.write_u32::<LittleEndian>((inter.total_file_size - 8) as u32)?;
        veca.write_u8('W' as u8)?;
        veca.write_u8('A' as u8)?;
        veca.write_u8('V' as u8)?;
        veca.write_u8('E' as u8)?;
    }
    {
        veca.write_u8('f' as u8)?;
        veca.write_u8('m' as u8)?;
        veca.write_u8('t' as u8)?;
        veca.write_u8(' ' as u8)?;
        veca.write_u32::<LittleEndian>(16)?;
        veca.write_u16::<LittleEndian>(inter.data_format)?;
        veca.write_u16::<LittleEndian>(ass.channels as u16)?;
        veca.write_u32::<LittleEndian>(ass.sample_rate as u32)?;
        veca.write_u32::<LittleEndian>(inter.bytes_per_second as u32)?;
        veca.write_u16::<LittleEndian>(inter.bytes_per_sample as u16)?;
        veca.write_u16::<LittleEndian>(ass.bits_per_sample as u16)?;
    }
    {
        veca.write_u8('d' as u8)?;
        veca.write_u8('a' as u8)?;
        veca.write_u8('t' as u8)?;
        veca.write_u8('a' as u8)?;
        veca.write_u32::<LittleEndian>((inter.total_file_size - 44) as u32)?;
    }
    Ok(veca)
}
