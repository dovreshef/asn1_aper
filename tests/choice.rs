use asn1_aper::{
    encode_int,
    APerDecode,
    APerEncode,
    BitString,
    Constraint,
    Constraints,
    DecodeError,
    Decoder,
    EncodeError,
    Encoder,
    ExtensionMarker,
    UNCONSTRAINED,
};

enum Foo {
    Foo { a: BitString },
    Bar { a: Vec<u8> },
    Baz { a: u8, b: u16 },
}

impl APerEncode for Foo {
    const CONSTRAINTS: Constraints = UNCONSTRAINED;

    fn to_aper(&self, _: Constraints) -> Result<Encoder, EncodeError> {
        let mut enc = (false as ExtensionMarker).to_aper(UNCONSTRAINED)?;
        match *self {
            Foo::Foo { ref a } => {
                enc.append(&encode_int(0, Some(0), Some(2))?);
                enc.append(
                    &a.to_aper(Constraints {
                        value: None,
                        size: Some(Constraint::new(None, Some(4))),
                    })?,
                );
            }
            Foo::Bar { ref a } => {
                enc.append(&encode_int(1, Some(0), Some(2))?);
                enc.append(&a.to_aper(UNCONSTRAINED)?);
            }
            Foo::Baz { ref a, ref b } => {
                enc.append(&encode_int(2, Some(0), Some(2))?);
                enc.append(&a.to_aper(UNCONSTRAINED)?);
                enc.append(&b.to_aper(UNCONSTRAINED)?);
            }
        };
        Ok(enc)
    }
}

impl APerDecode for Foo {
    const CONSTRAINTS: Constraints = UNCONSTRAINED;
    fn from_aper(decoder: &mut Decoder<'_>, _: Constraints) -> Result<Self, DecodeError> {
        let is_ext = ExtensionMarker::from_aper(decoder, UNCONSTRAINED);
        if is_ext.is_err() {
            return Err(is_ext.err().unwrap());
        }

        let choice = decoder.decode_int(Some(0), Some(2));
        if choice.is_err() {
            return Err(choice.err().unwrap());
        }

        let c = choice.unwrap();
        println!("{}", c);
        match c {
            0 => {
                let bs = BitString::from_aper(
                    decoder,
                    Constraints {
                        value: None,
                        size: Some(Constraint::new(None, Some(4))),
                    },
                );
                if bs.is_err() {
                    Err(bs.err().unwrap())
                } else {
                    Ok(Foo::Foo { a: bs.unwrap() })
                }
            }
            1 => {
                let v = Vec::<u8>::from_aper(
                    decoder,
                    Constraints {
                        value: None,
                        size: Some(Constraint::new(None, Some(3))),
                    },
                );
                if v.is_err() {
                    Err(v.err().unwrap())
                } else {
                    Ok(Foo::Bar { a: v.unwrap() })
                }
            }
            2 => {
                let a = u8::from_aper(decoder, UNCONSTRAINED)?;
                let b = u16::from_aper(decoder, UNCONSTRAINED)?;
                Ok(Foo::Baz {
                    a,
                    b,
                })
            }
            _ => Err(DecodeError::InvalidChoice),
        }
    }
}

#[test]
fn encode_foo() {
    let x: Foo = Foo::Foo {
        a: BitString::with_bytes_and_len(&vec![0x0e], 4),
    };
    let target: Vec<u8> = vec![0x1c];
    assert_eq!(target, *x.to_aper(UNCONSTRAINED).unwrap().bytes());
}

#[test]
fn encode_bar() {
    let x: Foo = Foo::Bar {
        a: vec![0x46, 0x4f, 0x4f],
    };
    let target: Vec<u8> = vec![32, 104, 201, 233, 224];
    assert_eq!(target, *x.to_aper(UNCONSTRAINED).unwrap().bytes());
}

#[test]
fn encode_baz() {
    let x: Foo = Foo::Baz { a: 42, b: 300 };
    let target: Vec<u8> = vec![69, 64, 37, 128];
    assert_eq!(target, *x.to_aper(UNCONSTRAINED).unwrap().bytes());
}

#[test]
fn decode_foo() {
    let data = b"\x1c"; // 0001 1100
    let mut d = Decoder::new(data);
    let f = Foo::from_aper(&mut d, UNCONSTRAINED).unwrap();

    match f {
        Foo::Foo { a: x } => {
            for i in 0..4 {
                if i == 1 || i == 2 || i == 3 {
                    assert_eq!(true, x.is_set(i));
                } else {
                    assert_eq!(false, x.is_set(i));
                }
            }
        }
        _ => assert!(false),
    }
}

#[test]
fn decode_bar() {
    let data = b"\x01\x03\x46\x4f\x4f";
    let mut d = Decoder::new(data);
    d.read(5).unwrap(); // strip left-padding
    let f = Foo::from_aper(&mut d, UNCONSTRAINED).unwrap();
    let target_bar = vec![0x46 as u8, 0x4f as u8, 0x4f as u8];

    match f {
        Foo::Bar { a: x } => {
            assert_eq!(x.len(), target_bar.len());
            for i in 0..x.len() {
                assert_eq!(x[i], target_bar[i]);
            }
        }
        _ => assert!(false),
    }
}

#[test]
fn decode_baz() {
    let data = b"\x02\x88\xf9\x3b";
    let mut d = Decoder::new(data);
    d.read(5).unwrap(); // strip left-padding
    let f = Foo::from_aper(&mut d, UNCONSTRAINED).unwrap();

    match f {
        Foo::Baz { a: x, b: y } => {
            assert_eq!(0x88 as u8, x);
            assert_eq!(0xf93b as u16, y);
        }
        _ => assert!(false),
    }
}
