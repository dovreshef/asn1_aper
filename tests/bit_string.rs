use asn1_aper::{
    APerDecode,
    APerEncode,
    BitString,
    Constraint,
    Constraints,
    Decoder,
};

#[test]
fn get_set() {
    let mut b = BitString::with_len(64);
    assert_eq!(false, b.is_set(0));
    b.set(0, true);
    assert_eq!(true, b.is_set(0));
}

#[test]
fn get_set_non_boundary() {
    let mut b = BitString::with_len(64);
    b.set(9, true);
    assert_eq!(true, b.is_set(9));
}

#[test]
fn decode_padded() {
    let data = b"\x00\xe0\x00";
    let mut d = Decoder::new(data);
    let b = BitString::from_aper(
        &mut d,
        Constraints {
            value: None,
            size: Some(Constraint::new(None, Some(20))),
        },
    )
    .unwrap();
    println!("{:?}", b);
    for i in 0..20 {
        if i == 17 || i == 18 || i == 19 {
            assert_eq!(true, b.is_set(i));
        } else {
            assert_eq!(false, b.is_set(i));
        }
    }
}

#[test]
fn decode_padded_small() {
    let data = b"\x0e"; // 0000 1110
    let mut d = Decoder::new(data);
    d.read(4).unwrap();
    let b = BitString::from_aper(
        &mut d,
        Constraints {
            value: None,
            size: Some(Constraint::new(None, Some(4))),
        },
    )
    .unwrap();
    println!("{:?}", b);
    for i in 0..4 {
        if i == 1 || i == 2 || i == 3 {
            assert_eq!(true, b.is_set(i));
        } else {
            assert_eq!(false, b.is_set(i));
        }
    }
}

#[test]
fn decode_unpadded() {
    let data = b"\x00\x00\xe0";
    let mut d = Decoder::new(data);
    let b = BitString::from_aper(
        &mut d,
        Constraints {
            value: None,
            size: Some(Constraint::new(None, Some(24))),
        },
    )
    .unwrap();
    println!("{:?}", b);
    for i in 0..24 {
        if i == 5 || i == 6 || i == 7 {
            assert_eq!(true, b.is_set(i));
        } else {
            assert_eq!(false, b.is_set(i));
        }
    }
}

#[test]
fn encode_padded_small() {
    let bs = BitString::with_bytes_and_len(&vec![0x0e as u8], 4);
    let target: Vec<u8> = vec![0xe0];
    assert_eq!(
        target,
        *bs.to_aper(Constraints {
            value: None,
            size: Some(Constraint::new(None, Some(4))),
        })
        .unwrap()
        .bytes()
    );
}
