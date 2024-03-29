//! MessagePack Rust Value and ValueRef types

use crate::consumer::*;
use crate::producer::*;
use crate::*;

#[cfg(feature = "serde")]
macro_rules! visit {
    ($id:ident, $ty:ty, $n:ident, $b:block) => {
        fn $id<E>(self, $n: $ty) -> result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        $b
    };
}

/// MessagePack Utf8 String Reference type
#[derive(Clone, PartialEq)]
pub struct Utf8Str(pub Box<[u8]>);

#[cfg(feature = "serde")]
impl serde::Serialize for Utf8Str {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.as_str() {
            Ok(s) => serializer.serialize_str(s),
            Err(_) => serializer.serialize_bytes(&self.0),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Utf8Str {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Utf8Str;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("str, or bytes")
            }

            visit!(visit_str, &str, v, {
                Ok(Utf8Str(v.to_string().into_bytes().into_boxed_slice()))
            });
            visit!(visit_string, String, v, {
                Ok(Utf8Str(v.into_bytes().into_boxed_slice()))
            });
            visit!(visit_bytes, &[u8], v, {
                Ok(Utf8Str(v.to_vec().into_boxed_slice()))
            });
            visit!(visit_byte_buf, Vec<u8>, v, {
                Ok(Utf8Str(v.into_boxed_slice()))
            });
        }

        deserializer.deserialize_string(V)
    }
}

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

impl<'a> From<&Utf8StrRef<'a>> for Utf8Str {
    fn from(s: &Utf8StrRef<'a>) -> Self {
        Self(s.0.to_vec().into_boxed_slice())
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
#[derive(Debug, Clone, PartialEq)]
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

#[cfg(feature = "serde")]
impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&ValueRef::from(self), serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Value;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any")
            }

            visit!(visit_bool, bool, v, { Ok(Value::Bool(v)) });
            visit!(visit_i8, i8, v, { Ok(Value::Num(v.into())) });
            visit!(visit_i16, i16, v, { Ok(Value::Num(v.into())) });
            visit!(visit_i32, i32, v, { Ok(Value::Num(v.into())) });
            visit!(visit_i64, i64, v, { Ok(Value::Num(v.into())) });
            visit!(visit_u8, u8, v, { Ok(Value::Num(v.into())) });
            visit!(visit_u16, u16, v, { Ok(Value::Num(v.into())) });
            visit!(visit_u32, u32, v, { Ok(Value::Num(v.into())) });
            visit!(visit_u64, u64, v, { Ok(Value::Num(v.into())) });
            visit!(visit_f32, f32, v, { Ok(Value::Num(v.into())) });
            visit!(visit_f64, f64, v, { Ok(Value::Num(v.into())) });
            visit!(visit_str, &str, v, {
                Ok(Value::Str(Utf8Str(
                    v.to_string().into_bytes().into_boxed_slice(),
                )))
            });
            visit!(visit_string, String, v, {
                Ok(Value::Str(Utf8Str(v.into_bytes().into_boxed_slice())))
            });
            visit!(visit_bytes, &[u8], v, {
                Ok(Value::Bin(v.to_vec().into_boxed_slice()))
            });
            visit!(visit_byte_buf, Vec<u8>, v, {
                Ok(Value::Bin(v.into_boxed_slice()))
            });

            fn visit_none<E>(self) -> result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Nil)
            }

            fn visit_some<D>(
                self,
                deserializer: D,
            ) -> result::Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                deserializer.deserialize_any(V)
            }

            fn visit_unit<E>(self) -> result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Nil)
            }

            fn visit_newtype_struct<D>(
                self,
                deserializer: D,
            ) -> result::Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                let dec: Self::Value = deserializer.deserialize_any(V)?;
                if let Value::Arr(mut arr) = dec {
                    if arr.len() == 2 {
                        let t = arr.remove(0);
                        let data = arr.remove(0);
                        if let (Value::Num(t), Value::Bin(data)) = (t, data) {
                            if t.fits::<i8>() {
                                return Ok(Value::Ext(t.to(), data));
                            }
                        }
                    }
                }
                unreachable!()
            }

            fn visit_seq<A>(
                self,
                mut acc: A,
            ) -> result::Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut arr = match acc.size_hint() {
                    Some(l) => Vec::with_capacity(l),
                    None => Vec::new(),
                };
                while let Some(v) = acc.next_element()? {
                    arr.push(v);
                }
                Ok(Value::Arr(arr))
            }

            fn visit_map<A>(
                self,
                mut acc: A,
            ) -> result::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut map = match acc.size_hint() {
                    Some(l) => Vec::with_capacity(l),
                    None => Vec::new(),
                };
                while let Some(pair) = acc.next_entry()? {
                    map.push(pair);
                }
                Ok(Value::Map(map))
            }
        }

        deserializer.deserialize_any(V)
    }
}

