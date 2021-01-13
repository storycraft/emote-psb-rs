/*
 * Created on Wed Jan 13 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::File};

    use emote_psb::{PsbReader, PsbRefs, PsbWriter, types::{PsbValue, collection::{PsbList, PsbObject, PsbUintArray}, number::PsbNumber, reference::PsbReference}};

    #[test]
    fn int_write() {
        let mut buffer = Vec::new();
        
        let written = PsbValue::Number(PsbNumber::Integer(-12322)).write_bytes(&mut buffer).unwrap();
    
        println!("written: {} buffer: {:?}", written, buffer);
    }

    #[test]
    fn float_write() {
        let mut buffer = Vec::new();
        
        let written = PsbValue::Number(PsbNumber::Float(122_f32)).write_bytes(&mut buffer).unwrap();
    
        println!("written: {} buffer: {:?}", written, buffer);
    }

    #[test]
    fn float0_write() {
        let mut buffer = Vec::new();
        
        let written = PsbValue::Number(PsbNumber::Float(0f32)).write_bytes(&mut buffer).unwrap();
    
        println!("written: {} buffer: {:?}", written, buffer);
    }

    #[test]
    fn double_write() {
        let mut buffer = Vec::new();
        
        let written = PsbValue::Number(PsbNumber::Double(122_f64)).write_bytes(&mut buffer).unwrap();
    
        println!("written: {} buffer: {:?}", written, buffer);
    }
    
    #[test]
    fn uint_array_write() {
        let mut buffer = Vec::new();
        
        let written = PsbValue::IntArray(
            PsbUintArray::from(vec![123, 12, 122])
        ).write_bytes(&mut buffer).unwrap();
    
        println!("written: {} buffer: {:?}", written, buffer);
    }

    #[test]
    fn list_write() {
        let mut buffer = Vec::new();
        
        let written = PsbValue::List(
            PsbList::from(
                vec![
                    PsbValue::Number(PsbNumber::Integer(12)),
                    PsbValue::StringRef(PsbReference::new(111)),
                ]
            )
        ).write_bytes_refs(&mut buffer, &PsbRefs::new(Vec::new(), Vec::new())).unwrap();
    
        println!("written: {} buffer: {:?}", written, buffer);
    }

    #[test]
    fn object_write() {
        let mut buffer = Vec::new();
        
        let written = PsbValue::Object(
            PsbObject::from({
                let mut map = HashMap::new();

                map.insert("sample1".into(), PsbValue::Number(PsbNumber::Integer(12)));
                map.insert("sample2".into(), PsbValue::Number(PsbNumber::Integer(13)));

                map
            })
        ).write_bytes_refs(&mut buffer, &PsbRefs::new(vec!["sample1".into(), "sample2".into()], Vec::new())).unwrap();
    
        println!("written: {} buffer: {:?}", written, buffer);
    }

    #[test]
    fn copy_test() {
        let file = File::open("01_com_001_01.ks.scn").unwrap();
        let mut file = PsbReader::open_psb(file).unwrap();

        let psb = file.load().unwrap();

        PsbWriter::new(psb, File::create("01_com_001_01.ks.re.scn").unwrap()).finish().unwrap();
    }
}