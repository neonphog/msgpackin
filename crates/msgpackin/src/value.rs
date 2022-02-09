//! MessagePack Rust Value and ValueRef types

use crate::consumer::*;
use crate::producer::*;
use crate::*;

/// MessagePack Utf8 String Reference type
#[derive(Clone)]
pub struct Utf8Str(pub Box<[u8]>);

impl fmt::Debug for Utf8Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Ok(s) => s.fmt(f),
            Err(_) => write!(f, "EInvalidUtf8({:?})", &self.0),
        }
    }
}

impl fmt::Display for Utf8Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Ok(s) => s.fmt(f),
            Err(_) => write!(f, "EInvalidUtf8({:?})", &self.0),
        }
    }
}

impl From<&str> for Utf8Str {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec().into_boxed_slice())
    }
}

impl From<&String> for Utf8Str {
    fn from(s: &String) -> Self {
        s.as_str().into()
    }
}

impl From<String> for Utf8Str {
    fn from(s: String) -> Self {
        Self(s.into_bytes().into_boxed_slice())
    }
}

impl<'a> From<Cow<'a, str>> for Utf8Str {
    fn from(c: Cow<'a, str>) -> Self {
        c.into_owned().into()
    }
}

impl Utf8Str {
    /// Get a Utf8StrRef from this instance
    pub fn as_ref(&self) -> Utf8StrRef {
        self.into()
    }

    /// Attempt to get this string as a `&str`
    pub fn as_str(&self) -> Result<&str> {
        lib::core::str::from_utf8(&self.0).map_err(|_| Error::EInvalidUtf8)
    }

    /// Attempt to convert this into a rust String
    pub fn into_string(self) -> Result<String> {
        String::from_utf8(self.0.into_vec()).map_err(|_| Error::EInvalidUtf8)
    }

    /// Get the underlying raw bytes of this "string" type
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// MessagePack Rust owned Value type
#[derive(Debug, Clone)]
pub enum Value {
    /// MessagePack `Nil` type
    Nil,

    /// MessagePack `Boolean` type
    Bool(bool),

    /// MessagePack `Number` type
    Num(Num),

    /// MessagePack `Bin` type
    Bin(Box<[u8]>),

    /// MessagePack `Str` type
    Str(Utf8Str),

    /// MessagePack `Arr` type
    Arr(Vec<Value>),

    /// MessagePack `Map` type
    Map(Vec<(Value, Value)>),

    /// MessagePack `Ext` type
    Ext(i8, Box<[u8]>),
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Nil
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

macro_rules! num_2_v {
    ($($t:ty)*) => {$(
        impl From<$t> for Value {
            fn from(n: $t) -> Self {
                Value::Num(n.into())
            }
        }
    )*};
}

num_2_v!( i8 i16 i32 i64 isize u8 u16 u32 u64 usize f32 f64 );

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Str(s.into())
    }
}

impl From<&String> for Value {
    fn from(s: &String) -> Self {
        Value::Str(s.into())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s.into())
    }
}

impl<'a> From<Cow<'a, str>> for Value {
    fn from(c: Cow<'a, str>) -> Self {
        Value::Str(c.into())
    }
}

impl From<&[u8]> for Value {
    fn from(b: &[u8]) -> Self {
        Value::Bin(b.to_vec().into_boxed_slice())
    }
}

impl From<Box<[u8]>> for Value {
    fn from(b: Box<[u8]>) -> Self {
        Value::Bin(b)
    }
}

impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Value::Bin(b.into_boxed_slice())
    }
}