impl PartialEq<ValueRef<'_>> for Value {
    fn eq(&self, oth: &ValueRef) -> bool {
        &self.as_ref() == oth
    }
}

impl<'a> From<&ValueRef<'a>> for Value {
    fn from(r: &ValueRef<'a>) -> Self {
        match r {
            ValueRef::Nil => Value::Nil,
            ValueRef::Bool(b) => Value::Bool(*b),
            ValueRef::Num(n) => Value::Num(*n),
            ValueRef::Bin(data) => Value::Bin(data.to_vec().into_boxed_slice()),
            ValueRef::Str(data) => Value::Str(data.into()),
            ValueRef::Ext(t, data) => {
                Value::Ext(*t, data.to_vec().into_boxed_slice())
            }
            ValueRef::Arr(a) => Value::Arr(a.iter().map(Into::into).collect()),
            ValueRef::Map(m) => Value::Map(
                m.iter().map(|(k, v)| (k.into(), v.into())).collect(),
            ),
        }
    }
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
    let _config = config;
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

    /// Encode this value as a `Vec<u8>`
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        self.to_sync(&mut out)?;
        Ok(out)
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub fn to_sync<'con, C>(&self, c: C) -> Result<()>
    where
        C: Into<DynConsumerSync<'con>>,
    {
        self.to_sync_config(c, &Config::default())
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub fn to_sync_config<'con, C>(&self, c: C, config: &Config) -> Result<()>
    where
        C: Into<DynConsumerSync<'con>>,
    {
        ValueRef::from(self).to_sync_config(c, config)
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub async fn to_async<'con, C>(&self, c: C) -> Result<()>
    where
        C: Into<DynConsumerAsync<'con>>,
    {
        self.to_async_config(c, &Config::default()).await
    }

    /// Encode this value as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub async fn to_async_config<'con, C>(
        &self,
        c: C,
        config: &Config,
    ) -> Result<()>
    where
        C: Into<DynConsumerAsync<'con>>,
    {
        ValueRef::from(self).to_async_config(c, config).await
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
#[derive(Clone, PartialEq)]
pub struct Utf8StrRef<'lt>(pub &'lt [u8]);

#[cfg(feature = "serde")]
impl<'lt> serde::Serialize for Utf8StrRef<'lt> {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.as_str() {
            Ok(s) => serializer.serialize_str(s),
            Err(_) => serializer.serialize_bytes(self.0),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Utf8StrRef<'de> {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Utf8StrRef<'de>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("str, or bytes")
            }

            visit!(visit_borrowed_str, &'de str, v, {
                Ok(Utf8StrRef(v.as_bytes()))
            });
            visit!(visit_borrowed_bytes, &'de [u8], v, { Ok(Utf8StrRef(v)) });
        }

        deserializer.deserialize_str(V)
    }
}

impl<'a> From<&'a Utf8Str> for Utf8StrRef<'a> {
    fn from(s: &'a Utf8Str) -> Self {
        Utf8StrRef(&s.0)
    }
}

