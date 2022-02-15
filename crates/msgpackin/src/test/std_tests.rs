use crate::*;

#[test]
fn std_encode_decode_demo() {
    let expect = Value::Map(vec![("foo".into(), "bar".into())]);
    let mut buf = Vec::new();

    {
        let writer: Box<dyn std::io::Write> = Box::new(&mut buf);
        expect.to_sync(writer).unwrap();
    }

    let reader: Box<dyn std::io::Read> = Box::new(buf.as_slice());
    let decoded = Value::from_sync(reader).unwrap();
    assert_eq!(expect, decoded);
}
