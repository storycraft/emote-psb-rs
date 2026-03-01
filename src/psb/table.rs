use std::io::SeekFrom;

use tokio::io::{AsyncRead, AsyncSeek, AsyncSeekExt};

use crate::value::io::{
    error::PsbValueReadError,
    read::{PsbStreamValueReader, ext::PsbValueReaderExt},
};

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct PsbStringItem {
    pub position: u64,
}

#[derive(Debug, Clone)]
pub struct PsbStringTable {
    pub items: Vec<PsbStringItem>,
}

impl PsbStringTable {
    pub(crate) async fn read_io(
        stream: &mut (impl AsyncRead + AsyncSeek + Unpin),
        offset_pos: u32,
        data_pos: u32,
    ) -> Result<Self, PsbValueReadError> {
        stream.seek(SeekFrom::Start(offset_pos as _)).await?;
        let mut offsets = vec![];
        PsbStreamValueReader::new(stream)
            .read_uint_array(&mut offsets)
            .await?;

        Ok(Self {
            items: offsets
                .into_iter()
                .map(|offset| PsbStringItem {
                    position: offset + data_pos as u64,
                })
                .collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct PsbResourceItem {
    pub position: u64,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct PsbResourceTable {
    pub items: Vec<PsbResourceItem>,
}

impl PsbResourceTable {
    pub(crate) async fn read_io(
        reader: &mut (impl AsyncRead + AsyncSeek + Unpin),
        offset_pos: u32,
        lengths_pos: u32,
        data_pos: u32,
    ) -> Result<Self, PsbValueReadError> {
        reader.seek(SeekFrom::Start(offset_pos as _)).await?;
        let mut offsets = vec![];
        PsbStreamValueReader::new(&mut *reader)
            .read_uint_array(&mut offsets)
            .await?;

        reader.seek(SeekFrom::Start(lengths_pos as _)).await?;
        let mut lengths = vec![];
        PsbStreamValueReader::new(&mut *reader)
            .read_uint_array(&mut lengths)
            .await?;

        Ok(Self {
            items: offsets
                .into_iter()
                .zip(lengths.into_iter())
                .map(|(offset, size)| PsbResourceItem {
                    position: offset + data_pos as u64,
                    size,
                })
                .collect::<Vec<_>>(),
        })
    }
}
