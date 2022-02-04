//! decode library code

use crate::const_::*;
use crate::num::*;

/// MessagePack Rust length markers come in these varieties
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LenType {
    /// Indicates the length of bytes following represent binary data
    Bin,

    /// Indicates the length of bytes following represent utf8 string data
    Str,

    /// Indicates an array of length count message pack objects
    Arr,

    /// Indicates a map of length count message pack key value pairs
    Map,

    /// Indicates the length of bytes following represent msgpack ext data
    Ext(i8),
}

/// MessagePack Rust decoded message pack tokens
#[derive(Clone, Copy, PartialEq)]
pub enum Token<'lt> {
    /// Indicates incomplete binary data for Bin, Str, or Ext tokens.
    /// The second tuple field (the u32 value) is the remaining length
    BinCont(&'lt [u8], u32),

    /// Indicates completed binary data for Bin, Str, or Ext tokens
    Bin(&'lt [u8]),

    /// A MessagePack length marker identifies an amount of something,
    /// see the LenType for what that something is
    Len(LenType, u32),

    /// MesagePack 'Nil' type
    Nil,

    /// A boolean value
    Bool(bool),

    /// A number value
    Num(Num),
}

impl core::fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Token::BinCont(b, r) => {
                write!(f, "BinCont({} bytes, {} remain)", b.len(), r)
            }
            Token::Bin(b) => write!(f, "Bin({} bytes)", b.len()),
            Token::Len(t, l) => write!(f, "Len({:?}, {} bytes)", t, l),
            Token::Nil => f.write_str("Nil"),
            Token::Bool(b) => write!(f, "Bool({})", b),
            Token::Num(n) => write!(f, "Num({:?})", n),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PartialStore<const N: usize>(u8, [u8; N]);

impl<const N: usize> PartialStore<N> {
    fn new() -> Self {
        Self(0, [0; N])
    }

    fn len(&self) -> u32 {
        self.0 as u32
    }

    fn push(&mut self, bytes: &[u8]) {
        let cursor = self.0 as usize;
        let l = bytes.len();
        self.1[cursor..cursor + l].copy_from_slice(bytes);
        self.0 += l as u8;
    }
}

impl PartialStore<2> {
    fn as_u16(&self) -> u16 {
        u16::from_be_bytes(self.1)
    }

    fn as_i16(&self) -> i16 {
        i16::from_be_bytes(self.1)
    }
}

impl PartialStore<4> {
    fn as_u32(&self) -> u32 {
        u32::from_be_bytes(self.1)
    }

    fn as_i32(&self) -> i32 {
        i32::from_be_bytes(self.1)
    }

    fn as_f32(&self) -> f32 {
        f32::from_be_bytes(self.1)
    }
}

impl PartialStore<8> {
    fn as_u64(&self) -> u64 {
        u64::from_be_bytes(self.1)
    }

    fn as_i64(&self) -> i64 {
        i64::from_be_bytes(self.1)
    }

    fn as_f64(&self) -> f64 {
        f64::from_be_bytes(self.1)
    }
}

#[derive(Debug, Clone, Copy)]
enum PendType {
    Len(LenType),
    ExtLen,
    Ext(u32),
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

/// internal decoder state enum
#[derive(Debug, Clone, Copy)]
enum DecState {
    /// default decoder state, waiting for top-level marker
    WantMarker,

    /// Special case for zero length data requests
    WantBinZero,

    /// we need this length more bin data
    WantBin(u32),

    /// awaiting 1 byte of pending internal data
    Pend8(PendType),

    /// awaiting 2 bytes of pending internal data
    Pend16(PendType, PartialStore<2>),

    /// awaiting 4 bytes of pending internal data
    Pend32(PendType, PartialStore<4>),

    /// awaiting 8 bytes of pending internal data
    Pend64(PendType, PartialStore<8>),
}

impl DecState {
    fn next_bytes_min(&self) -> u32 {
        use DecState::*;
        match self {
            WantMarker => 1,
            WantBinZero => 0,
            WantBin(l) => *l,
            Pend8(_) => 1,
            Pend16(_, p) => 2 - p.len(),
            Pend32(_, p) => 4 - p.len(),
            Pend64(_, p) => 8 - p.len(),
        }
    }
}

/// MessagePack Rust Decoder
pub struct Decoder {
    state: DecState,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            state: DecState::WantMarker,
        }
    }
}

