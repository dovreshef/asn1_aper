use crate::{
    APerDecode,
    APerEncode,
    Constraints,
    DecodeError,
    Decoder,
    EncodeError,
    Encoder,
};

impl APerEncode for bool {
    const CONSTRAINTS: Constraints = Constraints {
        value: None,
        size: None,
    };

    fn to_aper(&self, _: Constraints) -> Result<Encoder, EncodeError> {
        Ok(Encoder::with_bytes_and_padding(vec![(*self as u8) << 7], 7))
    }
}

impl APerDecode for bool {
    const CONSTRAINTS: Constraints = Constraints {
        value: None,
        size: None,
    };

    /// Read a `bool` from an aligned PER encoding.
    fn from_aper(decoder: &mut Decoder<'_>, _: Constraints) -> Result<Self, DecodeError> {
        let val = decoder.read(1)?;
        Ok(val > 0)
    }
}
