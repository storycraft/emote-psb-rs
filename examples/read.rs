use core::{error::Error, pin::pin};
use emote_psb::{
    psb::PsbFile,
    value::io::{
        error::PsbValueReadError,
        read::{PsbStreamValue, PsbStreamValueReader},
    },
};
use futures_util::TryStreamExt;
use std::{io::SeekFrom, time::Instant};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncSeek, AsyncSeekExt, BufReader},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut file = BufReader::new(File::open("01_com_001_01.ks.scn").await?);
    let psb = PsbFile::open(&mut file).await?;
    dbg!(&psb);

    file.seek(SeekFrom::Start(psb.entrypoint as _)).await?;
    let mut reader = PsbStreamValueReader::new(&mut file);
    let start = Instant::now();
    read_value(&mut reader, &mut vec![]).await?;
    println!("Elapsed: {} ms", start.elapsed().as_millis());

    Ok(())
}

async fn read_value(
    reader: &mut PsbStreamValueReader<impl AsyncRead + AsyncSeek + Unpin>,
    buf: &mut Vec<u64>,
) -> Result<(), PsbValueReadError> {
    match reader.next().await? {
        PsbStreamValue::Primitive(v) => {
            println!("value: {v:?}")
        }
        PsbStreamValue::UintArray {
            len,
            item_byte_size,
        } => {
            let mut stream = pin!(reader.next_uint_array(item_byte_size, len));
            let buf_start = buf.len();
            while let Some(v) = stream.try_next().await? {
                buf.push(v);
            }
            println!("UintArray({len}): {:?}", &buf[buf_start..]);
            buf.drain(buf_start..);
        }
        PsbStreamValue::List => {
            let mut access = reader.next_list().await?;
            Box::pin(async move {
                println!("List start:");
                while let Some(value_reader) = access.next().await? {
                    read_value(value_reader, buf).await?;
                }
                println!("List end");

                Ok::<_, PsbValueReadError>(())
            })
            .await?;
        }
        PsbStreamValue::Object => {
            let mut access = reader.next_object().await?;
            Box::pin(async move {
                println!("Object start:");
                while let Some((name, value_reader)) = access.next().await? {
                    print!("name: {name:?} ");
                    read_value(value_reader, buf).await?;
                }
                println!("Object end");

                Ok::<_, PsbValueReadError>(())
            })
            .await?;
        }
    }

    Ok(())
}
