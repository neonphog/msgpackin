mod decode_tests;
mod encode_tests;

use crate::decode::*;
use crate::encode::*;

#[test]
fn test_lib_doc_demo() {
    const S1: &str = "hello ";
    const S2: &str = "world!";

    // this is a no_std, no alloc crate, everything must be on the stack
    let mut buf: [u8; 15] = [0; 15];
    let mut cur = 0;

    {
        // small helper closure to write consecutive data to our buffer
        let mut write = |data: &[u8]| {
            buf[cur..cur + data.len()].copy_from_slice(data);
            cur += data.len();
        };

        // construct a new encoder
        let mut enc = Encoder::new();

        // write the bytes marking an array msgpack type of length 2
        write(&enc.enc_arr_len(2));

        // write the length of the string we are trying to encode
        write(&enc.enc_str_len(S1.as_bytes().len() as u32));

        // write the actual string bytes
        write(S1.as_bytes());

        // write the second string length
        write(&enc.enc_str_len(S2.as_bytes().len() as u32));

        // write the second string bytes
        write(S2.as_bytes());
    }

    // make sure we wrote the correct bytes to the buffer
    assert_eq!(
        &[
            146, 166, 104, 101, 108, 108, 111, 32, 166, 119, 111, 114, 108,
            100, 33
        ],
        &buf[..]
    );

    let mut dec = Decoder::new();
    let mut iter = dec.parse(&buf);

    assert_eq!(Some(Token::Len(LenType::Arr, 2)), iter.next());
    assert_eq!(Some(Token::Len(LenType::Str, 6)), iter.next());
    assert_eq!(Some(Token::Bin(S1.as_bytes())), iter.next());
    assert_eq!(Some(Token::Len(LenType::Str, 6)), iter.next());
    assert_eq!(Some(Token::Bin(S2.as_bytes())), iter.next());
    assert_eq!(None, iter.next());
}
