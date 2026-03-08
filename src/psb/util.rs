use std::io::Read;

use crate::value::io::{
    error::PsbValueReadError,
    read::{PsbStreamValue, PsbStreamValueReader},
};

pub fn read_uint_array(
    reader: &mut PsbStreamValueReader<impl Read>,
    array: &mut impl Extend<u64>,
) -> Result<(), PsbValueReadError> {
    let PsbStreamValue::UintArray(len) = reader.read_next()? else {
        return Err(PsbValueReadError::InvalidValue);
    };

    reader.next_uint_array(len, array)?;
    Ok(())
}
