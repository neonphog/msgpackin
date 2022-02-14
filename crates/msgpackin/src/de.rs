//! serde Deserializer implementations

use crate::producer::*;
use crate::*;
use serde::de;
use serde::Deserialize;

/// Deserialize from something that can be converted
/// into a DynProducerComplete, such as a byte array slice (`&[u8]`)
pub fn from_ref<'de, P, T>(p: P) -> Result<T>
where
    P: Into<DynProducerComplete<'de>>,
    T: Deserialize<'de>,
{
    from_ref_config(p, &Config::default())
}

/// Deserialize from something that can be converted
/// into a DynProducerComplete, such as a byte array slice (`&[u8]`)
pub fn from_ref_config<'de, P, T>(p: P, config: &Config) -> Result<T>
where
    P: Into<DynProducerComplete<'de>>,
    T: Deserialize<'de>,
{
    let mut deserializer = DeserializerSync::from_ref_config(p, config)?;
    T::deserialize(&mut deserializer)
}

/// Deserialize from something that can be converted
/// into a DynProducerSync, such as a byte array slice (`&[u8]`)
pub fn from_sync<'de, P, T>(p: P) -> Result<T>
where
    P: Into<DynProducerSync<'de>>,
    T: de::DeserializeOwned,
{
    from_sync_config(p, &Config::default())
}

/// Deserialize from something that can be converted
/// into a DynProducerSync, such as a byte array slice (`&[u8]`)
pub fn from_sync_config<'de, P, T>(p: P, config: &Config) -> Result<T>
where
    P: Into<DynProducerSync<'de>>,
    T: de::DeserializeOwned,
{
    let mut deserializer = DeserializerSync::from_sync_config(p, config)?;
    T::deserialize(&mut deserializer)
}

/// Deserialize from something that can be converted
/// into a DynProducerAsync, such as a byte array slice (`&[u8]`)
pub async fn from_async<'de, P, T>(p: P) -> Result<T>
where
    P: Into<DynProducerAsync<'de>>,
    T: de::DeserializeOwned,
{
    from_async_config(p, &Config::default()).await
}

/// Deserialize from something that can be converted
/// into a DynProducerAsync, such as a byte array slice (`&[u8]`)
pub async fn from_async_config<'de, P, T>(p: P, config: &Config) -> Result<T>
where
    P: Into<DynProducerAsync<'de>>,
    T: de::DeserializeOwned,
{
    let mut deserializer =
        DeserializerSync::from_async_config(p, config).await?;
    T::deserialize(&mut deserializer)
}

/// a value that is either owned or a reference
enum MetaValue<'lt> {
    O(Value),
    R(ValueRef<'lt>),
}

/// Msgpackin serde DeserializerSync
pub struct DeserializerSync<'de>(Option<MetaValue<'de>>);

