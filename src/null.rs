use crate::{
    APerDecode,
    APerEncode,
    Constraints,
    DecodeError,
    Decoder,
    EncodeError,
    Encoder,
};

impl APerEncode for () {
    const CONSTRAINTS: Constraints = Constraints {
        value: None,
        size: None,
    };

    fn to_aper(&self, _: Constraints) -> Result<Encoder, EncodeError> {
        Ok(Encoder::new())
    }
}

impl APerDecode for () {
    const CONSTRAINTS: Constraints = Constraints {
        value: None,
        size: None,
    };

    /// Read `()` from an aligned PER encoding.
    fn from_aper(_: &mut Decoder<'_>, _: Constraints) -> Result<Self, DecodeError> {
        Ok(())
    }
}
