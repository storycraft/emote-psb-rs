//! MDF (compressed PSB) reading and writing support.

pub mod error;

use std::io::{self, BufRead, Read, Seek, SeekFrom, Take, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::{Compression, bufread::ZlibDecoder, write::ZlibEncoder};

use crate::{
    PSB_MDF_SIGNATURE,
    mdf::error::{MdfCreateError, MdfOpenError},
};

/// A streaming reader for MDF (zlib-compressed PSB) files.
///
/// MDF files consist of an 8-byte header (signature + compressed-data length)
/// followed by the zlib-compressed PSB data. [`MdfReader`] transparently
/// decompresses the data as it is read.
///
/// # Example
///
/// ```no_run
/// use emote_psb::mdf::MdfReader;
/// use std::{fs::File, io::{BufReader, Read}};
///
/// let file = BufReader::new(File::open("sample.mdf").unwrap());
/// let mut reader = MdfReader::open(file).unwrap();
/// let mut buf = Vec::new();
/// reader.read_to_end(&mut buf).unwrap();
/// ```
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

    /// Returns total size of mdf data stream
    #[inline]
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl<T: BufRead> Read for MdfReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

/// A streaming writer for MDF (zlib-compressed PSB) files.
///
/// Write the PSB data to this writer as if it were a normal [`Write`] sink; call
/// [`finish`] when done to flush the zlib stream and back-fill the compressed-data
/// length field in the header.
///
/// [`finish`]: MdfWriter::finish
pub struct MdfWriter<T: Write> {
    inner: ZlibEncoder<T>,
    stream_start: u64,
}

impl<T: Write + Seek> MdfWriter<T> {
    /// Creates a new [`MdfWriter`], writing the MDF header to `stream`.
    ///
    /// - `stream` — writable, seekable output stream.
    /// - `level` — zlib compression level (0 = no compression, 9 = maximum).
    ///
    /// # Errors
    ///
    /// Returns [`MdfCreateError`] if writing the header fails.
    pub fn new(mut stream: T, level: u8) -> Result<Self, MdfCreateError> {
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

impl<T: Write> Write for MdfWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
