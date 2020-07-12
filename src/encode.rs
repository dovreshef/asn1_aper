use crate::{
    constraints::{
        Constraints,
        LENGTH_DET_LONG,
        LENGTH_DET_SHORT,
        LENGTH_MASK_LONG,
        LENGTH_MASK_SHORT,
    },
    utils::shift_bytes_left,
};
use byteorder::{
    BigEndian,
    WriteBytesExt,
};

/// Trait for Aligned PER encoding.
///
/// # Examples
///
/// Consider an enum that corresponds to the ASN.1 Choice type below. (Note the extension marker)
///
/// ```
/// Foo ::= SEQUENCE {
///     a BIT STRING(SIZE(4))
/// }
///
/// Bar ::= SEQUENCE {
///     a OCTET STRING
/// }
///
/// Baz ::= SEQUENCE {
///     a INTEGER(0..255)
///     b INTEGER(0..65535)
/// }
///
/// MyMsg ::= CHOICE {
///     foo Foo
///     bar Bar
///     baz Baz
///     ...
/// }
/// ```
///
/// The corresponding enum and `APerEncode` implementation would look like this.
///
/// ```
/// use asn1_aper::{BitString, APerEncode, Constraint, Constraints, UNCONSTRAINED};
///
/// enum MyMsg {
///     foo { a: BitString, },
///     bar { a: Vec<u8>, },
///     baz { a: u8, b: u16, },
/// }
///
/// impl APerEncode for MyMsg {
///     const CONSTRAINTS: Constraints = UNCONSTRAINED;
///     fn to_aper(&self, constraints: Constraints) -> Result<Encoder, EncodeError> {
///         let mut enc = (false as ExtensionMarker).to_aper(UNCONSTRAINED)?;
///         match *self {
///             Foo::foo{a: ref a} => {
///                 enc.append(&encode_int(0, Some(0), Some(2))?);
///                 enc.append(&a.to_aper(Constraints {
///                     value: None,
///                     size: Some(Constraint::new(None, Some(4))),
///                 })?);
///             },
///             Foo::bar{a: ref a} => {
///                 enc.append(&encode_int(1, Some(0), Some(2))?);
///                 enc.append(&a.to_aper(UNCONSTRAINED)?);
///             },
///             Foo::baz{a: ref a, b: ref b} => {
///                 enc.append(&encode_int(2, Some(0), Some(2))?);
///                 enc.append(&a.to_aper(UNCONSTRAINED)?);
///                 enc.append(&b.to_aper(UNCONSTRAINED)?);
///             },
///         };
///         Ok(enc)
///     }
/// }
/// ```
pub trait APerEncode: Sized {
    /// PER-visible Constraints
    const CONSTRAINTS: Constraints;

    /// For use with `Encoder::append`
    fn to_aper(&self, constraints: Constraints) -> Result<Encoder, EncodeError>;
}

#[derive(Debug, PartialEq)]
pub enum EncodeError {
    MissingSizeConstraint,
    MissingValueConstraint,
    NotImplemented,
    WriteError,
}

/// A wrapper for an aligned PER encoding.
///
/// An `Encoder` is just a vector of bytes with right-padding at the end if necessary.
///
/// # Examples
///
/// ```
/// extern crate asn1;
/// use asn1::aper::{self, APerElement, Constraint, Constraints, Encoder, UNCONSTRAINED};
///
/// let mut enc = Encoder::new();
/// enc.append(&true.to_aper(UNCONSTRAINED).unwrap()).unwrap();
/// println!("enc = {:?}", *enc.bytes()); // Prints enc = [128]
/// ```
#[derive(Debug)]
pub struct Encoder {
    bytes: Vec<u8>,
    r_padding: usize,
}

impl Encoder {
    /// Construct a new, empty `Encoder`.
    pub fn new() -> Encoder {
        Encoder {
            bytes: Vec::new(),
            r_padding: 0,
        }
    }

    /// Construct a new `Encoder` with `bytes` and `r_pad` bits of right-padding.
    pub fn with_bytes_and_padding(bytes: Vec<u8>, r_pad: usize) -> Encoder {
        Encoder {
            bytes,
            r_padding: r_pad,
        }
    }

    /// Construct a new `Encoder` with `bytes` and zero bits of right-padding.
    pub fn with_bytes(bytes: Vec<u8>) -> Encoder {
        Self::with_bytes_and_padding(bytes, 0)
    }

