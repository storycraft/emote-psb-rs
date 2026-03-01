use core::pin::pin;

use futures_util::TryStreamExt;
use tokio::io::AsyncRead;

use crate::value::io::{
    error::PsbValueReadError,
    read::{PsbStreamValue, PsbStreamValueReader},
};

#[extend::ext(name = PsbValueReaderExt)]
pub impl<T: AsyncRead + Unpin> PsbStreamValueReader<T> {
    async fn read_uint_array(&mut self, array: &mut Vec<u64>) -> Result<(), PsbValueReadError> {
        let PsbStreamValue::UintArray {
            item_byte_size,
            len,
        } = self.next().await?
        else {
            return Err(PsbValueReadError::InvalidValue);
        };

        let mut stream = pin!(self.next_uint_array(item_byte_size, len));
        while let Some(v) = stream.try_next().await? {
            array.push(v);
        }
        Ok(())
    }
}
