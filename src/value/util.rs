use std::io::{self, Read, Write};

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
    Ok(i64::from_ne_bytes(
        read_partial_uint(stream, size)?.to_ne_bytes(),
    ))
}

pub fn write_partial_uint(stream: &mut impl Write, v: u64, size: u8) -> io::Result<()> {
    match size {
        0 => Ok(()),
        1..=8 => {
            stream.write_all(&v.to_le_bytes()[..size as usize])?;
            Ok(())
        }

        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    }
}

pub fn get_n(mut number: i64) -> u8 {
    if number < 0 {
        number = -number;
    }

    if number <= 0x7f {
        1
    } else if number <= 0x7fff {
        2
    } else if number <= 0x7fffff {
        3
    } else if number <= 0x7fffffff {
        4
    } else if number <= 0x7fffffffff {
        5
    } else if number <= 0x7fffffffffff {
        6
    } else if number <= 0x7fffffffffffff {
        7
    } else {
        8
    }
}

pub fn get_uint_n(number: u64) -> u8 {
    if number <= 0xff {
        1
    } else if number <= 0xffff {
        2
    } else if number <= 0xffffff {
        3
    } else if number <= 0xffffffff {
        4
    } else if number <= 0xffffffffff {
        5
    } else if number <= 0xffffffffffff {
        6
    } else if number <= 0xffffffffffffff {
        7
    } else {
        8
    }
}
