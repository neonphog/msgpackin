# msgpackin

Msgpackin pure Rust MessagePack encoding / decoding library.

Msgpackin supports `no_std`, but requires the `alloc` crate.
If you're looking for no alloc support, see the `msgpackin_core` crate.

Msgpackin supports serde with the `serde` feature in either `no_std`
or `std` mode. If you want `std` mode, you must also enable the
`serde_std` feature, as a temporary workaround until weak dependency
features are stablized.

#### Roadmap

- [x] Value / ValueRef encoding and decoding without serde
- [x] zero-copy string/binary/ext data with `from_ref`
- [x] `no_std` serde support
- [x] `std::io::{Read, Write}` support in `std` mode
- [x] Async IO support via `futures-io` or `tokio` features
- [ ] recursion depth checking (the config is currently a stub)
- [ ] hooks for managed encoding / decoding of ext types
      (e.g. Timestamp (`-1`))
- [ ] benchmarking / optimization

#### Features

- `std` - enabled by default, pulls in the rust std library, enabling
          encoding and decoding via `std::io::{Read, Write}` traits
- `serde` - enables serialization / deserialization through the `serde`
            crate
- `futures-io` - enables async encoding and decoding through the futures
                 `io::{AsyncRead, AsyncWrite}` traits
- `tokio` - enables async encoding and decoding through the tokio
            `io::{AsyncRead, AsyncWrite}` traits

#### `no_std` Example

```rust
use msgpackin::*;
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
```

#### `std` Example

```rust
use msgpackin::*;
let expect = Value::Map(vec![("foo".into(), "bar".into())]);
let mut buf = Vec::new();

{
    let writer: Box<dyn std::io::Write> = Box::new(&mut buf);
    expect.to_sync(writer).unwrap();
}

let reader: Box<dyn std::io::Read> = Box::new(buf.as_slice());
let decoded = Value::from_sync(reader).unwrap();
assert_eq!(expect, decoded);
```

#### Async Example

```rust
use msgpackin::*;
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
```

#### `serde` Example

```rust
use msgpackin::*;
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
```

License: Apache-2.0
