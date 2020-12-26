/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod array;
pub mod number;

pub const PSB_TYPE_NONE: u8 = 0x00;
pub const PSB_TYPE_NULL: u8 = 0x01;
pub const PSB_TYPE_FALSE: u8 = 0x02;
pub const PSB_TYPE_TRUE: u8 = 0x03;
pub const PSB_TYPE_INTEGER: u8 = 0x04;
pub const PSB_TYPE_FLOAT0: u8 = 0x1d;
pub const PSB_TYPE_FLOAT: u8 = 0x1e;
pub const PSB_TYPE_DOUBLE: u8 = 0x1f;

pub const PSB_TYPE_INTEGER_ARRAY: u8 = 0x0C;


#[repr(u8)]
pub enum PsbTypeIdentifier {

    None = 0x0,
    Null = 0x1,
    False = 0x2,
    True = 0x3,

    // int, long
    NumberN0 = 0x4,
    NumberN1 = 0x5,
    NumberN2 = 0x6,
    NumberN3 = 0x7,
    NumberN4 = 0x8,
    NumberN5 = 0x9,
    NumberN6 = 0xA,
    NumberN7 = 0xB,
    NumberN8 = 0xC,

    // array N(NUMBER) is count mask
    ArrayN1 = 0xD,
    ArrayN2 = 0xE,
    ArrayN3 = 0xF,
    ArrayN4 = 0x10,
    ArrayN5 = 0x11,
    ArrayN6 = 0x12,
    ArrayN7 = 0x13,
    ArrayN8 = 0x14,

    // index of strings table
    StringN1 = 0x15,
    StringN2 = 0x16,
    StringN3 = 0x17,
    StringN4 = 0x18,

    // resource of thunk
    ResourceN1 = 0x19,
    ResourceN2 = 0x1A,
    ResourceN3 = 0x1B,
    ResourceN4 = 0x1C,

    // fpu value
    Float0 = 0x1D,
    Float = 0x1E,
    Double = 0x1F,

    // objects
    List = 0x20, // object list
    Objects = 0x21, // object dictionary

    ExtraChunkN1 = 0x22,
    ExtraChunkN2 = 0x23,
    ExtraChunkN3 = 0x24,
    ExtraChunkN4 = 0x25,

    // used by compiler,it's fake
    Integer = 0x80,
    String = 0x81,
    Resource = 0x82,
    Decimal = 0x83,
    Array = 0x84,
    Boolean = 0x85,
    BTree = 0x86

}