fn priv_decode<'func, 'prod>(
    iter: &mut (impl Iterator<Item = OwnedToken> + 'func),
    config: &Config,
) -> Result<Value> {
    match iter.next() {
        Some(OwnedToken::Nil) => Ok(Value::Nil),
        Some(OwnedToken::Bool(b)) => Ok(Value::Bool(b)),
        Some(OwnedToken::Num(n)) => Ok(Value::Num(n)),
        Some(OwnedToken::Bin(b)) => Ok(Value::Bin(b)),
        Some(OwnedToken::Str(s)) => Ok(Value::Str(Utf8Str(s))),
        Some(OwnedToken::Ext(t, d)) => Ok(Value::Ext(t, d)),
        Some(OwnedToken::Arr(l)) => {
            let mut arr = Vec::with_capacity(l as usize);
            for _ in 0..l {
                arr.push(priv_decode(iter, config)?);
            }
            Ok(Value::Arr(arr))
        }
        Some(OwnedToken::Map(l)) => {
            let mut map = Vec::with_capacity(l as usize);
            for _ in 0..l {
                let key = priv_decode(iter, config)?;
                let val = priv_decode(iter, config)?;
                map.push((key, val));
            }
            Ok(Value::Map(map))
        }
        None => Err(Error::EDecode {
            expected: "Marker".into(),
            got: "UnexpectedEOF".into(),
        }),
    }
}

impl Value {
    /// Get a ValueRef from this instance
    pub fn as_ref(&self) -> ValueRef {
        self.into()
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub fn encode_sync<'con, C>(&self, c: C) -> Result<()>
    where
        C: Into<DynConsumerSync<'con>>,
    {
        self.encode_sync_config(c, &Config::default())
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub fn encode_sync_config<'con, C>(
        &self,
        c: C,
        config: &Config,
    ) -> Result<()>
    where
        C: Into<DynConsumerSync<'con>>,
    {
        ValueRef::from(self).encode_sync_config(c, config)
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub async fn encode_async<'con, C>(&self, c: C) -> Result<()>
    where
        C: Into<DynConsumerAsync<'con>>,
    {
        self.encode_async_config(c, &Config::default()).await
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub async fn encode_async_config<'con, C>(
        &self,
        c: C,
        config: &Config,
    ) -> Result<()>
    where
        C: Into<DynConsumerAsync<'con>>,
    {
        ValueRef::from(self).encode_async_config(c, config).await
    }

    /// Decode a Value from something that can be converted
    /// into a DynProducerSync, such as a byte array slice (`&[u8]`)
    pub fn from_sync<'prod, P>(p: P) -> Result<Self>
    where
        P: Into<DynProducerSync<'prod>>,
    {
        Self::from_sync_config(p, &Config::default())
    }

    /// Decode a Value from something that can be converted
    /// into a DynProducerSync, such as a byte array slice (`&[u8]`)
    pub fn from_sync_config<'prod, P>(p: P, config: &Config) -> Result<Self>
    where
        P: Into<DynProducerSync<'prod>>,
    {
        let mut tokens = Vec::new();
        let mut dec = msgpackin_core::decode::Decoder::new();
        let mut p = p.into();
        priv_decode_owned_sync(&mut tokens, &mut dec, &mut p, config)?;
        let mut iter = tokens.into_iter();
        priv_decode(&mut iter, config)
    }

    /// Decode a Value from something that can be converted
    /// into a DynProducerAsync, such as a byte array slice (`&[u8]`)
    pub async fn from_async<'prod, P>(p: P) -> Result<Self>
    where
        P: Into<DynProducerAsync<'prod>>,
    {
        Self::from_async_config(p, &Config::default()).await
    }

    /// Decode a Value from something that can be converted
    /// into a DynProducerAsync, such as a byte array slice (`&[u8]`)
    pub async fn from_async_config<'prod, P>(
        p: P,
        config: &Config,
    ) -> Result<Self>
    where
        P: Into<DynProducerAsync<'prod>>,
    {
        let mut tokens = Vec::new();
        let mut dec = msgpackin_core::decode::Decoder::new();
        let mut p = p.into();
        priv_decode_owned_async(&mut tokens, &mut dec, &mut p, config).await?;
        let mut iter = tokens.into_iter();
        priv_decode(&mut iter, config)
    }
}

/// MessagePack Utf8 String Reference type
pub struct Utf8StrRef<'lt>(pub &'lt [u8]);

impl<'a> From<&'a Utf8Str> for Utf8StrRef<'a> {
    fn from(s: &'a Utf8Str) -> Self {
        Utf8StrRef(&s.0)
    }
}

impl fmt::Debug for Utf8StrRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Ok(s) => s.fmt(f),
            Err(_) => write!(f, "EInvalidUtf8({:?})", &self.0),
        }
    }
}

impl fmt::Display for Utf8StrRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Ok(s) => s.fmt(f),
            Err(_) => write!(f, "EInvalidUtf8({:?})", &self.0),
        }
    }
}

impl<'lt> Utf8StrRef<'lt> {
    /// Attempt to get this string as a `&str`
    pub fn as_str(&self) -> Result<&'lt str> {
        lib::core::str::from_utf8(self.0).map_err(|_| Error::EInvalidUtf8)
    }

    /// Get the underlying raw bytes of this "string" type
    pub fn as_bytes(&self) -> &'lt [u8] {
        self.0
    }
}

