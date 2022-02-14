//! serde Serializer implementations

use crate::consumer::*;
use crate::*;
use serde::{ser, Serialize};

/// Serialize to a `Vec<u8>`
pub fn to_bytes<T>(t: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_bytes_config(t, Config::default())
}

/// Serialize to a `Vec<u8>`
pub fn to_bytes_config<T>(t: &T, config: Config) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let mut out = Vec::new();
    to_sync_config(t, &mut out, config)?;
    Ok(out)
}

/// Serialize synchronously to anything that can be converted into a
/// `DynConsumerSync`, e.g. `Write`
pub fn to_sync<'lt, T, C>(t: &T, c: C) -> Result<()>
where
    T: Serialize + ?Sized,
    C: Into<DynConsumerSync<'lt>>,
{
    let c = c.into();
    t.serialize(&mut SerializerSync::new(Config::default(), c).as_ref())
}

/// Serialize synchronously to anything that can be converted into a
/// `DynConsumerSync`, e.g. `Write`
pub fn to_sync_config<'lt, T, C>(t: &T, c: C, config: Config) -> Result<()>
where
    T: Serialize + ?Sized,
    C: Into<DynConsumerSync<'lt>>,
{
    let c = c.into();
    t.serialize(&mut SerializerSync::new(config, c).as_ref())
}

/// Serialize asynchronously to anything that can be converted into a
/// `DynConsumerAsync`, e.g. `AsyncWrite`.
/// Note, as serde only supplies a synchronous api for now, this function
/// will buffer the serialized bytes first, then write them to the async
/// consumer
pub async fn to_async<'lt, T, C>(t: &T, c: C) -> Result<()>
where
    T: Serialize + ?Sized,
    C: Into<DynConsumerAsync<'lt>>,
{
    to_async_config(t, c, Config::default()).await
}

/// Serialize asynchronously to anything that can be converted into a
/// `DynConsumerAsync`, e.g. `AsyncWrite`.
/// Note, as serde only supplies a synchronous api for now, this function
/// will buffer the serialized bytes first, then write them to the async
/// consumer
pub async fn to_async_config<'lt, T, C>(
    t: &T,
    c: C,
    config: Config,
) -> Result<()>
where
    T: Serialize + ?Sized,
    C: Into<DynConsumerAsync<'lt>>,
{
    let mut c = c.into();
    let mut buf = Vec::new();
    t.serialize(&mut SerializerSync::new(config, &mut buf).as_ref())?;
    c.write(&buf).await
}

/// Reference type for a sync serializer
pub struct SerializerSyncRef<'a, 'lt> {
    /// serializer config reference
    pub config: &'a Config,

    /// the current consumer
    pub con: &'a mut DynConsumerSync<'lt>,

    /// the current encoder
    pub enc: &'a mut msgpackin_core::encode::Encoder,
}

/// trait for a sync serializer
pub trait AsSerializerSync<'lt> {
    /// get a reference to this sync serializer
    fn as_ref(&mut self) -> SerializerSyncRef<'_, 'lt>;
}

/// Msgpackin serde SerializerSync
pub struct SerializerSync<'lt> {
    config: Config,
    con: DynConsumerSync<'lt>,
    enc: msgpackin_core::encode::Encoder,
}

impl<'lt> AsSerializerSync<'lt> for SerializerSync<'lt> {
    fn as_ref(&mut self) -> SerializerSyncRef<'_, 'lt> {
        let SerializerSync { config, con, enc } = self;
        SerializerSyncRef { config, con, enc }
    }
}

impl<'lt> SerializerSync<'lt> {
    /// Construct a new SerializerSync for given consumer
    pub fn new<C: Into<DynConsumerSync<'lt>>>(
        config: Config,
        consumer: C,
    ) -> Self {
        Self {
            config,
            con: consumer.into(),
            enc: msgpackin_core::encode::Encoder::new(),
        }
    }
}

