use std::io::{self, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::value::{
    PsbPrimitive,
    io::{
        PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
        PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
        PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0,
        PSB_TYPE_INTEGER_ARRAY_N, PSB_TYPE_INTEGER_N, PSB_TYPE_LIST, PSB_TYPE_NONE, PSB_TYPE_NULL,
        PSB_TYPE_OBJECT, PSB_TYPE_RESOURCE_N, PSB_TYPE_STRING_N, PSB_TYPE_TRUE,
        error::PsbValueReadError,
    },
    number::PsbNumber,
    util::{read_partial_int, read_partial_uint},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PsbStreamValue {
    Primitive(PsbPrimitive),
    UintArray(usize),
    List,
    Object,
}

#[derive(Debug)]
pub struct PsbStreamValueReader<T> {
    stream: T,
    buf: Vec<u64>,
}

impl<T: Read> PsbStreamValueReader<T> {
    pub const fn new(stream: T) -> Self {
        Self {
            stream,
            buf: vec![],
        }
    }

    pub fn read_next(&mut self) -> Result<PsbStreamValue, PsbValueReadError> {
        const PSB_TYPE_INTEGER_ARRAY_START: u8 = PSB_TYPE_INTEGER_ARRAY_N + 1;
        const PSB_TYPE_INTEGER_ARRAY_MAX: u8 = PSB_TYPE_INTEGER_ARRAY_N + 8;

        let value_type = self.stream.read_u8()?;
        if let Some(primitive) = self.read_primitive(value_type)? {
            return Ok(PsbStreamValue::Primitive(primitive));
        }

        match value_type {
            value_type @ PSB_TYPE_INTEGER_ARRAY_START..=PSB_TYPE_INTEGER_ARRAY_MAX => {
                let len =
                    read_partial_uint(&mut self.stream, value_type - PSB_TYPE_INTEGER_ARRAY_N)?;
                Ok(PsbStreamValue::UintArray(len as usize))
            }

            PSB_TYPE_LIST => Ok(PsbStreamValue::List),
            PSB_TYPE_OBJECT => Ok(PsbStreamValue::Object),

            value_type => Err(PsbValueReadError::InvalidValueType(value_type)),
        }
    }

    /// Read uint array
    pub fn next_uint_array(&mut self, len: usize, buf: &mut impl Extend<u64>) -> io::Result<()> {
        let item_byte_size = self.stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N;
        for _ in 0..len {
            buf.extend(read_partial_uint(&mut self.stream, item_byte_size));
        }

        Ok(())
    }

    // /// Read stream of items in list
    // pub async fn next_list<'a>(&'a mut self) -> Result<PsbListAccess<'a, T>, PsbValueReadError>
    // where
    //     T: AsyncSeek,
    // {
    //     let offsets = self.read_uint_array_buf()?;
    //     let data_start = self.stream.stream_position()?;

    //     Ok(PsbListAccess {
    //         buf_reader: PsbBufReader::new(self, offsets.start),
    //         offsets,
    //         data_start,
    //     })
    // }

    // /// Read streams of items in object
    // pub async fn next_object<'a>(&'a mut self) -> Result<PsbObjectAccess<'a, T>, PsbValueReadError>
    // where
    //     T: AsyncSeek,
    // {
    //     let names = self.read_uint_array_buf()?;
    //     let offsets = self.read_uint_array_buf()?;

    //     let data_start = self.stream.stream_position()?;
    //     Ok(PsbObjectAccess {
    //         buf_reader: PsbBufReader::new(self, names.start),
    //         names,
    //         offsets,
    //         data_start,
    //     })
    // }

    fn read_primitive(
        &mut self,
        value_type: u8,
    ) -> Result<Option<PsbPrimitive>, PsbValueReadError> {
        const PSB_TYPE_INTEGER_START: u8 = PSB_TYPE_INTEGER_N;
        const PSB_TYPE_INTEGER_MAX: u8 = PSB_TYPE_INTEGER_N + 8;
        const PSB_TYPE_RESOURCE_START: u8 = PSB_TYPE_RESOURCE_N + 1;
        const PSB_TYPE_RESOURCE_MAX: u8 = PSB_TYPE_RESOURCE_N + 4;
        const PSB_TYPE_STRING_START: u8 = PSB_TYPE_STRING_N + 1;
        const PSB_TYPE_STRING_MAX: u8 = PSB_TYPE_STRING_N + 4;
        const PSB_TYPE_EXTRA_START: u8 = PSB_TYPE_EXTRA_N + 1;
        const PSB_TYPE_EXTRA_MAX: u8 = PSB_TYPE_EXTRA_N + 4;

        match value_type {
            PSB_TYPE_NONE => Ok(Some(PsbPrimitive::None)),
            PSB_TYPE_NULL => Ok(Some(PsbPrimitive::Null)),

            PSB_TYPE_FALSE => Ok(Some(PsbPrimitive::Bool(false))),
            PSB_TYPE_TRUE => Ok(Some(PsbPrimitive::Bool(true))),

            PSB_TYPE_DOUBLE => Ok(Some(PsbPrimitive::Number(PsbNumber::Double(
                self.stream.read_f64::<LittleEndian>()?,
            )))),
            PSB_TYPE_FLOAT0 => Ok(Some(PsbPrimitive::Number(PsbNumber::Float(0.0)))),
            PSB_TYPE_FLOAT => Ok(Some(PsbPrimitive::Number(PsbNumber::Float(
                self.stream.read_f32::<LittleEndian>()?,
            )))),

            value_type @ PSB_TYPE_INTEGER_START..=PSB_TYPE_INTEGER_MAX => {
                Ok(Some(PsbPrimitive::Number(PsbNumber::Integer(
                    read_partial_int(&mut self.stream, value_type - PSB_TYPE_INTEGER_N)?,
                ))))
            }

            value_type @ PSB_TYPE_RESOURCE_START..=PSB_TYPE_RESOURCE_MAX => {
                Ok(Some(PsbPrimitive::Resource(
                    read_partial_uint(&mut self.stream, value_type - PSB_TYPE_RESOURCE_N)?
                        .try_into()
                        .map_err(|_| PsbValueReadError::InvalidValue)?,
                )))
            }

            value_type @ PSB_TYPE_STRING_START..=PSB_TYPE_STRING_MAX => {
                Ok(Some(PsbPrimitive::String(
                    read_partial_uint(&mut self.stream, value_type - PSB_TYPE_STRING_N)?
                        .try_into()
                        .map_err(|_| PsbValueReadError::InvalidValue)?,
                )))
            }

            value_type @ PSB_TYPE_EXTRA_START..=PSB_TYPE_EXTRA_MAX => {
                Ok(Some(PsbPrimitive::ExtraResource(
                    read_partial_uint(&mut self.stream, value_type - PSB_TYPE_EXTRA_N)?
                        .try_into()
                        .map_err(|_| PsbValueReadError::InvalidValue)?,
                )))
            }

            PSB_COMPILER_INTEGER => Ok(Some(PsbPrimitive::CompilerNumber)),
            PSB_COMPILER_STRING => Ok(Some(PsbPrimitive::CompilerString)),
            PSB_COMPILER_RESOURCE => Ok(Some(PsbPrimitive::CompilerResource)),
            PSB_COMPILER_ARRAY => Ok(Some(PsbPrimitive::CompilerArray)),
            PSB_COMPILER_DECIMAL => Ok(Some(PsbPrimitive::CompilerDecimal)),
            PSB_COMPILER_BOOL => Ok(Some(PsbPrimitive::CompilerBool)),
            PSB_COMPILER_BINARY_TREE => Ok(Some(PsbPrimitive::CompilerBinaryTree)),

            _ => Ok(None),
        }
    }
}

#[derive(Debug)]
struct PsbBufReader<'a, T> {
    inner: &'a mut PsbStreamValueReader<T>,
    buf_start: usize,
}

impl<'a, T> PsbBufReader<'a, T> {
    const fn new(inner: &'a mut PsbStreamValueReader<T>, buf_start: usize) -> Self {
        Self { inner, buf_start }
    }
}

impl<'a, T> Drop for PsbBufReader<'a, T> {
    fn drop(&mut self) {
        self.inner.buf.drain(self.buf_start..);
    }
}

// #[derive(Debug)]
// pub struct PsbListAccess<'a, T> {
//     buf_reader: PsbBufReader<'a, T>,
//     offsets: Range<usize>,
//     data_start: u64,
// }

// impl<'a, T> PsbListAccess<'a, T>
// where
//     T: AsyncRead + AsyncSeek + Unpin,
// {
//     pub async fn next(&mut self) -> io::Result<Option<&mut PsbStreamValueReader<T>>> {
//         let Some(offset) = self.offsets.next() else {
//             return Ok(None);
//         };

//         self.buf_reader.inner.stream.seek(SeekFrom::Start(
//             self.data_start + self.buf_reader.inner.buf[offset],
//         ))?;
//         Ok(Some(self.buf_reader.inner))
//     }
// }

// #[derive(Debug)]
// pub struct PsbObjectAccess<'a, T> {
//     buf_reader: PsbBufReader<'a, T>,
//     names: Range<usize>,
//     offsets: Range<usize>,
//     data_start: u64,
// }

// impl<'a, T> PsbObjectAccess<'a, T>
// where
//     T: AsyncRead + AsyncSeek + Unpin,
// {
//     pub async fn next(
//         &mut self,
//     ) -> io::Result<Option<(PsbNameIndex, &mut PsbStreamValueReader<T>)>> {
//         let Some(name) = self.names.next() else {
//             return Ok(None);
//         };
//         let Some(offset) = self.offsets.next() else {
//             return Ok(None);
//         };

//         let index = PsbNameIndex(self.buf_reader.inner.buf[name]);
//         self.buf_reader.inner.stream.seek(SeekFrom::Start(
//             self.data_start + self.buf_reader.inner.buf[offset],
//         ))?;
//         Ok(Some((index, self.buf_reader.inner)))
//     }
// }