impl Decoder {
    /// Default constructor for Decoder
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the minimum bytes required to do the next atomic decode.
    /// Note the decoder will work fine if you pass less or more,
    /// but it may result in a partial decode requiring you to do some
    /// memory copying to read a string, for example.
    pub fn next_bytes_min(&self) -> u32 {
        self.state.next_bytes_min()
    }

    /// Parse a length of encoded messagepack binary data into
    /// an iterator of Token tokens
    pub fn parse<'dec, 'buf>(
        &'dec mut self,
        data: &'buf [u8],
    ) -> TokenIter<'dec, 'buf> {
        TokenIter {
            dec: self,
            data,
            cursor: 0,
        }
    }

    // -- private -- //

    fn set_want_bin_data(&mut self, len: u32) {
        if len == 0 {
            self.state = DecState::WantBinZero;
        } else {
            self.state = DecState::WantBin(len);
        }
    }
}

/// Token Iterator returned from parse
pub struct TokenIter<'dec, 'buf> {
    dec: &'dec mut Decoder,
    data: &'buf [u8],
    cursor: usize,
}

impl<'dec, 'buf> TokenIter<'dec, 'buf> {
    /// get a byte or none if end of buffer
    fn get_byte(&mut self) -> Option<u8> {
        if self.cursor >= self.data.len() {
            None
        } else {
            self.cursor += 1;
            Some(self.data[self.cursor - 1])
        }
    }

