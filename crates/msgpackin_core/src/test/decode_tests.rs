use crate::decode::*;

const MAX_TOKS: usize = 32;

fn exec_decode_tests<'a>(
    fixture: &'a [u8],
) -> [Option<Token<'a>>; MAX_TOKS] {
    let mut out1 = [None; MAX_TOKS];
    let mut out1_cursor = 0;

    // first, the straight-forward test
    {
        let mut dec = Decoder::new();
        let mut iter = dec.parse(fixture);
        while let Some(token) = iter.next() {
            out1[out1_cursor] = Some(token);
            out1_cursor += 1;
        }
    }

    let mut out2 = [None; MAX_TOKS];
    let mut out2_cursor = 0;

    // now the byte-by-byte test
    {
        let mut dec = Decoder::new();
        let mut start_buf = None;
        for c in 0..fixture.len() {
            let mut iter = dec.parse(&fixture[c..=c]);
            while let Some(token) = iter.next() {
                use Token::*;
                match token {
                    BinCont(_, _) => {
                        if start_buf.is_none() {
                            start_buf = Some(c);
                        }
                    }
                    Bin(_) => {
                        if let Some(start) = start_buf {
                            out2[out2_cursor] = Some(Bin(&fixture[start..=c]));
                            out2_cursor += 1;
                        } else {
                            out2[out2_cursor] = Some(token);
                            out2_cursor += 1;
                        }
                    }
                    token => {
                        out2[out2_cursor] = Some(token);
                        out2_cursor += 1;
                    }
                }
            }
        }
    }

    // make sure the results are the same
    assert_eq!(out1, out2);

    out1
}

const FIXTURE_NIL: &[&[u8]] = &[&[0xc0]];

