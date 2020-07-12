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
//! (see [Decoder::decode_int](aper/struct.Decoder.html#method.decode_int) and
//! [encode_int](aper/fn.encode_int.html)).
mod bit_string;
mod bool;
/// Tools for encoding and decoding ASN.1 messages of the Aligned PER flavor.
mod constraints;
mod decode;
mod encode;
mod extensions;
mod integer;
mod null;
mod sequence;
mod sequence_of;
mod utils;

pub use crate::{
    bit_string::BitString,
    bool::*,
    constraints::{
        Constraint,
        Constraints,
        UNCONSTRAINED,
    },
    decode::{
        APerDecode,
        DecodeError,
        Decoder,
    },
    encode::{
        encode_int,
        encode_length,
        APerEncode,
        EncodeError,
        Encoder,
    },
    extensions::*,
    integer::*,
    null::*,
    sequence::*,
    sequence_of::*,
};
