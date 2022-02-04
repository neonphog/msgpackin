//! encode library code

use crate::const_::*;
use crate::num::*;
use core::ops::Deref;

/// MessagePack Rust variable-size byte array result
#[derive(Debug, Clone, Copy)]
pub struct VarBytes(VbPriv);

impl core::ops::Deref for VarBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl core::convert::AsRef<[u8]> for VarBytes {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

impl core::borrow::Borrow<[u8]> for VarBytes {
    fn borrow(&self) -> &[u8] {
        self.deref()
    }
}

#[derive(Debug, Clone, Copy)]
enum VbPriv {
    /// 1-length byte array
    B1([u8; 1]),

    /// 2-length byte array
    B2([u8; 2]),

    /// 3-length byte array
    B3([u8; 3]),

    /// 4-length byte array
    B4([u8; 4]),

    /// 5-length byte array
    B5([u8; 5]),

    /// 6-length byte array
    B6([u8; 6]),

    /// 9-length byte array
    B9([u8; 9]),
}

macro_rules! _bf {
    ($($t:expr => $s:literal,)*) => {$(
        impl From<[u8; $s]> for VarBytes {
            fn from(t: [u8; $s]) -> Self {
                Self($t(t))
            }
        }
    )*};
}

_bf! {
    VbPriv::B1 => 1,
    VbPriv::B2 => 2,
    VbPriv::B3 => 3,
    VbPriv::B4 => 4,
    VbPriv::B5 => 5,
    VbPriv::B6 => 6,
    VbPriv::B9 => 9,
}

impl core::ops::Deref for VbPriv {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        use VbPriv::*;
        match self {
            B1(b) => b,
            B2(b) => b,
            B3(b) => b,
            B4(b) => b,
            B5(b) => b,
            B6(b) => b,
            B9(b) => b,
        }
    }
}

/// MessagePack Rust Encoder
pub struct Encoder;

impl Default for Encoder {
    fn default() -> Self {
        Self
    }
}

impl Encoder {
    /// Default constructor for Encoder
    pub fn new() -> Self {
        Self::default()
    }

    /// Encode msgpack bytes for `nil`
    pub fn enc_nil(&mut self) -> VarBytes {
        [C_NIL].into()
    }

    /// Encode msgpack bytes for `bool`
    pub fn enc_bool(&mut self, b: bool) -> VarBytes {
        if b {
            [C_TRUE].into()
        } else {
            [C_FALSE].into()
        }
    }

    /// Encode msgpack bytes for msgpack `Num` type
    pub fn enc_num<N: Into<Num>>(&mut self, n: N) -> VarBytes {
        let i = match n.into() {
            Num::F32(f) => {
                let mut out = [C_F32, 0, 0, 0, 0];
                out[1..5].copy_from_slice(&f.to_be_bytes());
                return out.into();
            }
            Num::F64(f) => {
                let mut out = [C_F64, 0, 0, 0, 0, 0, 0, 0, 0];
                out[1..9].copy_from_slice(&f.to_be_bytes());
                return out.into();
            }
            Num::Signed(i) => i as i128,
            Num::Unsigned(u) => u as i128,
        };

        #[allow(clippy::manual_range_contains)]
        if i >= 0 && i < 128 {
            [i as u8].into()
        } else if i > -32 && i < 0 {
            [i as i8 as u8].into()
        } else if i >= i8::MIN as i128 && i <= i8::MAX as i128 {
            [C_I8, i as u8].into()
        } else if i >= i16::MIN as i128 && i <= i16::MAX as i128 {
            let mut out = [C_I16, 0, 0];
            out[1..3].copy_from_slice(&(i as i16).to_be_bytes());
            out.into()
        } else if i >= i32::MIN as i128 && i <= i32::MAX as i128 {
            let mut out = [C_I32, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&(i as i32).to_be_bytes());
            out.into()
        } else if i <= i64::MAX as i128 {
            let mut out = [C_I64, 0, 0, 0, 0, 0, 0, 0, 0];
            out[1..9].copy_from_slice(&(i as i64).to_be_bytes());
            out.into()
        } else if i <= u8::MAX as i128 {
            [C_U8, i as u8].into()
        } else if i <= u16::MAX as i128 {
            let mut out = [C_U16, 0, 0];
            out[1..3].copy_from_slice(&(i as u16).to_be_bytes());
            out.into()
        } else if i <= u32::MAX as i128 {
            let mut out = [C_U32, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&(i as u32).to_be_bytes());
            out.into()
        } else {
            let mut out = [C_U64, 0, 0, 0, 0, 0, 0, 0, 0];
            out[1..9].copy_from_slice(&(i as u64).to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for arbitrary binary byte length.
    /// There is no encode function for the bytes themselves,
    /// just copy them directly into your buffer
    pub fn enc_bin_len(&mut self, len: u32) -> VarBytes {
        if len < 256 {
            [C_BIN8, len as u8].into()
        } else if len < 65536 {
            let mut out = [C_BIN16, 0, 0];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [C_BIN32, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for utf8 string data byte length.
    /// There is no encode function for the bytes themselves,
    /// just copy them directly into your buffer (`as_bytes()`)
    pub fn enc_str_len(&mut self, len: u32) -> VarBytes {
        if len < 32 {
            [C_FIXSTR0 | (len as u8 & 0x1f)].into()
        } else if len < 256 {
            [C_STR8, len as u8].into()
        } else if len < 65536 {
            let mut out = [C_STR16, 0, 0];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [C_STR32, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for array marker / length
    pub fn enc_arr_len(&mut self, len: u32) -> VarBytes {
        if len < 16 {
            [C_FIXARR0 | (len as u8 & 0x0f)].into()
        } else if len < 65536 {
            let mut out = [C_ARR16, 0, 0];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [C_ARR32, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for map marker / length.
    /// This should be the number of key/value pairs in the map
    pub fn enc_map_len(&mut self, len: u32) -> VarBytes {
        if len < 16 {
            [C_FIXMAP0 | (len as u8 & 0x0f)].into()
        } else if len < 65536 {
            let mut out = [C_MAP16, 0, 0];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [C_MAP32, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for arbitrary msgpack ext byte length.
    /// There is no encode function for the bytes themselves,
    /// just copy them directly into your buffer
    pub fn enc_ext_len(&mut self, len: u32, t: i8) -> VarBytes {
        if len == 1 {
            [C_FIXEXT1, t as u8].into()
        } else if len == 2 {
            [C_FIXEXT2, t as u8].into()
        } else if len == 4 {
            [C_FIXEXT4, t as u8].into()
        } else if len == 8 {
            [C_FIXEXT8, t as u8].into()
        } else if len == 16 {
            [C_FIXEXT16, t as u8].into()
        } else if len < 256 {
            [C_EXT8, len as u8, t as u8].into()
        } else if len < 65536 {
            let mut out = [C_EXT16, 0, 0, t as u8];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [C_EXT32, 0, 0, 0, 0, t as u8];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }
}
