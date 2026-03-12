use std::io::{self, Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::value::{PSB_TYPE_INTEGER_ARRAY_N, de};

pub fn read_uint_array(stream: &mut impl Read, buf: &mut Vec<u64>) -> Result<usize, de::Error> {
    const PSB_TYPE_INTEGER_ARRAY_START: u8 = PSB_TYPE_INTEGER_ARRAY_N + 1;
    const PSB_TYPE_INTEGER_ARRAY_END: u8 = PSB_TYPE_INTEGER_ARRAY_N + 8;

    let len_n = match stream.read_u8()? {
        ty @ PSB_TYPE_INTEGER_ARRAY_START..=PSB_TYPE_INTEGER_ARRAY_END => {
            ty - PSB_TYPE_INTEGER_ARRAY_N
        }
        ty => return Err(de::Error::InvalidValueType(ty)),
    };

    let len = read_partial_uint(stream, len_n)?;
    let item_byte_size = stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N;
    buf.reserve(len as _);
    for _ in 0..len {
        buf.push(read_partial_uint(stream, item_byte_size)?);
    }
    Ok(len as _)
}

pub fn write_uint_array(stream: &mut impl Write, buf: &[impl Into<u64> + Copy]) -> io::Result<()> {
    let len_n = get_uint_n(buf.len() as _);
    stream.write_u8(PSB_TYPE_INTEGER_ARRAY_N + len_n)?;
    stream.write_all(&buf.len().to_le_bytes()[..len_n as _])?;

    let max_v = buf
        .iter()
        .copied()
        .map(Into::into)
        .max()
        .unwrap_or_default();
    let max_n = get_uint_n(max_v);
    stream.write_u8(PSB_TYPE_INTEGER_ARRAY_N + max_n)?;
    for v in buf.iter().copied().map(Into::into) {
        stream.write_all(&v.to_le_bytes()[..max_n as _])?;
    }

    Ok(())
}

pub fn read_partial_uint(stream: &mut impl Read, size: u8) -> io::Result<u64> {
    match size {
        0 => Ok(0),
        1..=8 => {
            let mut buf = [0_u8; 8];
            stream.read_exact(&mut buf[..size as usize])?;

            Ok(u64::from_le_bytes(buf))
        }

        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    }
}

pub fn read_partial_int(stream: &mut impl Read, size: u8) -> io::Result<i64> {
    match size {
        0 => Ok(0),
        size @ 1..=8 => {
            let mut buf = [0_u8; 8];
            let len = size as usize;
            stream.read_exact(&mut buf[..len])?;
            if buf[len - 1] > 0x7f {
                buf[len..].fill(0xff);
            }

            Ok(i64::from_le_bytes(buf))
        }

        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    }
}

pub const fn get_n(v: i64) -> u8 {
    get_uint_n(u64::from_ne_bytes(v.to_ne_bytes()) << 1)
}

pub const fn get_uint_n(v: u64) -> u8 {
    if v <= 0xff {
        1
    } else if v <= 0xffff {
        2
    } else if v <= 0xffffff {
        3
    } else if v <= 0xffffffff {
        4
    } else if v <= 0xffffffffff {
        5
    } else if v <= 0xffffffffffff {
        6
    } else if v <= 0xffffffffffffff {
        7
    } else {
        8
    }
}
