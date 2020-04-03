//! This library provides tools for decoding/encoding ASN.1 messages to/from their corresponding Rust types.
//!
//! # ASN.1 Type Correspondence
//!
//! Below are the currently supported ASN.1 types and their corresponding types/constructs in Rust.
//!
//! | ASN.1 Type     | Rust Type             |
//! |----------------|-----------------------|
//! | BIT STRING     | BitString             |
//! | INTEGER*       | i8,i16,i32,u8,u16,u32 |
//! | NULL           | ()                    |
//! | OCTET STRING   | Vec\<u8\>             |
//! | SEQUENCE       | struct                |
//! | SEQUENCE OF    | Vec\<T\>              |
//! | CHOICE         | enum                  |
//!
//! *`INTEGER` fields of arbitrary widths (in PER encodings) can be decoded/encoded as long as they fit in an `i64`
//! (see [aper::Decoder::decode_int](aper/struct.Decoder.html#method.decode_int) and
//! [aper::encode_int](aper/fn.encode_int.html)).
extern crate byteorder;

/// Tools for encoding and decoding ASN.1 messages of the Aligned PER flavor.
pub mod aper;

mod bit_string;
mod integer;
mod sequence_of;
mod sequence;
mod bool;
mod extensions;
mod null;
mod utils;

pub use bit_string::BitString;
pub use bool::*;
pub use extensions::*;
pub use integer::*;
pub use null::*;
pub use sequence::*;
pub use sequence_of::*;
