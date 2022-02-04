//! MessagePack Rust Value and ValueRef types

use crate::*;

/// MessagePack Rust owned Value type
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
    /// TODO FIX THIS!!!
    Str(Box<[u8]>),

    /// MessagePack `Arr` type
    Arr(Vec<Value>),

    /// MessagePack `Map` type
    Map(Vec<(Value, Value)>),

    /// MessagePack `Ext` type
    Ext(i8, Box<[u8]>),
}

/// MessagePack Rust Value Reference type
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
    /// TODO FIX THIS!!!
    Str(&'lt [u8]),

    /// MessagePack `Arr` type
    Arr(Vec<ValueRef<'lt>>),

    /// MessagePack `Map` type
    Map(Vec<(ValueRef<'lt>, ValueRef<'lt>)>),

    /// MessagePack `Ext` type
    Ext(i8, &'lt [u8]),
}

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
            Some(Len(LenType::Bin, l)) => {
                if let Some(Bin(data)) = self.iter.next() {
                    if data.len() == l as usize {
                        return Ok(ValueRef::Bin(data));
                    }
                }
                panic!();
            }
            Some(Len(LenType::Str, l)) => {
                if let Some(Bin(data)) = self.iter.next() {
                    if data.len() == l as usize {
                        return Ok(ValueRef::Str(data));
                    }
                }
                panic!();
            }
            Some(Len(LenType::Ext(ext_type), l)) => {
                if let Some(Bin(data)) = self.iter.next() {
                    if data.len() == l as usize {
                        return Ok(ValueRef::Ext(ext_type, data));
                    }
                }
                panic!();
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
            _ => panic!(),
        }
    }
}

impl<'lt> ValueRef<'lt> {
    /// Convert to an owned Value type
    pub fn to_owned(&self) -> Value {
        unimplemented!()
    }

    /// Decode a ValueRef from a byte array reference
    pub fn from_slice(data: &'lt [u8]) -> Result<Self> {
        Self::from_slice_config(data, Config::default())
    }

    /// Decode a ValueRef from a byte array reference
    pub fn from_slice_config(data: &'lt [u8], _config: Config) -> Result<Self> {
        let mut dec = msgpackin_core::decode::Decoder::new();
        let mut dec = VRDecode {
            iter: dec.parse(data),
        };

        dec.next_val()
    }
}