impl<'de> DeserializerSync<'de> {
    /// Construct a DeserializerSync from something that can be converted
    /// into a DynProducerComplete, such as a byte array slice (`&[u8]`)
    pub fn from_ref_config<P: Into<DynProducerComplete<'de>>>(
        p: P,
        config: &Config,
    ) -> Result<DeserializerSync<'de>> {
        Ok(Self(Some(MetaValue::R(ValueRef::from_ref_config(
            p, config,
        )?))))
    }

    /// Construct a DeserializerSync from something that can be converted
    /// into a DynProducerSync, such as a byte array slice (`&[u8]`)
    pub fn from_sync_config<'a, P: Into<DynProducerSync<'a>>>(
        p: P,
        config: &Config,
    ) -> Result<DeserializerSync<'de>> {
        Ok(Self(Some(MetaValue::O(Value::from_sync_config(
            p, config,
        )?))))
    }

    /// Construct a DeserializerSync from something that can be converted
    /// into a DynProducerAsync, such as a byte array slice (`&[u8]`)
    pub async fn from_async_config<'a, P: Into<DynProducerAsync<'a>>>(
        p: P,
        config: &Config,
    ) -> Result<DeserializerSync<'de>> {
        Ok(Self(Some(MetaValue::O(
            Value::from_async_config(p, config).await?,
        ))))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut DeserializerSync<'de> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match &self.0 {
            Some(MetaValue::O(Value::Nil))
            | Some(MetaValue::R(ValueRef::Nil)) => {
                self.deserialize_unit(visitor)
            }
            Some(MetaValue::O(Value::Bool(_)))
            | Some(MetaValue::R(ValueRef::Bool(_))) => {
                self.deserialize_bool(visitor)
            }
            Some(MetaValue::O(Value::Num(n)))
            | Some(MetaValue::R(ValueRef::Num(n))) => match n {
                Num::Unsigned(_) => self.deserialize_u64(visitor),
                Num::Signed(_) => self.deserialize_i64(visitor),
                Num::F32(_) => self.deserialize_f32(visitor),
                Num::F64(_) => self.deserialize_f64(visitor),
            },
            Some(MetaValue::O(Value::Arr(_)))
            | Some(MetaValue::R(ValueRef::Arr(_))) => {
                self.deserialize_seq(visitor)
            }
            Some(MetaValue::O(Value::Map(_)))
            | Some(MetaValue::R(ValueRef::Map(_))) => {
                self.deserialize_map(visitor)
            }
            Some(MetaValue::O(Value::Str(_))) => {
                self.deserialize_string(visitor)
            }
            Some(MetaValue::R(ValueRef::Str(_))) => {
                self.deserialize_str(visitor)
            }
            Some(MetaValue::O(Value::Bin(_))) => {
                self.deserialize_byte_buf(visitor)
            }
            Some(MetaValue::R(ValueRef::Bin(_))) => {
                self.deserialize_bytes(visitor)
            }
            Some(MetaValue::O(Value::Ext(_, _)))
            | Some(MetaValue::R(ValueRef::Ext(_, _))) => {
                self.deserialize_newtype_struct(EXT_STRUCT_NAME, visitor)
            }
            None => Err("uninitialized".into()),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Bool(b)))
        | Some(MetaValue::R(ValueRef::Bool(b))) = self.0.take()
        {
            return visitor.visit_bool(b);
        }
        Err("expected bool".into())
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<i8>() {
                return visitor.visit_i8(n.to());
            }
        }
        Err("expected i8".into())
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<i16>() {
                return visitor.visit_i16(n.to());
            }
        }
        Err("expected i16".into())
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<i32>() {
                return visitor.visit_i32(n.to());
            }
        }
        Err("expected i32".into())
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<i64>() {
                return visitor.visit_i64(n.to());
            }
        }
        Err("expected i64".into())
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<u8>() {
                return visitor.visit_u8(n.to());
            }
        }
        Err("expected u8".into())
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<u16>() {
                return visitor.visit_u16(n.to());
            }
        }
        Err("expected u16".into())
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<u32>() {
                return visitor.visit_u32(n.to());
            }
        }
        Err("expected u32".into())
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<u64>() {
                return visitor.visit_u64(n.to());
            }
        }
        Err("expected u64".into())
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<f32>() {
                return visitor.visit_f32(n.to());
            }
        }
        Err("expected f32".into())
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Num(n)))
        | Some(MetaValue::R(ValueRef::Num(n))) = self.0.take()
        {
            if n.fits::<f64>() {
                return visitor.visit_f64(n.to());
            }
        }
        Err("expected f64".into())
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match (|this: Self| {
            let s = this.0.take();
            let s = match &s {
                Some(MetaValue::O(Value::Str(s))) => s.as_str(),
                Some(MetaValue::R(ValueRef::Str(s))) => s.as_str(),
                _ => return None,
            };
            let s = match s {
                Ok(s) => s,
                Err(_) => return None,
            };
            let mut iter = s.chars();
            // get the first character
            let c = match iter.next() {
                Some(c) => c,
                None => return None,
            };
            // make sure there are no more
            match iter.next() {
                None => Some(c),
                Some(_) => None,
            }
        })(self)
        {
            Some(c) => visitor.visit_char(c),
            None => Err("expected char".into()),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        use crate::value::Utf8Str;
        match self.0.take() {
            Some(MetaValue::O(Value::Str(Utf8Str(data)))) => {
                let data = Vec::from(data);
                match String::from_utf8(data) {
                    Ok(s) => return visitor.visit_string(s),
                    Err(e) => return visitor.visit_byte_buf(e.into_bytes()),
                }
            }
            Some(MetaValue::R(ValueRef::Str(s))) => match s.as_str() {
                Ok(s) => return visitor.visit_borrowed_str(s),
                Err(_) => return visitor.visit_borrowed_bytes(s.as_bytes()),
            },
            _ => (),
        }
        Err("expected str".into())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0.take() {
            Some(MetaValue::O(Value::Bin(data))) => {
                return visitor.visit_byte_buf(Vec::from(data));
            }
            Some(MetaValue::R(ValueRef::Bin(data))) => {
                return visitor.visit_borrowed_bytes(data);
            }
            _ => (),
        }
        Err("expected bytes".into())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match &self.0 {
            Some(MetaValue::O(Value::Nil))
            | Some(MetaValue::R(ValueRef::Nil)) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Some(MetaValue::O(Value::Nil))
        | Some(MetaValue::R(ValueRef::Nil)) = self.0.take()
        {
            return visitor.visit_unit();
        };
        Err("expected unit".into())
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if name == EXT_STRUCT_NAME {
            match self.0.take() {
                Some(MetaValue::O(Value::Ext(t, data))) => {
                    self.0.replace(MetaValue::O(Value::Arr(vec![
                        Value::Num(t.into()),
                        Value::Bin(data),
                    ])));
                }
                Some(MetaValue::R(ValueRef::Ext(t, data))) => {
                    self.0.replace(MetaValue::R(ValueRef::Arr(vec![
                        ValueRef::Num(t.into()),
                        ValueRef::Bin(data),
                    ])));
                }
                Some(oth) => {
                    self.0.replace(oth);
                }
                _ => (),
            }
        }

        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.0.take() {
            Some(MetaValue::O(Value::Arr(arr))) => {
                visitor.visit_seq(Seq(arr.into_iter().map(|v| MetaValue::O(v))))
            }
            Some(MetaValue::R(ValueRef::Arr(arr))) => {
                visitor.visit_seq(Seq(arr.into_iter().map(|v| MetaValue::R(v))))
            }
            _ => Err("expected seq".into()),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // bit of a hack - convert the map (k, v) tuples into just a flat
        // sequence so we can use the same access iterator
        match self.0.take() {
            Some(MetaValue::O(Value::Map(map))) => visitor.visit_seq(Seq(map
                .into_iter()
                .map(|(k, v)| [MetaValue::O(k), MetaValue::O(v)])
                .flatten())),
            Some(MetaValue::R(ValueRef::Map(map))) => {
                visitor.visit_seq(Seq(map
                    .into_iter()
                    .map(|(k, v)| [MetaValue::R(k), MetaValue::R(v)])
                    .flatten()))
            }
            _ => Err("expected map".into()),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        use de::IntoDeserializer;
        match self.0.take() {
            Some(MetaValue::O(Value::Str(s))) => {
                if let Ok(s) = s.as_str() {
                    return visitor.visit_enum(s.into_deserializer());
                }
            }
            Some(MetaValue::R(ValueRef::Str(s))) => {
                if let Ok(s) = s.as_str() {
                    return visitor.visit_enum(s.into_deserializer());
                }
            }
            Some(MetaValue::O(Value::Map(mut map))) => {
                if map.len() == 1 {
                    let (k, v) = map.remove(0);
                    return visitor.visit_enum(Enum(
                        Some(MetaValue::O(k)),
                        Some(MetaValue::O(v)),
                    ));
                }
            }
            Some(MetaValue::R(ValueRef::Map(mut map))) => {
                if map.len() == 1 {
                    let (k, v) = map.remove(0);
                    return visitor.visit_enum(Enum(
                        Some(MetaValue::R(k)),
                        Some(MetaValue::R(v)),
                    ));
                }
            }
            _ => (),
        }
        Err("expected enum".into())
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct Seq<'de, I: Iterator<Item = MetaValue<'de>>>(I);

impl<'de, I: Iterator<Item = MetaValue<'de>>> de::SeqAccess<'de>
    for Seq<'de, I>
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.0.next() {
            None => Ok(None),
            Some(v) => {
                let mut d = DeserializerSync(Some(v));
                seed.deserialize(&mut d).map(Some)
            }
        }
    }
}

// this is a bit of a hack... but DRY : )
impl<'de, I: Iterator<Item = MetaValue<'de>>> de::MapAccess<'de>
    for Seq<'de, I>
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        de::SeqAccess::next_element_seed(self, seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        match de::SeqAccess::next_element_seed(self, seed)? {
            Some(v) => Ok(v),
            None => Err("expected value".into()),
        }
    }
}

struct Enum<'de>(Option<MetaValue<'de>>, Option<MetaValue<'de>>);

impl<'de> de::EnumAccess<'de> for Enum<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let key = seed.deserialize(&mut DeserializerSync(self.0.take()))?;
        Ok((key, self))
    }
}

impl<'de> de::VariantAccess<'de> for Enum<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        // just ignoring any value that might have been placed here
        Ok(())
    }

    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut DeserializerSync(self.1.take()))
    }

    fn tuple_variant<V>(mut self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(
            &mut DeserializerSync(self.1.take()),
            visitor,
        )
    }

    fn struct_variant<V>(
        mut self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(
            &mut DeserializerSync(self.1.take()),
            visitor,
        )
    }
}

#[cfg(test)]
mod de_tests;
