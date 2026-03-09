use core::error::Error;
use emote_psb::{psb::read::PsbFile, value::PsbValue};
use std::{fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = BufReader::new(File::open("01_com_001_01.ks.scn")?);
    let mut psb = PsbFile::open(&mut file)?;
    println!("{:?}", psb.deserialize_root::<PsbValue>()?);

    Ok(())
}

