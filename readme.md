# emote-psb
Emote psb file library

## What is a PSB file?
PSB is a proprietary binary format used by E-mote, a 2D animation middleware developed by M2 Co., Ltd.
It stores structured data such as animation trees, sprites, and metadata that drive E-mote character animations.
MDF files are compressed, encrypted variants of PSB files.

## Features

 * **Read PSB files** — parse PSB files from any `BufRead + Seek` stream via `PsbFile::open`, supporting multiple PSB format versions
 * **Write PSB files** — serialize data to PSB format via `PsbWriter`, with configurable version, an encryption header flag, and Adler-32 checksum generation
 * **Read MDF files** — transparently decompress zlib-compressed MDF containers via `MdfReader`, exposing the inner PSB stream for further parsing
 * **Write MDF files** — produce MDF containers via `MdfWriter` with configurable zlib compression level
 * **Serde integration** — deserialize the PSB root object into any `serde::Deserialize` type, or serialize any `serde::Serialize` type directly into a PSB file
 * **Rich value type** — `PsbValue` represents the full PSB type system: null, booleans, integers, floats, strings, lists, objects, binary resources, extra resources, and PSB compiler intrinsics
 * **Resource access** — read embedded binary resources and extra resources as seekable byte streams via `PsbFile::open_resource` and `PsbFile::open_extra_resource`

## License
This project is licensed under the [MIT License](LICENSE).