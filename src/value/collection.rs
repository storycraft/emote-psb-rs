use std::{collections::HashMap, io::SeekFrom};

use crate::value::{error::PsbValueReadError, reference::PsbNameIndex, utill::PsbValueStreamExt};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};

use super::{PsbValue, PSB_TYPE_INTEGER_ARRAY_N};

#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct PsbUintArray(#[from] pub Vec<u64>);

impl PsbUintArray {
    pub(super) async fn read_io(
        stream: &mut (impl AsyncRead + Unpin),
        len: usize,
    ) -> Result<Self, PsbValueReadError> {
        let item_byte_size = stream.read_u8().await? - PSB_TYPE_INTEGER_ARRAY_N;

        let mut list = Vec::<u64>::with_capacity(len);
        for _ in 0..len {
            list.push(stream.read_partial_uint(item_byte_size).await?);
        }

        Ok(PsbUintArray(list))
    }

    // pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
    //     let len = self.vec.len() as u64;

    //     let count_written = PsbNumber::write_uint(self.get_n(), len, stream)? as u64;

    //     let n = self.get_item_n();

    //     stream.write_u8(n + PSB_TYPE_INTEGER_ARRAY_N)?;

    //     for num in self.vec.iter() {
    //         PsbNumber::write_uint(n, *num, stream)?;
    //     }

    //     Ok(1 + count_written + n as u64 * len)
    // }
}

#[derive(Debug, Default, Clone, PartialEq, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct PsbList(#[from] pub Vec<PsbValue>);

impl PsbList {
    pub(super) async fn read_io(
        stream: &mut (impl AsyncRead + AsyncSeek + Unpin),
    ) -> Result<Self, PsbValueReadError> {
        let PsbValue::IntArray(PsbUintArray(offsets)) =
            PsbValue::read_io_non_recursion(stream).await?
        else {
            return Err(PsbValueReadError::InvalidValue);
        };

        if offsets.is_empty() {
            return Ok(Self::default());
        }

        let values = Box::pin(async {
            let start = stream.stream_position().await?;
            let mut values = Vec::<PsbValue>::with_capacity(offsets.len());

            for offset in offsets {
                stream.seek(SeekFrom::Start(start + offset)).await?;
                values.push(PsbValue::read_io(stream).await?);
            }

            Ok::<_, PsbValueReadError>(values)
        })
        .await?;

        Ok(Self(values))
    }

    // pub fn write_bytes(&self, stream: &mut impl Write, table: &PsbRefs) -> Result<u64, PsbError> {
    //     let mut value_offset_cache = HashMap::<u64, &PsbValue>::new();

    //     let mut offsets = Vec::<u64>::new();
    //     let mut data_buffer = Vec::<u8>::new();

    //     let mut total_data_written = 0_u64;
    //     for value in &self.values {
    //         let mut cached = false;
    //         for (offset, cache_value) in &value_offset_cache {
    //             if value == *cache_value {
    //                 offsets.push(*offset);
    //                 cached = true;
    //                 break;
    //             }
    //         }

    //         if !cached {
    //             value_offset_cache.insert(total_data_written, value);
    //             offsets.push(total_data_written);

    //             total_data_written += value.write_bytes_refs(&mut data_buffer, table)?;
    //         }
    //     }

    //     let offset_written = PsbValue::IntArray(PsbUintArray::from(offsets)).write_bytes(stream)?;
    //     stream.write_all(&data_buffer)?;

    //     Ok(offset_written + total_data_written)
    // }
}

#[derive(Debug, Clone, Default, PartialEq, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct PsbObject(#[from] pub HashMap<PsbNameIndex, PsbValue>);

