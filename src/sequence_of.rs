use crate::{
    encode_length,
    APerDecode,
    APerEncode,
    Constraints,
    DecodeError,
    Decoder,
    EncodeError,
    Encoder,
};

impl<T: APerEncode> APerEncode for Vec<T> {
    const CONSTRAINTS: Constraints = Constraints {
        value: None,
        size: None,
    };

    fn to_aper(&self, constraints: Constraints) -> Result<Encoder, EncodeError> {
        let mut enc = encode_length(self.len())?;
        for x in self {
            let val = x.to_aper(Constraints {
                value: None,
                size: constraints.value,
            })?;
            enc.append(&val)?;
        }
        Ok(enc)
    }
}

impl<T: APerDecode> APerDecode for Vec<T> {
    const CONSTRAINTS: Constraints = Constraints {
        value: None,
        size: None,
    };

    /// Read a `Vec[T]` from an aligned PER encoding.
    fn from_aper(decoder: &mut Decoder<'_>, constraints: Constraints) -> Result<Self, DecodeError> {
        if constraints.size.is_none() {
            return Err(DecodeError::MissingSizeConstraint);
        }
        let sz_constr = constraints.size.unwrap();

        let mut min_len: usize = 0;
        let mut max_len: usize = 0;
        if sz_constr.min().is_some() {
            min_len = sz_constr.min().unwrap() as usize;
        }
        if sz_constr.max().is_some() {
            max_len = sz_constr.max().unwrap() as usize;
        }

        if max_len >= 65535 {
            return Err(DecodeError::NotImplemented);
        }

        let len: usize;
        if max_len == min_len {
            len = max_len;
        } else {
            let ret = decoder.decode_length();
            if ret.is_err() {
                return Err(ret.err().unwrap());
            }
            len = ret.unwrap();
        }

        // XXX: This is terrible, but convenient. Either fix or document thoroughly.
        let el_constrs = Constraints {
            value: None,
            size: constraints.value,
        };
        let mut content: Vec<T> = Vec::with_capacity(len);
        for _ in 0..len {
            let val = T::from_aper(decoder, el_constrs)?;
            content.push(val);
        }

        Ok(content)
    }
}
