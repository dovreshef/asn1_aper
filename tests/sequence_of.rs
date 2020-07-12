use asn1_aper::{
    APerDecode,
    APerEncode,
    BitString,
    Constraint,
    Constraints,
    Decoder,
    UNCONSTRAINED,
};
use std::i32;

#[test]
fn encode_sequence_of_u8() {
    let v: Vec<u8> = vec![0x46, 0x4f, 0x4f];
    let target: Vec<u8> = vec![0x03, 0x46, 0x4f, 0x4f];
    assert_eq!(target, *v.to_aper(UNCONSTRAINED).unwrap().bytes());
}

#[test]
fn decode_sequence_of_u8() {
    let data = b"\x03\x46\x4f\x4f";
    let mut d = Decoder::new(data);
    let v = Vec::<u8>::from_aper(
        &mut d,
        Constraints {
            value: None,
            size: Some(Constraint::new(None, Some(3))),
        },
    )
    .unwrap();
    assert_eq!(v.len(), data.len() - 1);
    for i in 0..v.len() {
        assert_eq!(v[i], data[i + 1]);
    }
}

#[test]
fn encode_sequence_of_u16() {
    let v: Vec<u16> = vec![0xfe46, 0xc04f, 0x884f];
    let target: Vec<u8> = vec![0x3, 0xfe, 0x46, 0xc0, 0x4f, 0x88, 0x4f];
    assert_eq!(target, *v.to_aper(UNCONSTRAINED).unwrap().bytes());
}

#[test]
fn decode_sequence_of_u16() {
    let data = b"\x03\xfe\x46\xc0\x4f\x88\x4f";
    let target = vec![0xfe46 as u16, 0xc04f as u16, 0x884f as u16];
    let mut d = Decoder::new(data);
    let v = Vec::<u16>::from_aper(
        &mut d,
        Constraints {
            value: None,
            size: Some(Constraint::new(None, Some(3))),
        },
    )
    .unwrap();
    assert_eq!(v.len(), target.len());
    for i in 0..v.len() {
        assert_eq!(v[i], target[i]);
    }
}

#[test]
fn encode_sequence_of_i32() {
    let v: Vec<i32> = vec![i32::MIN, i32::MIN + 1, i32::MIN + 2];
    let target: Vec<u8> = vec![
        0x3, 0x04, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x04, 0x00, 0x00, 0x00, 0x02,
    ];
    assert_eq!(target, *v.to_aper(UNCONSTRAINED).unwrap().bytes());
}

#[test]
fn decode_sequence_of_i32() {
    let data = b"\x03\x04\x00\x00\x00\x00\x04\x00\x00\x00\x01\x04\x00\x00\x00\x02";
    let mut target = Vec::new();
    use std::i32;
    for i in 0..3 {
        target.push(i32::MIN + i);
    }
    let mut d = Decoder::new(data);
    let v = Vec::<i32>::from_aper(
        &mut d,
        Constraints {
            value: None,
            size: Some(Constraint::new(None, Some(3))),
        },
    )
    .unwrap();
    assert_eq!(v.len(), target.len());
    for i in 0..v.len() {
        assert_eq!(v[i], target[i]);
    }
}

#[test]
fn decode_sequence_of_short_bit_string() {
    let data = b"\x02\xee";
    let mut d = Decoder::new(data);
    let v = Vec::<BitString>::from_aper(
        &mut d,
        Constraints {
            // here the "value" constraint is a constraint on the size of each element
            value: Some(Constraint::new(None, Some(4))),
            // "size" behaves normally
            size: Some(Constraint::new(None, Some(2))),
        },
    )
    .unwrap();
    assert_eq!(v.len(), 2);

    for i in 0..v.len() {
        for j in 0..4 {
            if j == 1 || j == 2 || j == 3 {
                assert_eq!(true, v[i].is_set(j));
            } else {
                assert_eq!(false, v[i].is_set(j));
            }
        }
    }
}

#[test]
fn decode_sequence_of_long_bit_string() {
    let data = b"\x02\x00\x00\xe0\x00\x00\xe0";
    let mut d = Decoder::new(data);
    let v = Vec::<BitString>::from_aper(
        &mut d,
        Constraints {
            // here the "value" constraint is a constraint on the size of each element
            value: Some(Constraint::new(None, Some(24))),
            // "size" behaves normally
            size: Some(Constraint::new(None, Some(2))),
        },
    )
    .unwrap();
    assert_eq!(v.len(), 2);

    for i in 0..v.len() {
        for j in 0..20 {
            if j == 5 || j == 6 || j == 7 {
                assert_eq!(true, v[i].is_set(j));
            } else {
                assert_eq!(false, v[i].is_set(j));
            }
        }
    }
}
