use crate::*;

#[test]
fn can_de_ext() {
    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct _ExtStruct((i8, Value));
    let mut enc = Vec::new();
    ValueRef::Ext(-42, b"hello").encode_sync(&mut enc).unwrap();
    let dec: _ExtStruct = from_ref(enc.as_slice()).unwrap();
    assert_eq!(
        _ExtStruct((-42, Value::Bin(b"hello".to_vec().into_boxed_slice()))),
        dec,
    );
}

#[test]
fn can_de_value_ext() {
    let mut enc = Vec::new();
    ValueRef::Ext(-42, b"hello").encode_sync(&mut enc).unwrap();
    let dec: ValueRef = from_ref(enc.as_slice()).unwrap();
    assert_eq!(ValueRef::Ext(-42, b"hello"), dec,);
}

/*
#[test]
fn can_de_ref() {
    let mut enc = Vec::new();
    ValueRef::Arr(vec![
        ValueRef::Str("hello".into()),
        ValueRef::Ext(-42, b"hello"),
    ]).encode_sync(&mut enc).unwrap();
    let r: ValueRef = from_ref(enc.as_slice()).unwrap();
    assert_eq!(ValueRef::Arr(vec![
        ValueRef::Str("hello".into()),
        ValueRef::Ext(-42, b"hello"),
    ]), r);
}
*/
