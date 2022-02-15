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