    /// Append `other` to the end of `self`, starting with the `r_padding`th LSB of `self`.
    pub fn append(&mut self, other: &Encoder) {
        let mut bytes = other.bytes().clone();
        let r_padding = other.r_padding();

        let n = self.bytes.len();

        if n == 0 {
            self.bytes.append(&mut bytes);
            self.r_padding = r_padding;
            return;
        }

        if bytes.len() == 0 {
            return;
        }

        // Fill LSBs of self.bytes first
        if self.r_padding > 0 {
            let mask = 0xFF << r_padding;
            self.bytes[n - 1] |= (bytes[0] & mask) >> (8 - self.r_padding);

            shift_bytes_left(&mut bytes, self.r_padding);

            let len = 8 - r_padding;
            if len <= self.r_padding {
                self.r_padding -= len;
                bytes.remove(0);
            }
        } else {
            self.r_padding = r_padding;
        }

        // Just append everything else
        self.bytes.append(&mut bytes);
    }

    /// Get a reference to the bytes of an encoder.
    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    /// Get a mutable reference to the bytes of an encoder.
    pub fn bytes_mut(&self) -> &Vec<u8> {
        &self.bytes
    }

    /// Get the number of right-padding bits.
    pub fn r_padding(&self) -> usize {
        self.r_padding
    }

    /// Set the number of right-padding bits.
    pub fn set_r_padding(&mut self, n: usize) {
        self.r_padding = n;
    }
}

/// Encode an aligned PER length determinant.
pub fn encode_length(len: usize) -> Result<Encoder, EncodeError> {
    if len < 128 {
        return Ok(Encoder::with_bytes(vec![
            (len as u8 & LENGTH_MASK_SHORT) | LENGTH_DET_SHORT,
        ]));
    } else if len < 65535 {
        let upper = (len >> 8) as u8;
        let lower = len as u8;
        return Ok(Encoder::with_bytes(vec![
            (upper & LENGTH_MASK_LONG) | LENGTH_DET_LONG,
            lower,
        ]));
    } else {
        return Err(EncodeError::NotImplemented);
    }
}

/// Encode an aligned PER integer between `min` and `max`.
///
/// You can encode the Rust primitive (u)ints: `i8`, `i16`, `i32`, `u8`, `u16`, and `u32` using their respective
/// `to_aper` functions. `encode_int` is useful if you want to encode an integer field that exists somewhere
/// between or beyond the primitive widths.
///
/// # Examples
///
/// For example, a value in [500, 503] can be encoded using two bits in aligned PER, so using
/// `u16` would be a waste if bandwidth is precious. The code below demonstrates how to decode such a field.
///
/// ```
/// extern crate asn1;
/// use asn1::{self, APerElement, Constraint, Constraints, Encoder, encode_int, UNCONSTRAINED};
///
/// let x = 501;
/// println!("{:?}", encode_int(x, Some(500), Some(503).unwrap().bytes()); // Prints [64]
/// ```
pub fn encode_int(value: i64, min: Option<i64>, max: Option<i64>) -> Result<Encoder, EncodeError> {
    if let (Some(l), Some(h)) = (min, max) {
        // constrained
        let v = value - l;
        let range = h - l + 1;
        let n_bits = (range as f64).log2().ceil() as usize;

        // No alignment
        if n_bits < 8 {
            return Ok(Encoder::with_bytes_and_padding(
                vec![(v as u8) << (8 - n_bits)],
                8 - n_bits,
            ));
        }

        // Simple case, no length determinant
        if n_bits <= 16 {
            let mut bytes = vec![v as u8];

            if n_bits > 8 {
                bytes.insert(0, (v >> 8) as u8);
            }
            return Ok(Encoder::with_bytes(bytes));
        }

        // Need to encode with length determinant
        let len = (n_bits as f64 / 8.).ceil() as usize;
        let mut enc = encode_length(len)?;
        let mut bytes: Vec<u8> = Vec::new();
        bytes
            .write_uint::<BigEndian>(v as u64, len)
            .map_err(|_| EncodeError::WriteError)?;
        enc.append(&Encoder::with_bytes(bytes));
        return Ok(enc);
    }

    let n_bits = (value as f64).log2().ceil() as usize;
    let len = (n_bits as f64 / 8.).ceil() as usize;
    let mut enc = encode_length(len)?;
    let mut bytes: Vec<u8> = Vec::new();

    match min {
        Some(min) => bytes
            .write_uint::<BigEndian>((value - min) as u64, len)
            .map_err(|_| EncodeError::WriteError)?,
        None => bytes
            .write_uint::<BigEndian>(value as u64, len)
            .map_err(|_| EncodeError::WriteError)?,
    }
    enc.append(&Encoder::with_bytes(bytes));
    Ok(enc)
}