    /// get a number of bytes (may be less than asked for)
    /// or none if there are not any bytes in the buffer
    fn get_bytes(&mut self, len: u32) -> Option<&'buf [u8]> {
        let data_len = self.data.len();
        if self.cursor >= data_len {
            None
        } else {
            let rem_len = data_len - self.cursor;
            let len = core::cmp::min(
                u32::MAX as usize,
                core::cmp::min(len as usize, rem_len),
            );
            let out = &self.data[self.cursor..self.cursor + len];
            self.cursor += len;
            Some(out)
        }
    }

    /// parse a want marker, will either return a token
    /// or tail recurse call back into self.next()
    fn parse_want_marker(&mut self) -> Option<Token<'buf>> {
        const FIXSTR_SIZE: u8 = 0x1f;
        const FIXARR_SIZE: u8 = 0x0f;
        const FIXMAP_SIZE: u8 = 0x0f;

        match self.get_byte()? {
            // positive fixint
            m @ 0x00..=0x7f => Some(Token::Num(m.into())),
            // fixmap
            m @ 0x80..=0x8f => {
                Some(Token::Len(LenType::Map, (m & FIXMAP_SIZE) as u32))
            }
            // fixarray
            m @ 0x90..=0x9f => {
                Some(Token::Len(LenType::Arr, (m & FIXARR_SIZE) as u32))
            }
            // fixstr
            m @ 0xa0..=0xbf => {
                let len = (m & FIXSTR_SIZE) as u32;
                self.dec.set_want_bin_data(len);
                Some(Token::Len(LenType::Str, len))
            }
            C_NIL => Some(Token::Nil),
            // reserved (this should never be used... treat it like nil)
            C_RES => Some(Token::Nil),
            C_FALSE => Some(Token::Bool(false)),
            C_TRUE => Some(Token::Bool(true)),
            C_BIN8 => {
                self.dec.state = DecState::Pend8(PendType::Len(LenType::Bin));
                self.next()
            }
            C_BIN16 => {
                self.dec.state = DecState::Pend16(
                    PendType::Len(LenType::Bin),
                    PartialStore::new(),
                );
                self.next()
            }
            C_BIN32 => {
                self.dec.state = DecState::Pend32(
                    PendType::Len(LenType::Bin),
                    PartialStore::new(),
                );
                self.next()
            }
            C_EXT8 => {
                self.dec.state = DecState::Pend8(PendType::ExtLen);
                self.next()
            }
            C_EXT16 => {
                self.dec.state =
                    DecState::Pend16(PendType::ExtLen, PartialStore::new());
                self.next()
            }
            C_EXT32 => {
                self.dec.state =
                    DecState::Pend32(PendType::ExtLen, PartialStore::new());
                self.next()
            }
            C_F32 => {
                self.dec.state =
                    DecState::Pend32(PendType::F32, PartialStore::new());
                self.next()
            }
            C_F64 => {
                self.dec.state =
                    DecState::Pend64(PendType::F64, PartialStore::new());
                self.next()
            }
            C_U8 => {
                self.dec.state = DecState::Pend8(PendType::U8);
                self.next()
            }
            C_U16 => {
                self.dec.state =
                    DecState::Pend16(PendType::U16, PartialStore::new());
                self.next()
            }
            C_U32 => {
                self.dec.state =
                    DecState::Pend32(PendType::U32, PartialStore::new());
                self.next()
            }
            C_U64 => {
                self.dec.state =
                    DecState::Pend64(PendType::U64, PartialStore::new());
                self.next()
            }
            C_I8 => {
                self.dec.state = DecState::Pend8(PendType::I8);
                self.next()
            }
            C_I16 => {
                self.dec.state =
                    DecState::Pend16(PendType::I16, PartialStore::new());
                self.next()
            }
            C_I32 => {
                self.dec.state =
                    DecState::Pend32(PendType::I32, PartialStore::new());
                self.next()
            }
            C_I64 => {
                self.dec.state =
                    DecState::Pend64(PendType::I64, PartialStore::new());
                self.next()
            }
            C_FIXEXT1 => {
                self.dec.state = DecState::Pend8(PendType::Ext(1));
                self.next()
            }
            C_FIXEXT2 => {
                self.dec.state = DecState::Pend8(PendType::Ext(2));
                self.next()
            }
            C_FIXEXT4 => {
                self.dec.state = DecState::Pend8(PendType::Ext(4));
                self.next()
            }
            C_FIXEXT8 => {
                self.dec.state = DecState::Pend8(PendType::Ext(8));
                self.next()
            }
            C_FIXEXT16 => {
                self.dec.state = DecState::Pend8(PendType::Ext(16));
                self.next()
            }
            C_STR8 => {
                self.dec.state = DecState::Pend8(PendType::Len(LenType::Str));
                self.next()
            }
            C_STR16 => {
                self.dec.state = DecState::Pend16(
                    PendType::Len(LenType::Str),
                    PartialStore::new(),
                );
                self.next()
            }
            C_STR32 => {
                self.dec.state = DecState::Pend32(
                    PendType::Len(LenType::Str),
                    PartialStore::new(),
                );
                self.next()
            }
            C_ARR16 => {
                self.dec.state = DecState::Pend16(
                    PendType::Len(LenType::Arr),
                    PartialStore::new(),
                );
                self.next()
            }
            C_ARR32 => {
                self.dec.state = DecState::Pend32(
                    PendType::Len(LenType::Arr),
                    PartialStore::new(),
                );
                self.next()
            }
            C_MAP16 => {
                self.dec.state = DecState::Pend16(
                    PendType::Len(LenType::Map),
                    PartialStore::new(),
                );
                self.next()
            }
            C_MAP32 => {
                self.dec.state = DecState::Pend32(
                    PendType::Len(LenType::Map),
                    PartialStore::new(),
                );
                self.next()
            }
            // negative fixint
            m @ 0xe0..=0xff => Some(Token::Num((m as i8).into())),
        }
    }

    /// parse binary data of given length out of the available buffer
    fn parse_want_bin_data(&mut self, len: u32) -> Option<Token<'buf>> {
        let bytes = match self.get_bytes(len) {
            None => {
                self.dec.set_want_bin_data(len);
                return None;
            }
            Some(b) => b,
        };
        if bytes.len() == len as usize {
            Some(Token::Bin(bytes))
        } else {
            self.dec.set_want_bin_data(len - bytes.len() as u32);
            Some(Token::BinCont(bytes, len - bytes.len() as u32))
        }
    }

    /// We got a length, set up our state
    /// appropriate to the specific length type
    fn parse_got_len(&mut self, t: LenType, len: u32) -> Option<Token<'buf>> {
        use LenType::*;
        match t {
            Bin | Str | Ext(_) => {
                self.dec.set_want_bin_data(len);
            }
            _ => (),
        }
        Some(Token::Len(t, len))
    }

    /// in this ext case we still need to read the type byte
    fn parse_ext_len(&mut self, len: u32) -> Option<Token<'buf>> {
        self.dec.state = DecState::Pend8(PendType::Ext(len));
        self.next()
    }

    /// in this ext case we have already read the type byte
    fn parse_ext(&mut self, t: i8, len: u32) -> Option<Token<'buf>> {
        self.dec.set_want_bin_data(len);
        Some(Token::Len(LenType::Ext(t), len))
    }

    /// We are awaiting a single byte, try to read that byte
    /// and delegate given the current interal pend type state.
    fn parse_pend_8(&mut self, t: PendType) -> Option<Token<'buf>> {
        let b = match self.get_byte() {
            None => {
                self.dec.state = DecState::Pend8(t);
                return None;
            }
            Some(b) => b,
        };
        match t {
            PendType::Len(t) => self.parse_got_len(t, b as u32),
            PendType::ExtLen => self.parse_ext_len(b as u32),
            PendType::Ext(len) => self.parse_ext(b as i8, len),
            PendType::U8 => Some(Token::Num(b.into())),
            PendType::I8 => Some(Token::Num((b as i8).into())),
            _ => unreachable!(),
        }
    }

    /// We are awaiting two bytes, try to read those bytes
    /// and delegate given the current interal pend type state.
    fn parse_pend_16(
        &mut self,
        t: PendType,
        mut p: PartialStore<2>,
    ) -> Option<Token<'buf>> {
        let bytes = match self.get_bytes(2 - p.len()) {
            None => {
                self.dec.state = DecState::Pend16(t, p);
                return None;
            }
            Some(b) => b,
        };
        p.push(bytes);
        if p.len() == 2 {
            match t {
                PendType::Len(t) => self.parse_got_len(t, p.as_u16() as u32),
                PendType::ExtLen => self.parse_ext_len(p.as_u16() as u32),
                PendType::U16 => Some(Token::Num(p.as_u16().into())),
                PendType::I16 => Some(Token::Num(p.as_i16().into())),
                _ => unreachable!(),
            }
        } else {
            self.dec.state = DecState::Pend16(t, p);
            None
        }
    }

    /// We are awaiting four bytes, try to read those bytes
    /// and delegate given the current interal pend type state.
    fn parse_pend_32(
        &mut self,
        t: PendType,
        mut p: PartialStore<4>,
    ) -> Option<Token<'buf>> {
        let bytes = match self.get_bytes(4 - p.len()) {
            None => {
                self.dec.state = DecState::Pend32(t, p);
                return None;
            }
            Some(b) => b,
        };
        p.push(bytes);
        if p.len() == 4 {
            match t {
                PendType::Len(t) => self.parse_got_len(t, p.as_u32()),
                PendType::ExtLen => self.parse_ext_len(p.as_u32()),
                PendType::U32 => Some(Token::Num(p.as_u32().into())),
                PendType::I32 => Some(Token::Num(p.as_i32().into())),
                PendType::F32 => Some(Token::Num(p.as_f32().into())),
                _ => unreachable!(),
            }
        } else {
            self.dec.state = DecState::Pend32(t, p);
            None
        }
    }

    /// We are awaiting eight bytes, try to read those bytes
    /// and delegate given the current interal pend type state.
    fn parse_pend_64(
        &mut self,
        t: PendType,
        mut p: PartialStore<8>,
    ) -> Option<Token<'buf>> {
        let bytes = match self.get_bytes(8 - p.len()) {
            None => {
                self.dec.state = DecState::Pend64(t, p);
                return None;
            }
            Some(b) => b,
        };
        p.push(bytes);
        if p.len() == 8 {
            match t {
                PendType::U64 => Some(Token::Num(p.as_u64().into())),
                PendType::I64 => Some(Token::Num(p.as_i64().into())),
                PendType::F64 => Some(Token::Num(p.as_f64().into())),
                _ => unreachable!(),
            }
        } else {
            self.dec.state = DecState::Pend64(t, p);
            None
        }
    }
}

impl<'dec, 'buf> core::iter::Iterator for TokenIter<'dec, 'buf> {
    type Item = Token<'buf>;

    fn next(&mut self) -> Option<Self::Item> {
        match core::mem::replace(&mut self.dec.state, DecState::WantMarker) {
            DecState::WantMarker => self.parse_want_marker(),
            DecState::WantBinZero => Some(Token::Bin(&[])),
            DecState::WantBin(len) => self.parse_want_bin_data(len),
            DecState::Pend8(t) => self.parse_pend_8(t),
            DecState::Pend16(t, p) => self.parse_pend_16(t, p),
            DecState::Pend32(t, p) => self.parse_pend_32(t, p),
            DecState::Pend64(t, p) => self.parse_pend_64(t, p),
        }
    }
}