/// MessagePack Rust Value Reference type
#[derive(Debug)]
pub enum ValueRef<'lt> {
    /// MessagePack `Nil` type
    Nil,

    /// MessagePack `Boolean` type
    Bool(bool),

    /// MessagePack `Number` type
    Num(Num),

    /// MessagePack `Bin` type
    Bin(&'lt [u8]),

    /// MessagePack `Str` type
    Str(Utf8StrRef<'lt>),

    /// MessagePack `Arr` type
    Arr(Vec<ValueRef<'lt>>),

    /// MessagePack `Map` type
    Map(Vec<(ValueRef<'lt>, ValueRef<'lt>)>),

    /// MessagePack `Ext` type
    Ext(i8, &'lt [u8]),
}

impl<'a> From<&'a Value> for ValueRef<'a> {
    fn from(v: &'a Value) -> Self {
        match v {
            Value::Nil => ValueRef::Nil,
            Value::Bool(b) => ValueRef::Bool(*b),
            Value::Num(n) => ValueRef::Num(*n),
            Value::Bin(data) => ValueRef::Bin(data),
            Value::Str(data) => ValueRef::Str(data.into()),
            Value::Ext(t, data) => ValueRef::Ext(*t, data),
            Value::Arr(a) => ValueRef::Arr(a.iter().map(Into::into).collect()),
            Value::Map(m) => ValueRef::Map(
                m.iter().map(|(k, v)| (k.into(), v.into())).collect(),
            ),
        }
    }
}

macro_rules! stub_wrap {
    ($($t:tt)*) => { $($t)* };
}

macro_rules! async_wrap {
    ($($t:tt)*) => { Box::pin(async move { $($t)* }) };
}

macro_rules! mk_encode {
    (
        $id:ident,
        ($($con:tt)*),
        ($($await:tt)*),
        ($($ret:tt)*),
        $wrap:ident,
    ) => {
        fn $id<'func, 'con>(
            val: &'func ValueRef,
            enc: &'func mut msgpackin_core::encode::Encoder,
            con: &'func mut $($con)*,
            config: &'func Config,
        ) -> $($ret)* {$wrap! {
            match val {
                ValueRef::Nil => con.write(&enc.enc_nil())$($await)*,
                ValueRef::Bool(b) => con.write(&enc.enc_bool(*b))$($await)*,
                ValueRef::Num(n) => con.write(&enc.enc_num(*n))$($await)*,
                ValueRef::Bin(data) => {
                    con.write(&enc.enc_bin_len(data.len() as u32))$($await)*?;
                    con.write(data)$($await)*
                }
                ValueRef::Str(data) => {
                    con.write(&enc.enc_str_len(data.0.len() as u32))$($await)*?;
                    con.write(&data.0)$($await)*
                }
                ValueRef::Ext(t, data) => {
                    con.write(
                        &enc.enc_ext_len(data.len() as u32, *t),
                    )$($await)*?;
                    con.write(data)$($await)*
                }
                ValueRef::Arr(a) => {
                    con.write(&enc.enc_arr_len(a.len() as u32))$($await)*?;
                    for item in a.iter() {
                        $id(item, enc, con, config)$($await)*?;
                    }
                    Ok(())
                }
                ValueRef::Map(m) => {
                    con.write(&enc.enc_map_len(m.len() as u32))$($await)*?;
                    for (key, value) in m.iter() {
                        $id(key, enc, con, config)$($await)*?;
                        $id(value, enc, con, config)$($await)*?;
                    }
                    Ok(())
                }
            }
        }}
    };
}

mk_encode!(
    priv_encode_sync,
    (DynConsumerSync<'con>),
    (),
    (Result<()>),
    stub_wrap,
);

mk_encode!(
    priv_encode_async,
    (DynConsumerAsync<'con>),
    (.await),
    (BoxFut<'func, ()>),
    async_wrap,
);

struct VRDecode<'dec, 'buf> {
    iter: msgpackin_core::decode::TokenIter<'dec, 'buf>,
}

impl<'dec, 'buf> VRDecode<'dec, 'buf> {
    fn next_val(&mut self) -> Result<ValueRef<'buf>> {
        use msgpackin_core::decode::LenType;
        use msgpackin_core::decode::Token::*;
        match self.iter.next() {
            Some(Nil) => Ok(ValueRef::Nil),
            Some(Bool(b)) => Ok(ValueRef::Bool(b)),
            Some(Num(n)) => Ok(ValueRef::Num(n)),
            tok @ Some(Len(LenType::Bin, l)) => {
                if let Some(Bin(data)) = self.iter.next() {
                    if data.len() == l as usize {
                        return Ok(ValueRef::Bin(data));
                    }
                }
                Err(Error::EDecode {
                    expected: format!("Some(Bin({:?} bytes))", l),
                    got: format!("{:?}", tok),
                })
            }
            tok @ Some(Len(LenType::Str, l)) => {
                if let Some(Bin(data)) = self.iter.next() {
                    if data.len() == l as usize {
                        return Ok(ValueRef::Str(Utf8StrRef(data)));
                    }
                }
                Err(Error::EDecode {
                    expected: format!("Some(Bin({:?} bytes))", l),
                    got: format!("{:?}", tok),
                })
            }
            tok @ Some(Len(LenType::Ext(ext_type), l)) => {
                if let Some(Bin(data)) = self.iter.next() {
                    if data.len() == l as usize {
                        return Ok(ValueRef::Ext(ext_type, data));
                    }
                }
                Err(Error::EDecode {
                    expected: format!("Some(Bin({:?} bytes))", l),
                    got: format!("{:?}", tok),
                })
            }
            Some(Len(LenType::Arr, l)) => {
                let mut out = Vec::with_capacity(l as usize);
                for _ in 0..l {
                    out.push(self.next_val()?);
                }
                Ok(ValueRef::Arr(out))
            }
            Some(Len(LenType::Map, l)) => {
                let mut out = Vec::with_capacity(l as usize);
                for _ in 0..l {
                    let key = self.next_val()?;
                    let val = self.next_val()?;
                    out.push((key, val));
                }
                Ok(ValueRef::Map(out))
            }
            None => Err(Error::EDecode {
                expected: "Marker".into(),
                got: "UnexpectedEOF".into(),
            }),
            tok => Err(Error::EDecode {
                expected: "Marker".into(),
                got: format!("{:?}", tok),
            }),
        }
    }
}

impl<'lt> ValueRef<'lt> {
    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub fn encode_sync<'con, C>(&self, c: C) -> Result<()>
    where
        C: Into<DynConsumerSync<'con>>,
    {
        self.encode_sync_config(c, &Config::default())
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub fn encode_sync_config<'con, C>(
        &self,
        c: C,
        config: &Config,
    ) -> Result<()>
    where
        C: Into<DynConsumerSync<'con>>,
    {
        let mut enc = msgpackin_core::encode::Encoder::new();
        let mut c = c.into();
        priv_encode_sync(self, &mut enc, &mut c, config)
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub async fn encode_async<'con, C>(&self, c: C) -> Result<()>
    where
        C: Into<DynConsumerAsync<'con>>,
    {
        self.encode_async_config(c, &Config::default()).await
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub async fn encode_async_config<'con, C>(
        &self,
        c: C,
        config: &Config,
    ) -> Result<()>
    where
        C: Into<DynConsumerAsync<'con>>,
    {
        let mut enc = msgpackin_core::encode::Encoder::new();
        let mut c = c.into();
        priv_encode_async(self, &mut enc, &mut c, config).await
    }

    /// Decode a ValueRef from something that can be converted
    /// into a DynProducerComplete, such as a byte array slice (`&[u8]`)
    pub fn from_ref<P>(p: P) -> Result<Self>
    where
        P: Into<DynProducerComplete<'lt>>,
    {
        Self::from_ref_config(p, &Config::default())
    }

    /// Decode a ValueRef from something that can be converted
    /// into a DynProducerComplete, such as a byte array slice (`&[u8]`)
    pub fn from_ref_config<P>(p: P, config: &Config) -> Result<Self>
    where
        P: Into<DynProducerComplete<'lt>>,
    {
        let _config = config;
        let mut dec = msgpackin_core::decode::Decoder::new();
        let mut dec = VRDecode {
            iter: dec.parse(p.into().read_all()?),
        };

        dec.next_val()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_value_encode_decode() {
        let arr = Value::Arr(vec![
            Value::from(()),
            Value::from(true),
            Value::from(false),
            Value::from("hello"),
            Value::from(&b"hello"[..]),
            Value::from(-42_i8),
            Value::from(3.14159_f64),
        ]);
        let map = Value::Map(vec![
            (Value::from("array"), arr),
            (Value::from("nother"), Value::from("testing")),
        ]);
        let mut data = Vec::new();
        map.encode_sync(&mut data).unwrap();
        let mut data2 = Vec::new();
        futures::executor::block_on(async {
            map.encode_async(&mut data2).await.unwrap();
        });
        //println!("encoded: {}", String::from_utf8_lossy(&data));
        assert_eq!(data, data2);

        let _dec = ValueRef::from_ref(data.as_slice()).unwrap();
        let _dec = Value::from_sync(data.as_slice()).unwrap();
        let _dec = futures::executor::block_on(async {
            Value::from_async(data.as_slice()).await
        })
        .unwrap();
        //println!("decoded: {:?}", dec);
    }
}
