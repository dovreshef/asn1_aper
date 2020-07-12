use crate::constraints::{
    Constraints,
    LENGTH_DET_FRAG,
    LENGTH_DET_LONG,
    LENGTH_MASK_LONG,
    LENGTH_MASK_SHORT,
};
use byteorder::{
    BigEndian,
    ByteOrder,
};

/// Trait for Aligned PER decoding.
///
/// # Examples
///
/// Consider a simple ASN.1 Sequence `foo` made up of a `BitString` and a 32-bit non-negative integer.
///
/// ```
/// foo ::= SEQUENCE {
///     bar BIT STRING(SIZE(4)
///     baz INTEGER(0..4294967295)
/// }
/// ```
///
/// The corresponding struct and `APerElement` implementation are shown below.
///
/// ```
/// use asn1_aper::{BitString, APerDecode, Constraint, Constraints, UNCONSTRAINED};
///
/// struct foo {
///     pub bar: BitString,
///     pub baz: u32,
/// }
///
/// impl APerDecode for Foo {
///    type Result = Self;
///    const TAG: u32 = 0xBEEF;
///    const CONSTRAINTS: Constraints = UNCONSTRAINED;
///    fn from_aper(decoder: &mut Decoder, constraints: Constraints) -> Result<Self::Result, DecodeError> {
///        let bar = BitString::from_aper(decoder , Constraints {
///            value: None,
///            size: Some(Constraint::new(Some(4), Some(4))),
///        })?;
///
///        let mut baz = u32::from_aper(decoder, UNCONSTRAINED)?;
///
///        Ok(Foo{
///            bar,
///            baz,
///        })
///    }
/// }
/// ```
///
/// Now let's consider an enum that corresponds to the ASN.1 Choice type below. (Note the extension marker)
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
/// The corresponding enum and `APerElement` implementation would look like this.
///
/// ```
/// use asn1_aper::{BitString, APerDecode, Constraint, Constraints, UNCONSTRAINED};
///
/// enum MyMsg {
///     foo { a: BitString, },
///     bar { a: Vec<u8>, },
///     baz { a: u8, b: u16, },
/// }
///
/// impl APerDecode for MyMsg {
///     const CONSTRAINTS: Constraints = UNCONSTRAINED;
///     fn from_aper(decoder: &mut Decoder, constraints: Constraints) -> Result<Self, DecodeError> {
///         let is_ext = ExtensionMarker::from_aper(decoder, UNCONSTRAINED)?;
///
///         let choice = decoder.decode_int(Some(0), Some(2))?;
///
///         match choice {
///             0 => {
///                 let bs = BitString::from_aper(decoder , Constraints {
///                     value: None,
///                     size: Some(Constraint::new(None, Some(4))),
///                 })?;
///                  Ok(MyMsg::foo{ a: bs })
///             },
///             1 => {
///                 let v = Vec::<u8>::from_aper(decoder, Constraints {
///                     value: None,
///                     size: Some(Constraint::new(None, Some(3))),
///                 })?;
///                 Ok(MyMsg::bar{ a: v, })
///             },
///             2 => {
///                 let a = u8::from_aper(decoder, UNCONSTRAINED)?;
///                 let b = u16::from_aper(decoder, UNCONSTRAINED)?;
///                 Ok(MyMsg::baz{ a, b, })
///             }
///             _ => Err(aper::DecodeError::InvalidChoice)
///         }
///     }
/// }
/// ```
pub trait APerDecode: Sized {
    /// PER-visible Constraints
    const CONSTRAINTS: Constraints;

    /// Constructor for the `Result` type given an aligned PER encoding.
    fn from_aper(decoder: &mut Decoder<'_>, constraints: Constraints) -> Result<Self, DecodeError>;
}

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    InvalidChoice,
    MalformedLength,
    MalformedInt,
    MissingSizeConstraint,
    MissingValueConstraint,
    NotEnoughBits,
    NotImplemented,
}

/// A bit-wise cursor used to decode aligned PER messages.
///
/// # Examples
///
/// ```
/// use asn1_aper::{self, Decoder, APerElement, UNCONSTRAINED};
/// let data = b"\x80\x2b"; // 43
/// let mut d = Decoder::new(data);
/// let x = i16::from_aper(&mut d, UNCONSTRAINED).unwrap();
/// println!("x = {}", x); // Prints x = 43
/// ```
pub struct Decoder<'a> {
    data: &'a [u8],
    len: usize,
    pos: usize,
}