impl PsbObject {
    pub async fn read_io(
        stream: &mut (impl AsyncRead + AsyncSeek + Unpin),
    ) -> Result<Self, PsbValueReadError> {
        let PsbValue::IntArray(PsbUintArray(names)) =
            PsbValue::read_io_non_recursion(stream).await?
        else {
            return Err(PsbValueReadError::InvalidValue);
        };

        // TODO
        let PsbValue::IntArray(PsbUintArray(offsets)) =
            PsbValue::read_io_non_recursion(stream).await?
        else {
            return Err(PsbValueReadError::InvalidValue);
        };

        if names.is_empty() {
            return Ok(Self::default());
        }

        let map = Box::pin(async {
            let start = stream.stream_position().await?;
            let mut map = HashMap::<PsbNameIndex, PsbValue>::with_capacity(names.len());
            for (name, offset) in names.into_iter().zip(offsets.into_iter()) {
                stream.seek(SeekFrom::Start(start + offset)).await?;
                // TODO:: reduce alloc
                map.insert(PsbNameIndex(name), PsbValue::read_io(stream).await?);
            }

            Ok::<_, PsbValueReadError>(map)
        })
        .await?;

        Ok(Self(map))
    }

    // pub fn write_bytes(
    //     &self,
    //     stream: &mut impl Write,
    //     ref_table: &PsbRefs,
    // ) -> Result<u64, PsbError> {
    //     let mut value_offset_cache = HashMap::<u64, &PsbValue>::new();

    //     let mut ref_cache = HashMap::<&String, u64>::new();

    //     let mut name_refs = Vec::<u64>::new();
    //     let mut offsets = Vec::<u64>::new();
    //     let mut data_buffer = Vec::<u8>::new();

    //     let mut total_data_written = 0_u64;

    //     for name in self.map.keys().sorted() {
    //         let value = self.map.get(name).unwrap();

    //         let name_ref = if ref_cache.contains_key(name) {
    //             *ref_cache.get(name).unwrap()
    //         } else {
    //             match ref_table.find_name_index(name) {
    //                 Some(index) => {
    //                     ref_cache.insert(name, index);

    //                     Ok(index)
    //                 }

    //                 None => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None)),
    //             }?
    //         };

    //         name_refs.push(name_ref);

    //         let mut cached = false;
    //         for (offset, cache_value) in value_offset_cache.iter() {
    //             if value == *cache_value {
    //                 offsets.push(*offset);
    //                 cached = true;
    //                 break;
    //             }
    //         }

    //         if !cached {
    //             value_offset_cache.insert(total_data_written, value);
    //             offsets.push(total_data_written);

    //             total_data_written += value.write_bytes_refs(&mut data_buffer, ref_table)?;
    //         }
    //     }

    //     let names_written =
    //         PsbValue::IntArray(PsbUintArray::from(name_refs)).write_bytes(stream)?;
    //     let offset_written = PsbValue::IntArray(PsbUintArray::from(offsets)).write_bytes(stream)?;

    //     stream.write_all(&data_buffer)?;

    //     Ok(names_written + offset_written + total_data_written)
    // }

    // pub fn collect_names(&self, vec: &mut Vec<String>) {
    //     for (name, child) in self.map.iter() {
    //         match child {
    //             PsbValue::Object(child_obj) => {
    //                 child_obj.collect_names(vec);
    //             }

    //             PsbValue::List(child_list) => {
    //                 child_list.collect_names(vec);
    //             }

    //             _ => {}
    //         }

    //         if !vec.contains(name) {
    //             vec.push(name.clone());
    //         }
    //     }
    // }

    // pub fn collect_strings(&self, vec: &mut Vec<String>) {
    //     for (_, child) in self.map.iter() {
    //         match child {
    //             PsbValue::Object(child_obj) => {
    //                 child_obj.collect_strings(vec);
    //             }

    //             PsbValue::List(child_list) => {
    //                 child_list.collect_strings(vec);
    //             }

    //             PsbValue::String(string) => {
    //                 if !vec.contains(string.string()) {
    //                     vec.push(string.string().clone());
    //                 }
    //             }

    //             _ => {}
    //         }
    //     }
    // }
}
