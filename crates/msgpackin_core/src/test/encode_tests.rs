use crate::encode::*;

#[test]
fn it_works() {
    fn check_len(len: usize, bytes: &[u8]) {
        assert_eq!(len, bytes.len());
    }

    let mut enc = Encoder::new();
    check_len(2, &enc.enc_ext_len(1, 0));
}
