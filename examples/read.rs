use core::error::Error;
use emote_psb::{
    psb::{read::PsbFile, write::PsbWriter},
    value::PsbValue,
};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = BufReader::new(File::open("sample.psb")?);
    let mut psb = PsbFile::open(&mut file)?;
    dbg!(&psb);

    let root = psb.deserialize_root::<PsbValue>()?;

    let mut out = BufWriter::new(File::create("test.psb")?);
    let writer = PsbWriter::new(psb.version, psb.encrypted, &root, &mut out)?;
    writer.finish()?;

    let mut file2 = BufReader::new(File::open("test.psb")?);
    let mut psb2 = PsbFile::open(&mut file2)?;
    dbg!(&psb2);
    assert_eq!(psb2.deserialize_root::<PsbValue>()?, root);

    Ok(())
}
