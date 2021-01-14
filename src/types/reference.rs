/*
 * Created on Sat Dec 26 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use crate::PsbError;

use super::number::PsbNumber;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PsbResourceRef {

    pub resource_ref: u64

}

impl PsbResourceRef {

    pub fn get_n(&self) -> u8 {
        PsbNumber::get_uint_n(self.resource_ref)
    }

    pub fn from_bytes(n: u8, stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        let (ref_read, refernece) = PsbNumber::read_uint(n, stream)?;

        Ok((ref_read, Self { resource_ref: refernece as u64 }))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        Ok(PsbNumber::write_uint(self.get_n(), self.resource_ref, stream)? as u64)
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PsbExtraRef {

    pub extra_resource_ref: u64

}

impl PsbExtraRef {

    pub fn get_n(&self) -> u8 {
        PsbNumber::get_uint_n(self.extra_resource_ref)
    }

    pub fn from_bytes(n: u8, stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        let (ref_read, refernece) = PsbNumber::read_uint(n, stream)?;

        Ok((ref_read, Self { extra_resource_ref : refernece as u64 }))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        Ok(PsbNumber::write_uint(self.get_n(), self.extra_resource_ref, stream)? as u64)
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PsbStringRef {

    pub string_ref: u64

}

impl PsbStringRef {

    pub fn get_n(&self) -> u8 {
        PsbNumber::get_uint_n(self.string_ref)
    }

    pub fn from_bytes(n: u8, stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        let (ref_read, refernece) = PsbNumber::read_uint(n, stream)?;

        Ok((ref_read, Self { string_ref : refernece as u64 }))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        Ok(PsbNumber::write_uint(self.get_n(), self.string_ref, stream)? as u64)
    }

}