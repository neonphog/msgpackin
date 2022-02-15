use crate::*;

#[test]
fn serde_encode_decode_demo() {
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct X {
        pub nil: (),
        pub bool_: bool,
        pub int: i8,
        pub big_int: u64,
        pub float: f64,
        pub str_: String,
        pub arr: Vec<String>,
    }

    let expect = X {
        nil: (),
        bool_: true,
        int: -42,
        big_int: u64::MAX,
        float: 3.141592653589793,
        str_: "hello".into(),
        arr: vec!["one".into(), "two".into()],
    };

    let encoded = to_bytes(&expect).unwrap();
    let decoded: X = from_sync(encoded.as_slice()).unwrap();
    assert_eq!(expect, decoded);
}

#[test]
fn can_ser_ext() {
    #[derive(serde::Serialize)]
    struct _ExtStruct((i8, ValueRef<'static>));
    let e = _ExtStruct((-42, ValueRef::Bin(b"hello")));
    let enc = to_bytes(&e).unwrap();
    let dec = ValueRef::from_ref(enc.as_slice()).unwrap();
    assert_eq!(Value::Ext(-42, b"hello".to_vec().into_boxed_slice()), dec);
}

#[test]
fn can_ser_value() {
    let enc = to_bytes(&ValueRef::Arr(vec![
        ValueRef::Ext(-42, b"hello"),
        ValueRef::Map(vec![(
            ValueRef::Str("test".into()),
            ValueRef::Str("val".into()),
        )]),
    ]))
    .unwrap();
    let dec = ValueRef::from_ref(enc.as_slice()).unwrap();
    assert_eq!(
        ValueRef::Arr(vec![
            ValueRef::Ext(-42, b"hello"),
            ValueRef::Map(vec![(
                ValueRef::Str("test".into()),
                ValueRef::Str("val".into())
            ),]),
        ]),
        dec
    );
}

#[test]
fn can_ser_tuple() {
    let enc = to_bytes(&("hello", 42, <Option<i8>>::None, true)).unwrap();
    let dec = ValueRef::from_ref(enc.as_slice()).unwrap();
    assert_eq!(
        Value::Arr(vec![
            Value::Str("hello".into()),
            Value::Num(42.into()),
            Value::Nil,
            Value::Bool(true),
        ]),
        dec
    );
}

#[test]
fn can_ser_struct() {
    #[derive(serde::Serialize)]
    struct X {
        s: &'static str,
        u: usize,
        n: Option<i8>,
        b: bool,
    }
    let enc = to_bytes(&X {
        s: "hello",
        u: 42,
        n: None,
        b: true,
    })
    .unwrap();
    let dec = ValueRef::from_ref(enc.as_slice()).unwrap();
    assert_eq!(
        Value::Map(vec![
            (Value::Str("s".into()), Value::Str("hello".into())),
            (Value::Str("u".into()), Value::Num(42.into())),
            (Value::Str("n".into()), Value::Nil),
            (Value::Str("b".into()), Value::Bool(true)),
        ]),
        dec
    );
}

#[test]
fn can_ser_flatten() {
    #[derive(serde::Serialize)]
    struct X {
        a: i8,
        b: i8,
    }

    #[derive(serde::Serialize)]
    struct Y {
        a: i8,

        #[serde(flatten)]
        b: X,
    }

    let enc = to_bytes(&Y {
        a: 42,
        b: X { a: 0, b: 0 },
    })
    .unwrap();

    let dec = ValueRef::from_ref(enc.as_slice()).unwrap();

    // note the two 'a's in this map... is that a good thing??
    assert_eq!(
        Value::Map(vec![
            (Value::Str("a".into()), Value::Num(42.into())),
            (Value::Str("a".into()), Value::Num(0.into())),
            (Value::Str("b".into()), Value::Num(0.into())),
        ]),
        dec
    );
}

#[test]
fn can_ser_enum() {
    #[derive(serde::Serialize)]
    enum X {
        Unit,
        NoUple(),
        OneUple(usize),
        TwoUple(usize, usize),
        NoStruct {},
        OneStruct { a: usize },
        TwoStruct { a: usize, b: usize },
    }

    let enc = to_bytes(&(
        X::Unit,
        X::NoUple(),
        X::OneUple(42),
        X::TwoUple(42, 42),
        X::NoStruct {},
        X::OneStruct { a: 42 },
        X::TwoStruct { a: 42, b: 42 },
    ))
    .unwrap();

    let dec = ValueRef::from_ref(enc.as_slice()).unwrap();

    assert_eq!(
        Value::Arr(vec![
            Value::Str("Unit".into()),
            Value::Map(
                vec![(Value::Str("NoUple".into()), Value::Arr(vec![])),]
            ),
            Value::Map(vec![(
                Value::Str("OneUple".into()),
                // note this oddity of no array here
                Value::Num(42.into()),
            )]),
            Value::Map(vec![(
                Value::Str("TwoUple".into()),
                Value::Arr(vec![Value::Num(42.into()), Value::Num(42.into())]),
            )]),
            Value::Map(vec![(
                Value::Str("NoStruct".into()),
                Value::Map(vec![]),
            )]),
            Value::Map(vec![(
                Value::Str("OneStruct".into()),
                Value::Map(vec![(
                    Value::Str("a".into()),
                    Value::Num(42.into())
                ),]),
            )]),
            Value::Map(vec![(
                Value::Str("TwoStruct".into()),
                Value::Map(vec![
                    (Value::Str("a".into()), Value::Num(42.into())),
                    (Value::Str("b".into()), Value::Num(42.into())),
                ]),
            )]),
        ]),
        dec
    );
}

#[test]
fn can_de_ext() {
    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct _ExtStruct((i8, Value));
    let mut enc = Vec::new();
    ValueRef::Ext(-42, b"hello").to_sync(&mut enc).unwrap();
    let dec: _ExtStruct = from_ref(enc.as_slice()).unwrap();
    assert_eq!(
        _ExtStruct((-42, Value::Bin(b"hello".to_vec().into_boxed_slice()))),
        dec,
    );
}

#[test]
fn can_de_value_ext() {
    let mut enc = Vec::new();
    ValueRef::Ext(-42, b"hello").to_sync(&mut enc).unwrap();
    let dec: ValueRef = from_ref(enc.as_slice()).unwrap();
    assert_eq!(ValueRef::Ext(-42, b"hello"), dec,);
}

#[test]
fn can_de_ref() {
    let mut enc = Vec::new();
    ValueRef::Arr(vec![
        ValueRef::Str("hello".into()),
        ValueRef::Ext(-42, b"hello"),
    ])
    .to_sync(&mut enc)
    .unwrap();
    let r: ValueRef = from_ref(enc.as_slice()).unwrap();
    assert_eq!(
        ValueRef::Arr(vec![
            ValueRef::Str("hello".into()),
            ValueRef::Ext(-42, b"hello")
        ]),
        r
    );
}

#[test]
fn can_de_enum() {
    #[derive(Debug, PartialEq, serde::Deserialize)]
    enum X {
        Unit,
        NoUple(),
        OneUple(i8),
        TwoUple(i8, i8),
        NoStruct {},
        OneStruct { a: i8 },
        TwoStruct { a: i8, b: i8 },
    }

    let mut enc = Vec::new();
    Value::Arr(vec![
        Value::Map(vec![("Unit".into(), ().into())]),
        Value::Map(vec![("NoUple".into(), Value::Arr(vec![]))]),
        Value::Map(vec![("OneUple".into(), 42.into())]),
        Value::Map(vec![(
            "TwoUple".into(),
            Value::Arr(vec![42.into(), 43.into()]),
        )]),
        Value::Map(vec![("NoStruct".into(), Value::Map(vec![]))]),
        Value::Map(vec![(
            "OneStruct".into(),
            Value::Map(vec![("a".into(), 42.into())]),
        )]),
        Value::Map(vec![(
            "TwoStruct".into(),
            Value::Map(vec![("a".into(), 42.into()), ("b".into(), 43.into())]),
        )]),
    ])
    .to_sync(&mut enc)
    .unwrap();
    let r: Vec<X> = from_sync(enc.as_slice()).unwrap();
    assert_eq!(
        vec![
            X::Unit,
            X::NoUple(),
            X::OneUple(42),
            X::TwoUple(42, 43),
            X::NoStruct {},
            X::OneStruct { a: 42 },
            X::TwoStruct { a: 42, b: 43 },
        ],
        r
    );
}
