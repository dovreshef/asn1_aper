This crate provides tools for encoding and decoding APER (Aligned Packed Encoding Rules) ASN.1 messages.

# Documentation

Run `cargo doc --open`

# Usage

Add the following to your `Cargo.toml`.

```rust
[dependencies]
asn1 = { git = "https://github.com/dovreshef/asn1_aper" }
```

To encode/decode your own types, just implement the `APerEncode`\\`APerDecode` traits. Below is an example of a decode operation.

let's consider an enum that corresponds to the ASN.1 Choice type below. (Note the extension marker)

```
Foo ::= SEQUENCE {
    a BIT STRING(SIZE(4))
}

Bar ::= SEQUENCE {
    a OCTET STRING
}

Baz ::= SEQUENCE {
    a INTEGER(0..255)
    b INTEGER(0..65535)
}

MyMsg ::= CHOICE {
    foo Foo
    bar Bar
    baz Baz
    ...
}
```

```rust
use asn1_aper::{BitString, APerDecode, Constraint, Constraints, UNCONSTRAINED};

enum MyMsg {
    foo { a: BitString, },
    bar { a: Vec<u8>, },
    baz { a: u8, b: u16, },
}

impl APerDecode for MyMsg {
    const CONSTRAINTS: Constraints = UNCONSTRAINED;
    fn from_aper(decoder: &mut Decoder, constraints: Constraints) -> Result<Self, DecodeError> {
        let is_ext = ExtensionMarker::from_aper(decoder, UNCONSTRAINED)?;

        let choice = decoder.decode_int(Some(0), Some(2))?;

        match choice {
            0 => {
                let bs = BitString::from_aper(decoder , Constraints {
                    value: None,
                    size: Some(Constraint::new(None, Some(4))),
                })?;
                 Ok(MyMsg::foo{ a: bs })
            },
            1 => {
                let v = Vec::<u8>::from_aper(decoder, Constraints {
                    value: None,
                    size: Some(Constraint::new(None, Some(3))),
                })?;
                Ok(MyMsg::bar{ a: v, })
            },
            2 => {
                let a = u8::from_aper(decoder, UNCONSTRAINED)?;
                let b = u16::from_aper(decoder, UNCONSTRAINED)?;
                Ok(MyMsg::baz{ a, b, })
            }
            _ => Err(aper::DecodeError::InvalidChoice)
        }
    }
}
```