impl<'a, 'b, 'lt> ser::Serializer for &'b mut SerializerSyncRef<'a, 'lt> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializerSyncContainer<'a, 'b, 'lt>;
    type SerializeTuple = SerializerSyncContainer<'a, 'b, 'lt>;
    type SerializeTupleStruct = SerializerSyncContainer<'a, 'b, 'lt>;
    type SerializeTupleVariant = SerializerSyncContainer<'a, 'b, 'lt>;
    type SerializeMap = SerializerSyncContainer<'a, 'b, 'lt>;
    type SerializeStruct = SerializerSyncContainer<'a, 'b, 'lt>;
    type SerializeStructVariant = SerializerSyncContainer<'a, 'b, 'lt>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.con.write(&self.enc.enc_bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.con.write(&self.enc.enc_num(v))
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        if v.as_bytes().len() > u32::MAX as usize {
            return Err("str too long".into());
        }
        self.con
            .write(&self.enc.enc_str_len(v.as_bytes().len() as u32))?;
        self.con.write(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        if v.len() > u32::MAX as usize {
            return Err("bin too long".into());
        }
        self.con.write(&self.enc.enc_bin_len(v.len() as u32))?;
        self.con.write(v)
    }

    fn serialize_none(self) -> Result<()> {
        self.con.write(&self.enc.enc_nil())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.con.write(&self.enc.enc_nil())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.con.write(&self.enc.enc_nil())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if name == EXT_STRUCT_NAME {
            let mut buf = Vec::new();
            {
                let SerializerSyncRef {
                    config,
                    con: _,
                    enc,
                } = self;
                let mut tmp_con: DynConsumerSync<'_> = (&mut buf).into();
                let mut r = SerializerSyncRef {
                    config,
                    con: &mut tmp_con,
                    enc,
                };
                value.serialize(&mut r)?;
            }
            match ValueRef::from_ref(buf.as_slice())? {
                ValueRef::Arr(mut arr) => {
                    if arr.len() == 2 {
                        match (arr.remove(0), arr.remove(0)) {
                            (ValueRef::Num(t), ValueRef::Bin(data)) => {
                                if t.fits::<i8>() {
                                    let t: i8 = t.to();
                                    self.con.write(
                                        &self
                                            .enc
                                            .enc_ext_len(data.len() as u32, t),
                                    )?;
                                    return self.con.write(data);
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }
        // fallback to just encoding
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.con.write(&self.enc.enc_map_len(1))?;
        self.serialize_str(variant)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(len) => {
                if len > u32::MAX as usize {
                    return Err("arr too long".into());
                }
                self.con.write(&self.enc.enc_arr_len(len as u32))?;
                Ok(SerializerSyncContainer::priv_new(self, Mode::Dir))
            }
            None => Ok(SerializerSyncContainer::priv_new(
                self,
                Mode::BufArr(0, Vec::new()),
            )),
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.con.write(&self.enc.enc_map_len(1))?;
        self.serialize_str(variant)?;
        self.serialize_tuple(len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        match len {
            Some(len) => {
                if len > u32::MAX as usize {
                    return Err("map too long".into());
                }
                self.con.write(&self.enc.enc_map_len(len as u32))?;
                Ok(SerializerSyncContainer::priv_new(self, Mode::Dir))
            }
            None => Ok(SerializerSyncContainer::priv_new(
                self,
                Mode::BufMap(0, Vec::new()),
            )),
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.con.write(&self.enc.enc_map_len(1))?;
        self.serialize_str(variant)?;
        self.serialize_struct(name, len)
    }
}

enum Mode {
    Dir,
    BufArr(u32, Vec<u8>),
    BufMap(u32, Vec<u8>),
}

/// Serializer for containers like arr/map
pub struct SerializerSyncContainer<'a, 'b, 'lt> {
    ser: &'b mut SerializerSyncRef<'a, 'lt>,
    mode: Mode,
}

impl<'a, 'b, 'lt> SerializerSyncContainer<'a, 'b, 'lt> {
    fn priv_new(ser: &'b mut SerializerSyncRef<'a, 'lt>, mode: Mode) -> Self {
        Self { ser, mode }
    }
}

impl<'a, 'b, 'lt> ser::SerializeSeq for SerializerSyncContainer<'a, 'b, 'lt> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let SerializerSyncContainer { ser, mode } = self;
        match mode {
            Mode::Dir => value.serialize(&mut **ser),
            Mode::BufMap(..) => unreachable!(),
            Mode::BufArr(count, buf) => {
                *count += 1;
                let SerializerSyncRef {
                    config,
                    con: _,
                    enc,
                } = ser;
                let mut tmp_con: DynConsumerSync<'_> = buf.into();
                let mut r = SerializerSyncRef {
                    config,
                    con: &mut tmp_con,
                    enc,
                };
                value.serialize(&mut r)
            }
        }
    }

    fn end(self) -> Result<()> {
        let SerializerSyncContainer { ser, mode } = self;
        match mode {
            Mode::Dir => Ok(()),
            Mode::BufMap(..) => unreachable!(),
            Mode::BufArr(count, buf) => {
                ser.con.write(&ser.enc.enc_arr_len(count))?;
                ser.con.write(&buf)
            }
        }
    }
}

impl<'a, 'b, 'lt> ser::SerializeTuple for SerializerSyncContainer<'a, 'b, 'lt> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, 'b, 'lt> ser::SerializeTupleStruct
    for SerializerSyncContainer<'a, 'b, 'lt>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, 'b, 'lt> ser::SerializeTupleVariant
    for SerializerSyncContainer<'a, 'b, 'lt>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, 'b, 'lt> ser::SerializeMap for SerializerSyncContainer<'a, 'b, 'lt> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let SerializerSyncContainer { ser, mode } = self;
        match mode {
            Mode::Dir => key.serialize(&mut **ser),
            Mode::BufArr(..) => unreachable!(),
            Mode::BufMap(count, buf) => {
                *count += 1;
                let SerializerSyncRef {
                    config,
                    con: _,
                    enc,
                } = ser;
                let mut tmp_con: DynConsumerSync<'_> = buf.into();
                let mut r = SerializerSyncRef {
                    config,
                    con: &mut tmp_con,
                    enc,
                };
                key.serialize(&mut r)
            }
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let SerializerSyncContainer { ser, mode } = self;
        match mode {
            Mode::Dir => value.serialize(&mut **ser),
            Mode::BufArr(..) => unreachable!(),
            Mode::BufMap(_, buf) => {
                let SerializerSyncRef {
                    config,
                    con: _,
                    enc,
                } = ser;
                let mut tmp_con: DynConsumerSync<'_> = buf.into();
                let mut r = SerializerSyncRef {
                    config,
                    con: &mut tmp_con,
                    enc,
                };
                value.serialize(&mut r)
            }
        }
    }

    fn end(self) -> Result<()> {
        let SerializerSyncContainer { ser, mode } = self;
        match mode {
            Mode::Dir => Ok(()),
            Mode::BufArr(..) => unreachable!(),
            Mode::BufMap(count, buf) => {
                ser.con.write(&ser.enc.enc_map_len(count))?;
                ser.con.write(&buf)
            }
        }
    }
}

impl<'a, 'b, 'lt> ser::SerializeStruct
    for SerializerSyncContainer<'a, 'b, 'lt>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

impl<'a, 'b, 'lt> ser::SerializeStructVariant
    for SerializerSyncContainer<'a, 'b, 'lt>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

#[cfg(test)]
mod ser_tests;
