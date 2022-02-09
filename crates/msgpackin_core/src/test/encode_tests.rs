use crate::decode::*;
use crate::encode::*;

const MAX_TOKS: usize = 32;
const MAX_ENC_LEN: usize = 1024;

fn check_encode_test(expect: &[u8], result: &[u8]) {
    fn parse<'a, 'b>(t: &'a mut [Option<Token<'b>>; MAX_TOKS], d: &'b [u8]) {
        let mut dec = Decoder::new();
        let mut iter = dec.parse(d);
        let mut cur = 0;
        while let Some(token) = iter.next() {
            t[cur] = Some(token);
            cur += 1;
        }
    }

    let mut expect_toks = [None; MAX_TOKS];
    let mut result_toks = [None; MAX_TOKS];

    parse(&mut expect_toks, expect);
    parse(&mut result_toks, result);

    // catch more descriptive errors first
    assert_eq!(expect_toks, result_toks);

    // but we also want the bytes to exactly match
    assert_eq!(expect, result);
}

struct TestBuf {
    buf: [u8; MAX_ENC_LEN],
    cur: usize,
}

impl TestBuf {
    fn new() -> Self {
        Self {
            buf: [0; MAX_ENC_LEN],
            cur: 0,
        }
    }

    fn put(&mut self, d: &[u8]) {
        self.buf[self.cur..self.cur + d.len()].copy_from_slice(d);
        self.cur += d.len()
    }

    fn get(&self) -> &[u8] {
        &self.buf[..self.cur]
    }
}

#[test]
fn encode_nil() {
    let mut enc = Encoder::new();
    let mut buf = TestBuf::new();
    buf.put(&enc.enc_nil());
    check_encode_test(&[0xc0], buf.get());
}

#[test]
fn encode_true() {
    let mut enc = Encoder::new();
    let mut buf = TestBuf::new();
    buf.put(&enc.enc_bool(true));
    check_encode_test(&[0xc3], buf.get());
}

#[test]
fn encode_false() {
    let mut enc = Encoder::new();
    let mut buf = TestBuf::new();
    buf.put(&enc.enc_bool(false));
    check_encode_test(&[0xc2], buf.get());
}

#[test]
fn encode_f32() {
    let mut enc = Encoder::new();
    let mut buf = TestBuf::new();
    buf.put(&enc.enc_num(0.5_f32));
    check_encode_test(&[0xca, 0x3f, 0x00, 0x00, 0x00], buf.get());
}

#[test]
fn encode_f64() {
    let mut enc = Encoder::new();
    let mut buf = TestBuf::new();
    buf.put(&enc.enc_num(3.141592653589793_f64));
    check_encode_test(
        &[0xcb, 0x40, 0x09, 0x21, 0xfb, 0x54, 0x44, 0x2d, 0x18],
        buf.get(),
    );
}

#[test]
fn encode_pos_fixint() {
    for u in 0..127 {
        let expect = [u];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(u));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_neg_fixint() {
    for i in -31..0 {
        let expect = [i as u8];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(i));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_u8() {
    for u in 128..=u8::MAX {
        let expect = [0xcc, u];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(u));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_i8() {
    for i in i8::MIN..=-32 {
        let expect = [0xd0, i as u8];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(i));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_u16() {
    for u in [u8::MAX as u16 + 1, u16::MAX / 2, u16::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xcd, bytes[0], bytes[1]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(u));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_i16() {
    for i in [i8::MIN as i16 - 1, i16::MIN / 2, i16::MIN] {
        let bytes = i.to_be_bytes();
        let expect = [0xd1, bytes[0], bytes[1]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(i));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_u32() {
    for u in [u16::MAX as u32 + 1, u32::MAX / 2, u32::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xce, bytes[0], bytes[1], bytes[2], bytes[3]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(u));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_i32() {
    for i in [i16::MIN as i32 - 1, i32::MIN / 2, i32::MIN] {
        let bytes = i.to_be_bytes();
        let expect = [0xd2, bytes[0], bytes[1], bytes[2], bytes[3]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(i));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_u64() {
    for u in [u32::MAX as u64 + 1, u64::MAX / 2, u64::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [
            0xcf, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            bytes[6], bytes[7],
        ];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(u));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_i64() {
    for i in [i32::MIN as i64 - 1, i64::MIN / 2, i64::MIN] {
        let bytes = i.to_be_bytes();
        let expect = [
            0xd3, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            bytes[6], bytes[7],
        ];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_num(i));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_fixstr() {
    for u in 0_u8..=31 {
        let expect = [0xa0 + u];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_str_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_str8() {
    for u in 32..=u8::MAX {
        let expect = [0xd9, u];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_str_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_str16() {
    for u in [u8::MAX as u16 + 1, u16::MAX / 2, u16::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xda, bytes[0], bytes[1]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_str_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_str32() {
    for u in [u16::MAX as u32 + 1, u32::MAX / 2, u32::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xdb, bytes[0], bytes[1], bytes[2], bytes[3]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_str_len(u));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_bin8() {
    for u in u8::MIN..=u8::MAX {
        let expect = [0xc4, u];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_bin_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_bin16() {
    for u in [u8::MAX as u16 + 1, u16::MAX / 2, u16::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xc5, bytes[0], bytes[1]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_bin_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_bin32() {
    for u in [u16::MAX as u32 + 1, u32::MAX / 2, u32::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xc6, bytes[0], bytes[1], bytes[2], bytes[3]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_bin_len(u));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_fixext() {
    for (t, l, expect_bytes) in [
        (1, 1, &[0xd4, 0x01]),
        (2, 2, &[0xd5, 0x02]),
        (3, 4, &[0xd6, 0x03]),
        (4, 8, &[0xd7, 0x04]),
        (5, 16, &[0xd8, 0x05]),
    ] {
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_ext_len(l, t));
        check_encode_test(expect_bytes, buf.get());
    }
}

#[test]
fn encode_ext8() {
    for u in [3, 5, 7, 9, 15, 17, 128, 255] {
        let expect = [0xc7, u, 0x06];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_ext_len(u as u32, 6));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_ext16() {
    for u in [u8::MAX as u16 + 1, u16::MAX / 2, u16::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xc8, bytes[0], bytes[1], 0x07];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_ext_len(u as u32, 7));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_ext32() {
    for u in [u16::MAX as u32 + 1, u32::MAX / 2, u32::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xc9, bytes[0], bytes[1], bytes[2], bytes[3], 0x08];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_ext_len(u, 8));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_fixarr() {
    for u in 0_u8..=15 {
        let expect = [0x90 + u];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_arr_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_arr16() {
    for u in [16_u16, u16::MAX / 2, u16::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xdc, bytes[0], bytes[1]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_arr_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_arr32() {
    for u in [u16::MAX as u32 + 1, u32::MAX / 2, u32::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xdd, bytes[0], bytes[1], bytes[2], bytes[3]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_arr_len(u));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_fixmap() {
    for u in 0_u8..=15 {
        let expect = [0x80 + u];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_map_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_map16() {
    for u in [16_u16, u16::MAX / 2, u16::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xde, bytes[0], bytes[1]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_map_len(u as u32));
        check_encode_test(&expect, buf.get());
    }
}

#[test]
fn encode_map32() {
    for u in [u16::MAX as u32 + 1, u32::MAX / 2, u32::MAX] {
        let bytes = u.to_be_bytes();
        let expect = [0xdf, bytes[0], bytes[1], bytes[2], bytes[3]];
        let mut enc = Encoder::new();
        let mut buf = TestBuf::new();
        buf.put(&enc.enc_map_len(u));
        check_encode_test(&expect, buf.get());
    }
}
