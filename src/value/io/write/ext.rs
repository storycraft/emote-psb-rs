use core::num::NonZeroU8;

use tokio::io::AsyncWrite;

use crate::value::{
    io::{error::PsbValueWriteError, write::PsbStreamValueWriter},
    util::get_uint_n,
};

#[extend::ext(name = PsbValueWriterExt)]
pub impl<T: AsyncWrite + Unpin> PsbStreamValueWriter<T> {
    async fn write_uint_array(&mut self, array: &[u64]) -> Result<(), PsbValueWriteError> {
        let mut entry = self
            .begin_uint_array(
                array.len() as _,
                NonZeroU8::new(array.iter().copied().max().map(get_uint_n).unwrap_or(1)).unwrap(),
            )
            .await?;

        for &v in array {
            entry.write_next(v).await?;
        }
        Ok(())
    }
}