impl<'a> From<&'a str> for Utf8StrRef<'a> {
    fn from(s: &'a str) -> Self {
        Utf8StrRef(s.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for Utf8StrRef<'a> {
    fn from(s: &'a [u8]) -> Self {
        Utf8StrRef(s)
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
#[derive(Debug, Clone, PartialEq)]
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

#[cfg(feature = "serde")]
impl<'lt> serde::Serialize for ValueRef<'lt> {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ValueRef::Nil => serializer.serialize_unit(),
            ValueRef::Bool(b) => serializer.serialize_bool(*b),
            ValueRef::Num(Num::Unsigned(u)) => serializer.serialize_u64(*u),
            ValueRef::Num(Num::Signed(i)) => serializer.serialize_i64(*i),
            ValueRef::Num(Num::F32(f)) => serializer.serialize_f32(*f),
            ValueRef::Num(Num::F64(f)) => serializer.serialize_f64(*f),
            ValueRef::Bin(data) => serializer.serialize_bytes(data),
            ValueRef::Str(s) => serde::Serialize::serialize(s, serializer),
            ValueRef::Arr(arr) => {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr.iter() {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            ValueRef::Map(map) => {
                use serde::ser::SerializeMap;
                let mut ser = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map.iter() {
                    ser.serialize_entry(k, v)?;
                }
                ser.end()
            }
            ValueRef::Ext(t, data) => serializer.serialize_newtype_struct(
                EXT_STRUCT_NAME,
                &(t, ValueRef::Bin(data)),
            ),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ValueRef<'de> {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = ValueRef<'de>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any")
            }

            visit!(visit_bool, bool, v, { Ok(ValueRef::Bool(v)) });
            visit!(visit_i8, i8, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_i16, i16, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_i32, i32, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_i64, i64, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_u8, u8, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_u16, u16, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_u32, u32, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_u64, u64, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_f32, f32, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_f64, f64, v, { Ok(ValueRef::Num(v.into())) });
            visit!(visit_borrowed_str, &'de str, v, {
                Ok(ValueRef::Str(Utf8StrRef(v.as_bytes())))
            });
            visit!(visit_borrowed_bytes, &'de [u8], v, {
                Ok(ValueRef::Bin(v))
            });

            fn visit_none<E>(self) -> result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ValueRef::Nil)
            }

            fn visit_some<D>(
                self,
                deserializer: D,
            ) -> result::Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                deserializer.deserialize_any(V)
            }

            fn visit_unit<E>(self) -> result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ValueRef::Nil)
            }

            fn visit_newtype_struct<D>(
                self,
                deserializer: D,
            ) -> result::Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                let dec: Self::Value = deserializer.deserialize_any(V)?;
                if let ValueRef::Arr(mut arr) = dec {
                    if arr.len() == 2 {
                        let t = arr.remove(0);
                        let data = arr.remove(0);
                        if let (ValueRef::Num(t), ValueRef::Bin(data)) =
                            (t, data)
                        {
                            if t.fits::<i8>() {
                                return Ok(ValueRef::Ext(t.to(), data));
                            }
                        }
                    }
                }
                unreachable!()
            }

            fn visit_seq<A>(
                self,
                mut acc: A,
            ) -> result::Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut arr = match acc.size_hint() {
                    Some(l) => Vec::with_capacity(l),
                    None => Vec::new(),
                };
                while let Some(v) = acc.next_element()? {
                    arr.push(v);
                }
                Ok(ValueRef::Arr(arr))
            }

            fn visit_map<A>(
                self,
                mut acc: A,
            ) -> result::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut map = match acc.size_hint() {
                    Some(l) => Vec::with_capacity(l),
                    None => Vec::new(),
                };
                while let Some(pair) = acc.next_entry()? {
                    map.push(pair);
                }
                Ok(ValueRef::Map(map))
            }
        }

        deserializer.deserialize_any(V)
    }
}

impl PartialEq<Value> for ValueRef<'_> {
    fn eq(&self, oth: &Value) -> bool {
        self == &oth.as_ref()
    }
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
    /// Convert this ValueRef into an owned Value
    pub fn to_owned(&self) -> Value {
        self.into()
    }

    /// Encode this value ref as a `Vec<u8>`
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        self.to_sync(&mut out)?;
        Ok(out)
    }

    /// Encode this value ref as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub fn to_sync<'con, C>(&self, c: C) -> Result<()>
    where
        C: Into<DynConsumerSync<'con>>,
    {
        self.to_sync_config(c, &Config::default())
    }

    /// Encode this value ref as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub fn to_sync_config<'con, C>(&self, c: C, config: &Config) -> Result<()>
    where
        C: Into<DynConsumerSync<'con>>,
    {
        let mut enc = msgpackin_core::encode::Encoder::new();
        let mut c = c.into();
        priv_encode_sync(self, &mut enc, &mut c, config)
    }

    /// Encode this value ref as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub async fn to_async<'con, C>(&self, c: C) -> Result<()>
    where
        C: Into<DynConsumerAsync<'con>>,
    {
        self.to_async_config(c, &Config::default()).await
    }

    /// Encode this value ref as message pack data to the given consumer.
    /// E.g. `&mut Vec<u8>`
    pub async fn to_async_config<'con, C>(
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

    #[cfg(feature = "serde")]
    #[test]
    fn test_value_serde() {
        let arr = Value::Arr(vec![
            Value::from(true),
            Value::Ext(-42, b"hello".to_vec().into()),
            Value::from(false),
        ]);
        let _res = crate::to_bytes(&arr).unwrap();
    }

    #[test]
    fn test_value_encode_decode() {
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
        map.to_sync(&mut data).unwrap();
        let mut data2 = Vec::new();
        futures::executor::block_on(async {
            map.to_async(&mut data2).await.unwrap();
        });
        assert_eq!(data, data2);

        let dec1 = ValueRef::from_ref(data.as_slice()).unwrap();
        assert_eq!(dec1, map);

        let dec2 = Value::from_sync(data.as_slice()).unwrap();
        assert_eq!(dec1, dec2);

        let dec3 = futures::executor::block_on(async {
            Value::from_async(data.as_slice()).await
        })
        .unwrap();
        assert_eq!(dec2, dec3);
    }
}
