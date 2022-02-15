use crate::*;

#[test]
fn no_std_encode_decode_demo() {
    let expect = Value::Map(vec![
        ("nil".into(), ().into()),
        ("bool".into(), true.into()),
        ("int".into(), (-42_i8).into()),
        ("bigInt".into(), u64::MAX.into()),
        ("float".into(), 3.141592653589793_f64.into()),
        ("str".into(), "hello".into()),
        ("ext".into(), Value::Ext(-42, b"ext-data".to_vec().into())),
        ("arr".into(), Value::Arr(vec!["one".into(), "two".into()])),
    ]);
    let encoded = expect.to_bytes().unwrap();
    let decoded = ValueRef::from_ref(&encoded).unwrap();
    assert_eq!(expect, decoded);
}
