use crate::{
    encode_int,
    APerDecode,
    APerEncode,
    Constraints,
    DecodeError,
    Decoder,
    EncodeError,
    Encoder,
};
use std::{
    i16,
    i32,
    i8,
    u16,
    u32,
    u8,
};

macro_rules! int_impl {
    ($t:ident) => {
        impl APerEncode for $t {
            const CONSTRAINTS: Constraints = Constraints {
                value: None,
                size: None,
            };
            fn to_aper(&self, _: Constraints) -> Result<Encoder, EncodeError> {
                let val = encode_int(*self as i64, Some($t::MIN as i64), Some($t::MAX as i64))?;
                Ok(val)
            }
        }

        impl APerDecode for $t {
            const CONSTRAINTS: Constraints = Constraints {
                value: None,
                size: None,
            };
            /// Read an `$t` from an aligned PER encoding.
            fn from_aper(decoder: &mut Decoder<'_>, _: Constraints) -> Result<Self, DecodeError> {
                let val = decoder.decode_int(Some($t::MIN as i64), Some($t::MAX as i64))?;
                Ok(val as $t)
            }
        }
    };
}

int_impl!(i8);
int_impl!(i16);
int_impl!(i32);
int_impl!(u8);
int_impl!(u16);
int_impl!(u32);
