pub mod error;

use core::{
    pin::Pin,
    task::{Context, Poll},
};
use std::io::{self, SeekFrom};

use async_compression::tokio::{bufread::ZlibDecoder, write::ZlibEncoder};
use pin_project::pin_project;
use tokio::io::{
    AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt, BufReader,
    ReadBuf, Take,
};

use crate::{
    PSB_MDF_SIGNATURE,
    mdf::error::{MdfCreateError, MdfOpenError},
};

#[pin_project]
pub struct MdfReader<T> {
    #[pin]
    inner: ZlibDecoder<BufReader<Take<T>>>,
    size: u32,
}

impl<T: AsyncRead + Unpin> MdfReader<T> {
    /// Open new mdf stream
    pub async fn open(mut stream: T) -> Result<Self, MdfOpenError> {
        let signature = stream.read_u32_le().await?;
        if signature != PSB_MDF_SIGNATURE {
            return Err(MdfOpenError::InvalidSignature);
        }

        let size = stream.read_u32_le().await?;
        Ok(Self {
            inner: ZlibDecoder::new(BufReader::new(stream.take(size as _))),
            size,
        })
    }

    #[inline]
    /// Returns total size of mdf data stream
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for MdfReader<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().inner.poll_read(cx, buf)
    }
}

#[pin_project]
pub struct MdfWriter<T> {
    #[pin]
    inner: ZlibEncoder<T>,
    stream_start: u64,
}

impl<T: AsyncWrite + AsyncSeek + Unpin> MdfWriter<T> {
    pub async fn create(mut stream: T) -> Result<Self, MdfCreateError> {
        // Write header
        stream.write_u32_le(PSB_MDF_SIGNATURE).await?;
        // Fill with zero for now
        stream.write_u32_le(0).await?;
        let stream_start = stream.stream_position().await?;
        Ok(Self {
            inner: ZlibEncoder::new(stream),
            stream_start,
        })
    }

    /// Finish mdf file
    pub async fn finish(self) -> io::Result<T> {
        let mut stream = self.inner.into_inner();
        let end = stream.stream_position().await?;
        stream.seek(SeekFrom::Start(self.stream_start - 4)).await?;
        stream
            .write_u32_le((end - self.stream_start) as u32)
            .await?;
        stream.seek(SeekFrom::Start(end)).await?;

        Ok(stream)
    }
}
