use crate::*;

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
