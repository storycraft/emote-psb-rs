/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod collection;
pub mod number;
pub mod resource;

use std::io::Read;

use collection::{PsbIntArray, PsbList, PsbMap};
use number::PsbNumber;

use crate::{ScnError, ScnErrorKind};
use byteorder::ReadBytesExt;

use self::resource::PsbResource;

pub const PSB_TYPE_NONE: u8 = 0x00;

pub const PSB_TYPE_NULL: u8 = 0x01;

pub const PSB_TYPE_FALSE: u8 = 0x02;
pub const PSB_TYPE_TRUE: u8 = 0x03;

/// 0 <= N <= 8
pub const PSB_TYPE_INTEGER_N: u8 = 0x04;
pub const PSB_TYPE_FLOAT0: u8 = 0x1d;
pub const PSB_TYPE_FLOAT: u8 = 0x1e;
pub const PSB_TYPE_DOUBLE: u8 = 0x1f;

/// 1 <= N <= 8
pub const PSB_TYPE_INTEGER_ARRAY_N: u8 = 0x0C;

/// 1 <= N <= 4
pub const PSB_TYPE_STRING: u8 = 0x14;

/// 1 <= N <= 4
pub const PSB_TYPE_RESOURCE_N: u8 = 0x18;

pub const PSB_TYPE_LIST: u8 = 0x20;
pub const PSB_TYPE_MAP: u8 = 0x21;

/// 1 <= N <= 8
pub const PSB_TYPE_EXTRA_N: u8 = 0x21;

pub const PSB_COMPILER_INTEGER: u8 = 0x80;
pub const PSB_COMPILER_STRING: u8 = 0x81;
pub const PSB_COMPILER_RESOURCE: u8 = 0x82;
pub const PSB_COMPILER_DECIMAL: u8 = 0x83;
pub const PSB_COMPILER_ARRAY: u8 = 0x84;
pub const PSB_COMPILER_BOOL: u8 = 0x85;
pub const PSB_COMPILER_BINARY_TREE: u8 = 0x86;

#[derive(Debug)]
pub enum PsbValue {

    None, Null,
    Bool(bool),
    Number(PsbNumber),
    IntArray(PsbIntArray),

    String(PsbResource),

    List(PsbList),
    Map(PsbMap),

    Resource(PsbResource),
    ExtraResource(PsbResource),

    CompilerNumber,
    CompilerString,
    CompilerResource,
    CompilerDecimal,
    CompilerArray,
    CompilerBool,
    CompilerBinaryTree

}

impl PsbValue {

    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, PsbValue), ScnError> {
        let value_type = stream.read_u8()?;

        match value_type {
            PSB_TYPE_NONE => Ok((1, PsbValue::None)),
            PSB_TYPE_NULL => Ok((1, PsbValue::Null)),

            PSB_TYPE_FALSE => Ok((1, PsbValue::Bool(false))),
            PSB_TYPE_TRUE => Ok((1, PsbValue::Bool(true))),
            
            PSB_TYPE_DOUBLE => {
                let (read, val) = PsbNumber::from_bytes(value_type, stream)?;
                Ok((read + 1, PsbValue::Number(val)))
            },

            PSB_TYPE_FLOAT0 => {
                let (read, val) = PsbNumber::from_bytes(value_type, stream)?;
                Ok((read + 1, PsbValue::Number(val)))
            },

            PSB_TYPE_FLOAT => {
                let (read, val) = PsbNumber::from_bytes(value_type, stream)?;
                Ok((read + 1, PsbValue::Number(val)))
            },

            _ if value_type >= PSB_TYPE_INTEGER_N && value_type <= PSB_TYPE_INTEGER_N + 8 => {
                let (read, number) = PsbNumber::from_bytes(value_type, stream)?;
                Ok((read + 1, PsbValue::Number(number)))
            },

            _ if value_type > PSB_TYPE_INTEGER_ARRAY_N && value_type <= PSB_TYPE_INTEGER_ARRAY_N + 8 => {
                let (read, array) = PsbIntArray::from_bytes(value_type - PSB_TYPE_INTEGER_ARRAY_N, stream)?;
                Ok((read + 1, PsbValue::IntArray(array)))
            },

            _ if value_type > PSB_TYPE_STRING && value_type <= PSB_TYPE_STRING + 4 => {
                let (read, string) = PsbResource::from_bytes(value_type - PSB_TYPE_STRING, stream)?;

                Ok((read + 1, PsbValue::String(string)))
            },

            PSB_TYPE_LIST => {
                let (read, list) = PsbList::from_bytes(stream)?;

                Ok((read + 1, PsbValue::List(list)))
            },

            PSB_TYPE_MAP => {
                let (read, map) = PsbMap::from_bytes(stream)?;

                Ok((read + 1, PsbValue::Map(map)))
            },

            _ if value_type > PSB_TYPE_RESOURCE_N && value_type <= PSB_TYPE_RESOURCE_N + 4 => {
                let (read, map) = PsbResource::from_bytes(value_type - PSB_TYPE_RESOURCE_N, stream)?;

                Ok((read + 1, PsbValue::Resource(map)))
            },

            _ if value_type > PSB_TYPE_EXTRA_N && value_type <= PSB_TYPE_EXTRA_N + 4 => {
                let (read, map) = PsbResource::from_bytes(value_type - PSB_TYPE_EXTRA_N, stream)?;

                Ok((read + 1, PsbValue::ExtraResource(map)))
            },

            PSB_COMPILER_INTEGER => Ok((1, PsbValue::CompilerNumber)),
            PSB_COMPILER_STRING => Ok((1, PsbValue::CompilerString)),
            PSB_COMPILER_RESOURCE => Ok((1, PsbValue::CompilerResource)),
            PSB_COMPILER_ARRAY => Ok((1, PsbValue::CompilerArray)),
            PSB_COMPILER_BOOL => Ok((1, PsbValue::CompilerBool)),
            PSB_COMPILER_BINARY_TREE => Ok((1, PsbValue::CompilerBinaryTree)),

            _ => {
                Err(ScnError::new(ScnErrorKind::InvalidPSBValue, None))
            }
        }
    } 

}