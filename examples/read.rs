use core::error::Error;
use emote_psb::{psb::read::PsbFile, value::PsbValue};
use std::{fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = BufReader::new(File::open("sample.psb")?);
    let mut psb = PsbFile::open(&mut file)?;
    dbg!(&psb);
    println!("{:?}", psb.deserialize_root::<PsbValue>()?);

    Ok(())
}
