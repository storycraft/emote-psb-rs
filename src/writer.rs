/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Seek, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{SCN_SIGNATURE, ScnError, ScnRefTable, header::ScnHeader, psb::PsbValue};

pub struct ScnWriter<T: Write + Seek> {

    pub header: ScnHeader,

    pub ref_table: ScnRefTable,

    pub entry: PsbValue,

    stream: T

}

impl<T: Write + Seek> ScnWriter<T> {

    pub fn new(
        header: ScnHeader,
        ref_table: ScnRefTable,
        entry: PsbValue,
        stream: T
    ) -> Self {
        Self {
            header,
            ref_table,
            entry,
            stream
        }
    }

    /// Write file and finish stream
    pub fn finish(mut self) -> Result<(u64, T), ScnError> {
        self.stream.write_u32::<LittleEndian>(SCN_SIGNATURE)?;
        let header_written = self.header.write_bytes(&mut self.stream)?;



        Ok((0, self.stream))
    }

}