pub mod error;

use std::io::{self, BufRead, Read, Seek, SeekFrom, Take, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::{Compression, bufread::ZlibDecoder, write::ZlibEncoder};

use crate::{
    PSB_MDF_SIGNATURE,
    mdf::error::{MdfCreateError, MdfOpenError},
};

pub struct MdfReader<T> {
    inner: ZlibDecoder<Take<T>>,
    size: u32,
}

impl<T: BufRead> MdfReader<T> {
    /// Open new mdf stream
    pub fn open(mut stream: T) -> Result<Self, MdfOpenError> {
        let signature = stream.read_u32::<LittleEndian>()?;
        if signature != PSB_MDF_SIGNATURE {
            return Err(MdfOpenError::InvalidSignature);
        }

        let size = stream.read_u32::<LittleEndian>()?;
        Ok(Self {
            inner: ZlibDecoder::new(stream.take(size as _)),
            size,
        })
    }

    #[inline]
    /// Returns total size of mdf data stream
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl<T: BufRead> Read for MdfReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

pub struct MdfWriter<T: Write> {
    inner: ZlibEncoder<T>,
    stream_start: u64,
}

impl<T: Write + Seek> MdfWriter<T> {
    pub fn create(mut stream: T, level: u8) -> Result<Self, MdfCreateError> {
        // Write header
        stream.write_u32::<LittleEndian>(PSB_MDF_SIGNATURE)?;
        // Fill with zero for now
        stream.write_u32::<LittleEndian>(0)?;
        let stream_start = stream.stream_position()?;
        Ok(Self {
            inner: ZlibEncoder::new(stream, Compression::new(level as _)),
            stream_start,
        })
    }

    /// Finish mdf file
    pub fn finish(self) -> io::Result<T> {
        let mut stream = self.inner.finish()?;

        let end = stream.stream_position()?;
        stream.seek(SeekFrom::Start(self.stream_start - 4))?;
        stream.write_u32::<LittleEndian>((end - self.stream_start) as u32)?;
        stream.seek(SeekFrom::Start(end))?;
        Ok(stream)
    }
}