impl<'a> Decoder<'a> {
    /// Construct a new `Decoder` with an array of bytes.
    pub fn new(data: &'a [u8]) -> Decoder<'_> {
        Decoder {
            data,
            len: 8 * data.len(),
            pos: 0,
        }
    }

    /// Read `n` bits. Where `0 <= n <= 8`. See [read_to_vec()](#method.read_to_vec) for larger `n`.
    /// Returns an `Err` if the read would consume more bits than are available. Else, returns the bits as a u8 with
    /// left-padding.
    ///
    /// # Examples
    ///
    /// In some cases, elements of aligned PER messages will be encoded using only the minimum number of bits required to
    /// express the value without alignment on a byte boundary. `read` allows you to decode these fields.
    ///
    /// For example, consider a bit field that only occupies three bits.
    ///
    /// ```
    /// let data = b"\xe0";
    /// let mut d = aper::Decoder::new(data);
    /// let x = d.read(3).unwrap();
    /// println!("x = 0x{:X}"); // Prints x = 0x07
    /// ```
    pub fn read(&mut self, n: usize) -> Result<u8, DecodeError> {
        if n == 0 {
            return Ok(0);
        }
        if self.pos + n > self.len {
            return Err(DecodeError::NotEnoughBits);
        }

        let l_bucket = self.pos / 8;
        let h_bucket = (self.pos + n) / 8;
        let h_off = (self.pos + n) - h_bucket * 8;
        let mut ret: u8;

        if l_bucket == h_bucket {
            let mask = (0xFF >> (8 - n)) << (8 - h_off);
            ret = (self.data[l_bucket] & mask) >> (8 - h_off);
        } else if l_bucket < h_bucket && h_off == 0 {
            let mask = 0xFF >> (8 - n);
            ret = self.data[l_bucket] & mask;
        } else {
            let l_mask = 0xFF >> (8 - (n - h_off));
            let h_mask = 0xFF << (8 - h_off);
            ret = (self.data[l_bucket] & l_mask) << h_off;
            ret |= (self.data[h_bucket] & h_mask) >> (8 - h_off);
        }
        self.pos += n;
        Ok(ret)
    }

    /// Read a byte.
    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        self.read(8).map_err(|_| DecodeError::NotEnoughBits)
    }

    /// Read `len` bits into `content`.
    /// Returns an `Err` if the read would consume more bits than are available. Else, the bits as a `u8`s with
    /// left-padding are pushed onto `content`.
    ///
    /// # Examples
    ///
    /// Some fields may span multiple bytes. `read_to_vec` allows you to decode these fields.
    ///
    /// ```
    /// use asn1_aper::aper::Decoder;
    /// let data = b"\xff\xf3";
    /// let mut d = Decoder::new(data);
    /// let mut x: Vec<u8> = Vec::with_capacity(2);
    /// d.read_to_vec(&mut x, 12).unwrap();
    /// assert_eq!(x, vec![255, 15]);
    /// ```
    pub fn read_to_vec(&mut self, content: &mut Vec<u8>, len: usize) -> Result<(), DecodeError> {
        if len == 0 {
            return Ok(());
        }
        if self.pos + len > self.len {
            return Err(DecodeError::NotEnoughBits);
        }

        if len < 8 {
            content.push(self.read(len)?);
        } else {
            let num_bytes = (len as f64 / 8.).ceil() as usize;
            for _ in 0..num_bytes {
                content.push(self.read_u8()?);
            }
            self.pos -= len % 8;
        }
        Ok(())
    }

    /// Decode an aligned PER length determinant
    pub fn decode_length(&mut self) -> Result<usize, DecodeError> {
        let val = self.read_u8().map_err(|_| DecodeError::MalformedLength)?;

        if val & LENGTH_DET_FRAG > 0 {
            return Err(DecodeError::NotImplemented);
        }

        if val & LENGTH_DET_LONG > 0 {
            let len = (val & LENGTH_MASK_LONG) as usize;
            let val = self.read_u8().map_err(|_| DecodeError::MalformedLength)?;
            return Ok((len << 8) + val as usize);
        }

        Ok((val & LENGTH_MASK_SHORT) as usize)
    }

    /// Decode an Aligned PER integer between `min` and `max`
    ///
    /// You can decode the Rust primitive (u)ints: `i8`, `i16`, `i32`, `u8`, `u16`, and `u32` using their respective
    /// `from_aper` constructors. `decode_int` is useful if you want to decode an integer field that exists somewhere
    /// between or beyond the primitive widths.
    ///
    /// # Examples
    ///
    /// For example, a value in [500, 503] can be encoded using two bits in aligned PER, so using
    /// `u8` would yield an incorrect value. The code below demonstrates how to decode such a field.
    ///
    /// ```
    /// let data = b"\x70"; // 0111 0000
    /// let mut d = aper::Decoder::new(data);
    /// let x = d.decode_int(Some(500), Some(503)).unwrap();
    /// let y = d.decode_int(Some(500), Some(503)).unwrap();
    /// println!("x = {}", x); // Prints x = 501
    /// println!("y = {}", y); // Prints y = 503
    /// ```
    pub fn decode_int(&mut self, min: Option<i64>, max: Option<i64>) -> Result<i64, DecodeError> {
        if let (Some(l), Some(h)) = (min, max) {
            // constrained
            let range = h - l + 1;
            let n_bits = (range as f64).log2().ceil() as usize;

            if n_bits < 8 {
                let val = self.read(n_bits)?;
                return Ok(val as i64 + l);
            }

            // Simple case, no length determinant
            if n_bits <= 16 {
                let mut val = self.read_u8()? as u16;
                if n_bits != 8 {
                    let n_val = self.read_u8()?;
                    val = (n_val as u16) + (val << 8);
                }
                return Ok(val as i64 + l);
            }

            // Need to decode length determinant
            let len = self.decode_length()?;
            if len > 8 {
                return Err(DecodeError::NotImplemented);
            }

            let mut content = Vec::with_capacity(len);
            self.read_to_vec(&mut content, len * 8)?;

            let val = BigEndian::read_uint(&content.as_slice(), len) as i64 + l;
            if val < l || val > h {
                return Err(DecodeError::MalformedInt);
            }
            return Ok(val);
        }

        let len = self.decode_length()?;
        let mut content = Vec::with_capacity(len);
        self.read_to_vec(&mut content, len * 8)?;

        match min {
            Some(min) => Ok(BigEndian::read_int(&content, len) + min),
            None => Ok(BigEndian::read_int(&content, len)),
        }
    }
}
