//! encode library code

/// MessagePack Rust variable-size byte array result
#[derive(Debug, Clone, Copy)]
pub struct VarBytes(VbPriv);

impl core::ops::Deref for VarBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &*self.0
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
    pub fn enc_nil(&mut self) -> [u8; 1] {
        [0xc0]
    }

    /// Encode msgpack bytes for `bool`
    pub fn enc_bool(&mut self, b: bool) -> [u8; 1] {
        if b {
            [0xc3]
        } else {
            [0xc2]
        }
    }

    /// Encode msgpack bytes for `u8`
    pub fn enc_u8(&mut self, u: u8) -> [u8; 2] {
        [0xcc, u]
    }

    /// Encode msgpack bytes for `u16`
    pub fn enc_u16(&mut self, u: u16) -> [u8; 3] {
        let mut out = [0xcd, 0, 0];
        out[1..3].copy_from_slice(&u.to_be_bytes());
        out
    }

    /// Encode msgpack bytes for `u32`
    pub fn enc_u32(&mut self, u: u32) -> [u8; 5] {
        let mut out = [0xce, 0, 0, 0, 0];
        out[1..5].copy_from_slice(&u.to_be_bytes());
        out
    }

    /// Encode msgpack bytes for `u64`
    pub fn enc_u64(&mut self, u: u64) -> [u8; 9] {
        let mut out = [0xcf, 0, 0, 0, 0, 0, 0, 0, 0];
        out[1..9].copy_from_slice(&u.to_be_bytes());
        out
    }

    /// Encode smallest msgpack bytes for an unsigned integer
    pub fn enc_small_uint(&mut self, u: u64) -> VarBytes {
        if u < 128 {
            [u as u8].into()
        } else if u < 256 {
            self.enc_u8(u as u8).into()
        } else if u < 65536 {
            self.enc_u16(u as u16).into()
        } else if u < 4294967296 {
            self.enc_u32(u as u32).into()
        } else {
            self.enc_u64(u).into()
        }
    }

    /// Encode msgpack bytes for `i8`
    pub fn enc_i8(&mut self, i: i8) -> [u8; 2] {
        [0xd0, i as u8]
    }

    /// Encode msgpack bytes for `i16`
    pub fn enc_i16(&mut self, i: i16) -> [u8; 3] {
        let mut out = [0xd1, 0, 0];
        out[1..3].copy_from_slice(&i.to_be_bytes());
        out
    }

    /// Encode msgpack bytes for `i32`
    pub fn enc_i32(&mut self, i: i32) -> [u8; 5] {
        let mut out = [0xd2, 0, 0, 0, 0];
        out[1..5].copy_from_slice(&i.to_be_bytes());
        out
    }

    /// Encode msgpack bytes for `i64`
    pub fn enc_i64(&mut self, i: i64) -> [u8; 9] {
        let mut out = [0xd3, 0, 0, 0, 0, 0, 0, 0, 0];
        out[1..9].copy_from_slice(&i.to_be_bytes());
        out
    }

    /// Encode smallest msgpack bytes for a signed integer
    pub fn enc_small_int(&mut self, i: i64) -> VarBytes {
        if i >= 0 {
            self.enc_small_uint(i as u64)
        } else if i > -32 {
            [i as i8 as u8].into()
        } else if i > -128 {
            self.enc_i8(i as i8).into()
        } else if i > -32768 {
            self.enc_i16(i as i16).into()
        } else if i > -2147483648 {
            self.enc_i32(i as i32).into()
        } else {
            self.enc_i64(i).into()
        }
    }

    /// Encode msgpack bytes for `f32`
    pub fn enc_f32(&mut self, f: f32) -> [u8; 5] {
        let mut out = [0xca, 0, 0, 0, 0];
        out[1..5].copy_from_slice(&f.to_be_bytes());
        out
    }

    /// Encode msgpack bytes for `f64`
    pub fn enc_f64(&mut self, f: f64) -> [u8; 9] {
        let mut out = [0xcb, 0, 0, 0, 0, 0, 0, 0, 0];
        out[1..9].copy_from_slice(&f.to_be_bytes());
        out
    }

    /// Encode msgpack bytes for arbitrary binary byte length.
    /// There is no encode function for the bytes themselves,
    /// just copy them directly into your buffer
    pub fn enc_bin_len(&mut self, len: u32) -> VarBytes {
        if len < 256 {
            [0xc4, len as u8].into()
        } else if len < 65536 {
            let mut out = [0xc5, 0, 0];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [0xc6, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for utf8 string data byte length.
    /// There is no encode function for the bytes themselves,
    /// just copy them directly into your buffer (`as_bytes()`)
    pub fn enc_str_len(&mut self, len: u32) -> VarBytes {
        if len < 32 {
            [0xa0 | (len as u8 & 0x1f)].into()
        } else if len < 256 {
            [0xd9, len as u8].into()
        } else if len < 65536 {
            let mut out = [0xda, 0, 0];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [0xdb, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for array marker / length
    pub fn enc_arr_len(&mut self, len: u32) -> VarBytes {
        if len < 16 {
            [0x90 | (len as u8 & 0x0f)].into()
        } else if len < 65536 {
            let mut out = [0xd9, 0, 0];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [0xdd, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for map marker / length.
    /// This should be the number of key/value pairs in the map
    pub fn enc_map_len(&mut self, len: u32) -> VarBytes {
        if len < 16 {
            [0x80 | (len as u8 & 0x0f)].into()
        } else if len < 65536 {
            let mut out = [0xde, 0, 0];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [0xdf, 0, 0, 0, 0];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }

    /// Encode msgpack bytes for arbitrary msgpack ext byte length.
    /// There is no encode function for the bytes themselves,
    /// just copy them directly into your buffer
    pub fn enc_ext_len(&mut self, len: u32, t: i8) -> VarBytes {
        if len == 1 {
            [0xd4, t as u8].into()
        } else if len == 2 {
            [0xd5, t as u8].into()
        } else if len == 4 {
            [0xd6, t as u8].into()
        } else if len == 8 {
            [0xd7, t as u8].into()
        } else if len == 16 {
            [0xd8, t as u8].into()
        } else if len < 256 {
            [0xd8, len as u8, t as u8].into()
        } else if len < 65536 {
            let mut out = [0xc9, 0, 0, t as u8];
            out[1..3].copy_from_slice(&(len as u16).to_be_bytes());
            out.into()
        } else {
            let mut out = [0xc9, 0, 0, 0, 0, t as u8];
            out[1..5].copy_from_slice(&len.to_be_bytes());
            out.into()
        }
    }
}
