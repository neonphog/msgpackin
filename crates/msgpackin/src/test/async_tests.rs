use crate::*;

#[test]
fn async_encode_decode_demo() {
    let expect = Value::Map(vec![("foo".into(), "bar".into())]);
    let mut buf = Vec::new();

    {
        let writer: Box<dyn tokio::io::AsyncWrite + Unpin> = Box::new(&mut buf);
        futures::executor::block_on(async { expect.to_async(writer).await })
            .unwrap();
    }

    let reader: Box<dyn tokio::io::AsyncRead + Unpin> =
        Box::new(buf.as_slice());
    let decoded =
        futures::executor::block_on(async { Value::from_async(reader).await })
            .unwrap();
    assert_eq!(expect, decoded);
}