#[test]
fn decode_nil() {
    for fixture in FIXTURE_NIL {
        let res = exec_decode_tests(fixture);
        assert!(
            matches!(res[0], Some(Token::Nil)),
            "expected Some(Nil), got: {:?}",
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

const FIXTURE_BOOL: &[(bool, &[u8])] = &[(false, &[0xc2]), (true, &[0xc3])];

#[test]
fn decode_bool() {
    for fixture in FIXTURE_BOOL {
        let res = exec_decode_tests(fixture.1);
        assert!(
            matches!(res[0], Some(Token::Bool(b)) if b == fixture.0),
            "expected Some(Bool({})), got: {:?}",
            fixture.0,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

const FIXTURE_BIN: &[(&[u8], &[u8])] = &[
    (&[], &[0xc4, 0x00]),
    (&[], &[0xc5, 0x00, 0x00]),
    (&[], &[0xc6, 0x00, 0x00, 0x00, 0x00]),
    (&[0x01], &[0xc4, 0x01, 0x01]),
    (&[0x01], &[0xc5, 0x00, 0x01, 0x01]),
    (&[0x01], &[0xc6, 0x00, 0x00, 0x00, 0x01, 0x01]),
    (&[0x00, 0xff], &[0xc4, 0x02, 0x00, 0xff]),
    (&[0x00, 0xff], &[0xc5, 0x00, 0x02, 0x00, 0xff]),
    (&[0x00, 0xff], &[0xc6, 0x00, 0x00, 0x00, 0x02, 0x00, 0xff]),
];

#[test]
fn decode_bin() {
    for fixture in FIXTURE_BIN {
        let res = exec_decode_tests(fixture.1);
        assert!(
            matches!(res[0], Some(Token::Len(LenType::Bin, len)) if len == fixture.0.len() as u32),
            "expect Some(Len(Bin, {})), got: {:?}",
            fixture.0.len(),
            res[0],
        );
        assert!(
            matches!(res[1], Some(Token::Bin(data)) if data == fixture.0),
            "expect Some(Bin({:?})), got: {:?}",
            fixture.0,
            res[1],
        );
        assert!(matches!(res[2], None), "expected None, got: Some(_)");
    }
}

const FIXTURE_EXT: &[(i8, &[u8], &[u8])] = &[
    (1, &[0x10], &[0xd4, 0x01, 0x10]),
    (2, &[0x20, 0x21], &[0xd5, 0x02, 0x20, 0x21]),
    (
        3,
        &[0x30, 0x31, 0x32, 0x33],
        &[0xd6, 0x03, 0x30, 0x31, 0x32, 0x33],
    ),
    (
        4,
        &[0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47],
        &[0xd7, 0x04, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47],
    ),
    (
        5,
        &[
            0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a,
            0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
        ],
        &[
            0xd8, 0x05, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
            0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
        ],
    ),
    (6, &[], &[0xc7, 0x00, 0x06]),
    (6, &[], &[0xc8, 0x00, 0x00, 0x06]),
    (6, &[], &[0xc9, 0x00, 0x00, 0x00, 0x00, 0x06]),
    (
        7,
        &[0x70, 0x71, 0x72],
        &[0xc7, 0x03, 0x07, 0x70, 0x71, 0x72],
    ),
    (
        7,
        &[0x70, 0x71, 0x72],
        &[0xc8, 0x00, 0x03, 0x07, 0x70, 0x71, 0x72],
    ),
    (
        7,
        &[0x70, 0x71, 0x72],
        &[0xc9, 0x00, 0x00, 0x00, 0x03, 0x07, 0x70, 0x71, 0x72],
    ),
];

#[test]
fn decode_ext() {
    for fixture in FIXTURE_EXT {
        let res = exec_decode_tests(fixture.2);
        assert!(
            matches!(res[0], Some(Token::Len(LenType::Ext(t), len)) if t == fixture.0 && len == fixture.1.len() as u32),
            "expect Some(Len(Ext({}), {})), got: {:?}",
            fixture.0,
            fixture.1.len(),
            res[0],
        );
        assert!(
            matches!(res[1], Some(Token::Bin(data)) if data == fixture.1),
            "expect Some(Bin({:?})), got: {:?}",
            fixture.1,
            res[1],
        );
        assert!(matches!(res[2], None), "expected None, got: Some(_)");
    }
}

const FIXTURE_STR: &[(&str, &[u8])] = &[
    ("", &[0xa0]),
    ("", &[0xd9, 0x00]),
    ("", &[0xda, 0x00, 0x00]),
    ("", &[0xdb, 0x00, 0x00, 0x00, 0x00]),
    ("a", &[0xa1, 0x61]),
    ("a", &[0xd9, 0x01, 0x61]),
    ("a", &[0xda, 0x00, 0x01, 0x61]),
    ("a", &[0xdb, 0x00, 0x00, 0x00, 0x01, 0x61]),
    (
        "1234567890123456789012345678901",
        &[
            0xbf, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31,
            0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31,
        ],
    ),
    (
        "1234567890123456789012345678901",
        &[
            0xd9, 0x1f, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31,
        ],
    ),
    (
        "1234567890123456789012345678901",
        &[
            0xda, 0x00, 0x1f, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38,
            0x39, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
            0x31,
        ],
    ),
    (
        "1234567890123456789012345678901",
        &[
            0xdb, 0x00, 0x00, 0x00, 0x1f, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36,
            0x37, 0x38, 0x39, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
            0x38, 0x39, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38,
            0x39, 0x30, 0x31,
        ],
    ),
    (
        "Кириллица",
        &[
            0xb2, 0xd0, 0x9a, 0xd0, 0xb8, 0xd1, 0x80, 0xd0, 0xb8, 0xd0, 0xbb,
            0xd0, 0xbb, 0xd0, 0xb8, 0xd1, 0x86, 0xd0, 0xb0,
        ],
    ),
    (
        "Кириллица",
        &[
            0xd9, 0x12, 0xd0, 0x9a, 0xd0, 0xb8, 0xd1, 0x80, 0xd0, 0xb8, 0xd0,
            0xbb, 0xd0, 0xbb, 0xd0, 0xb8, 0xd1, 0x86, 0xd0, 0xb0,
        ],
    ),
    (
        "ひらがな",
        &[
            0xac, 0xe3, 0x81, 0xb2, 0xe3, 0x82, 0x89, 0xe3, 0x81, 0x8c, 0xe3,
            0x81, 0xaa,
        ],
    ),
    (
        "ひらがな",
        &[
            0xd9, 0x0c, 0xe3, 0x81, 0xb2, 0xe3, 0x82, 0x89, 0xe3, 0x81, 0x8c,
            0xe3, 0x81, 0xaa,
        ],
    ),
    ("❤", &[0xa3, 0xe2, 0x9d, 0xa4]),
    ("❤", &[0xd9, 0x03, 0xe2, 0x9d, 0xa4]),
];

#[test]
fn decode_str() {
    for fixture in FIXTURE_STR {
        let res = exec_decode_tests(fixture.1);
        assert!(
            matches!(res[0], Some(Token::Len(LenType::Str, len)) if len == fixture.0.len() as u32),
            "expect Some(Len(Str, {})), got: {:?}",
            fixture.0.len(),
            res[0],
        );
        assert!(
            matches!(res[1], Some(Token::Bin(data)) if core::str::from_utf8(data).unwrap() == fixture.0),
            "expect Some(Bin({:?})), got: {:?}",
            fixture.0.as_bytes(),
            res[1],
        );
        assert!(matches!(res[2], None), "expected None, got: Some(_)");
    }
}

const FIXTURE_FLOAT_32: &[(f32, &[u8])] = &[
    (0.5, &[0xca, 0x3f, 0x00, 0x00, 0x00]),
    (-0.5, &[0xca, 0xbf, 0x00, 0x00, 0x00]),
];

#[test]
fn decode_float_32() {
    for fixture in FIXTURE_FLOAT_32 {
        let res = exec_decode_tests(fixture.1);
        assert!(
            matches!(res[0], Some(Token::F32(f)) if f == fixture.0),
            "expect Some(F32({})), got: {:?}",
            fixture.0,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

const FIXTURE_FLOAT_64: &[(f64, &[u8])] = &[
    (0.5, &[0xcb, 0x3f, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    (
        -0.5,
        &[0xcb, 0xbf, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    ),
];

#[test]
fn decode_float_64() {
    for fixture in FIXTURE_FLOAT_64 {
        let res = exec_decode_tests(fixture.1);
        assert!(
            matches!(res[0], Some(Token::F64(f)) if f == fixture.0),
            "expect Some(F64({})), got: {:?}",
            fixture.0,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_u8_pos_fixint() {
    for u in 0..127 {
        let buf = [u];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::U8(ru)) if ru == u),
            "expect Some(U8({})), got: {:?}",
            u,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_i8_neg_fixint() {
    for i in -32..0 {
        let buf = [i as u8];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::I8(ri)) if ri == i),
            "expect Some(I8({})), got: {:?}",
            i,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_u8() {
    for u in u8::MIN..=u8::MAX {
        let buf = [0xcc, u];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::U8(ru)) if ru == u),
            "{:?} expect Some(U8({})), got: {:?}",
            buf,
            u,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_u16() {
    for u in [u16::MIN, u16::MAX, 1, u16::MAX / 2, u16::MAX - 1] {
        let bytes = u.to_be_bytes();
        let buf = [0xcd, bytes[0], bytes[1]];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::U16(ru)) if ru == u),
            "{:?} expect Some(U16({})), got: {:?}",
            buf,
            u,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_u32() {
    for u in [u32::MIN, u32::MAX, 1, u32::MAX / 2, u32::MAX - 1] {
        let bytes = u.to_be_bytes();
        let buf = [0xce, bytes[0], bytes[1], bytes[2], bytes[3]];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::U32(ru)) if ru == u),
            "{:?} expect Some(U32({})), got: {:?}",
            buf,
            u,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_u64() {
    for u in [u64::MIN, u64::MAX, 1, u64::MAX / 2, u64::MAX - 1] {
        let bytes = u.to_be_bytes();
        let buf = [
            0xcf, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            bytes[6], bytes[7],
        ];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::U64(ru)) if ru == u),
            "{:?} expect Some(U64({})), got: {:?}",
            buf,
            u,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_i8() {
    for i in i8::MIN..=i8::MAX {
        let buf = [0xd0, i as u8];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::I8(ri)) if ri == i),
            "{:?} expect Some(I8({})), got: {:?}",
            buf,
            i,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_i16() {
    for i in [i16::MIN, i16::MAX, -1, 0, 1, i16::MIN + 1, i16::MAX - 1] {
        let bytes = i.to_be_bytes();
        let buf = [0xd1, bytes[0], bytes[1]];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::I16(ri)) if ri == i),
            "{:?} expect Some(I16({})), got: {:?}",
            buf,
            i,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_i32() {
    for i in [i32::MIN, i32::MAX, -1, 0, 1, i32::MIN + 1, i32::MAX - 1] {
        let bytes = i.to_be_bytes();
        let buf = [0xd2, bytes[0], bytes[1], bytes[2], bytes[3]];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::I32(ri)) if ri == i),
            "{:?} expect Some(I32({})), got: {:?}",
            buf,
            i,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

#[test]
fn decode_i64() {
    for i in [i64::MIN, i64::MAX, -1, 0, 1, i64::MIN + 1, i64::MAX - 1] {
        let bytes = i.to_be_bytes();
        let buf = [
            0xd3, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            bytes[6], bytes[7],
        ];
        let res = exec_decode_tests(&buf);
        assert!(
            matches!(res[0], Some(Token::I64(ri)) if ri == i),
            "{:?} expect Some(I64({})), got: {:?}",
            buf,
            i,
            res[0],
        );
        assert!(matches!(res[1], None), "expected None, got: Some(_)");
    }
}

const FIXTURE_ARR: &[(&[&str], &[u8])] = &[
    (&[], &[0x90]),
    (&[], &[0xdc, 0x00, 0x00]),
    (&[], &[0xdd, 0x00, 0x00, 0x00, 0x00]),
    (&["a"], &[0x91, 0xa1, 0x61]),
    (&["a"], &[0xdc, 0x00, 0x01, 0xa1, 0x61]),
    (&["a"], &[0xdd, 0x00, 0x00, 0x00, 0x01, 0xa1, 0x61]),
    (
        &[
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "1", "2", "3",
            "4", "5",
        ],
        &[
            0x9f, 0xa1, 0x31, 0xa1, 0x32, 0xa1, 0x33, 0xa1, 0x34, 0xa1, 0x35,
            0xa1, 0x36, 0xa1, 0x37, 0xa1, 0x38, 0xa1, 0x39, 0xa1, 0x30, 0xa1,
            0x31, 0xa1, 0x32, 0xa1, 0x33, 0xa1, 0x34, 0xa1, 0x35,
        ],
    ),
];

#[test]
fn decode_arr() {
    for fixture in FIXTURE_ARR {
        let res = exec_decode_tests(fixture.1);
        assert!(
            matches!(res[0], Some(Token::Len(LenType::Arr, len)) if len == fixture.0.len() as u32),
            "expect Some(Len(Arr, {})), got: {:?}",
            fixture.0.len(),
            res[0],
        );
        let mut idx = 1;
        for s in fixture.0 {
            assert!(
                matches!(res[idx], Some(Token::Len(LenType::Str, len)) if len == s.len() as u32),
                "expect Some(Len(Str, {})), got: {:?}",
                s.len(),
                res[idx],
            );
            idx += 1;
            assert!(
                matches!(res[idx], Some(Token::Bin(data)) if core::str::from_utf8(data).unwrap() == *s),
                "expect Some(Bin({:?})), got: {:?}",
                s.as_bytes(),
                res[idx],
            );
            idx += 1;
        }
        assert!(matches!(res[idx], None), "expected None, got: Some(_)");
    }
}

const FIXTURE_MAP: &[(&[&str], &[u8])] = &[
    (&[], &[0x80]),
    (&[], &[0xde, 0x00, 0x00]),
    (&[], &[0xdf, 0x00, 0x00, 0x00, 0x00]),
    (&["a", "b"], &[0x81, 0xa1, 0x61, 0xa1, 0x62]),
    (&["a", "b"], &[0xde, 0x00, 0x01, 0xa1, 0x61, 0xa1, 0x62]),
    (&["a", "b"], &[0xdf, 0x00, 0x00, 0x00, 0x01, 0xa1, 0x61, 0xa1, 0x62]),
];

#[test]
fn decode_map() {
    for fixture in FIXTURE_MAP {
        let res = exec_decode_tests(fixture.1);
        assert!(
            matches!(res[0], Some(Token::Len(LenType::Map, len)) if len == fixture.0.len() as u32 / 2),
            "expect Some(Len(Map, {})), got: {:?}",
            fixture.0.len(),
            res[0],
        );
        let mut idx = 1;
        for s in fixture.0 {
            assert!(
                matches!(res[idx], Some(Token::Len(LenType::Str, len)) if len == s.len() as u32),
                "expect Some(Len(Str, {})), got: {:?}",
                s.len(),
                res[idx],
            );
            idx += 1;
            assert!(
                matches!(res[idx], Some(Token::Bin(data)) if core::str::from_utf8(data).unwrap() == *s),
                "expect Some(Bin({:?})), got: {:?}",
                s.as_bytes(),
                res[idx],
            );
            idx += 1;
        }
        assert!(matches!(res[idx], None), "expected None, got: Some(_)");
    }
}
