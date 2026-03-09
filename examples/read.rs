use core::error::Error;
use emote_psb::psb::read::PsbFile;
use std::{fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = BufReader::new(File::open("01_com_001_01.ks.scn")?);
    let mut psb = PsbFile::open(&mut file)?;
    dbg!(&psb);

    dbg!(psb.deserialize_root::<Test>());

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct Test {
    hash: String,
}
