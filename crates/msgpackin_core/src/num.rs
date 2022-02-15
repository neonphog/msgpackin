//! Abstractions for working with numbers in MessagePack Rust

/// Indicates a type to which a MessagePack Rust Num instance can be coverted
pub trait NumTo<T> {
    /// Will this Num instance fit in the target type?
    /// Fits is defined as lossless conversion.
    /// E.g. an `f64` value of `42.0` will fit in all rust data types,
    /// but a `u16` value of `256` will not fit in a `u8`
    fn fits(&self) -> bool;

    /// Convert this Num instance into the destination type.
    /// If this instance is outside the bounds of the target
    /// type, it will be clamped to fit. Check `fits()` first
    /// if this is not desireable
    fn to(&self) -> T;
}

/// A number type that encapsulates what integers and floats can
/// be represented in MessagePack Rust
#[derive(Clone, Copy)]
pub enum Num {
    /// Num is backed by f32 storage.
    F32(f32),

    /// Num is backed by f64 storage.
    F64(f64),

    /// Num is backed by i64 storage.
    Signed(i64),

    /// Num is backed by u64 storage.
    Unsigned(u64),
}

impl core::fmt::Debug for Num {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Num::F32(n) => n.fmt(f),
            Num::F64(n) => n.fmt(f),
            Num::Signed(n) => n.fmt(f),
            Num::Unsigned(n) => n.fmt(f),
        }
    }
}

impl core::fmt::Display for Num {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Num::F32(n) => n.fmt(f),
            Num::F64(n) => n.fmt(f),
            Num::Signed(n) => n.fmt(f),
            Num::Unsigned(n) => n.fmt(f),
        }
    }
}

impl PartialEq for Num {
    fn eq(&self, oth: &Num) -> bool {
        match self {
            Num::F32(f) => oth.eq(f),
            Num::F64(f) => oth.eq(f),
            Num::Signed(i) => oth.eq(i),
            Num::Unsigned(u) => oth.eq(u),
        }
    }
}

macro_rules! p_eq {
    ($($t:ty)*) => {$(
        impl PartialEq<$t> for Num {
            fn eq(&self, oth: &$t) -> bool {
                match self {
                    Num::F32(f) => *oth as f32 == *f && *oth as f32 as $t == *oth,
                    Num::F64(f) => *oth as f64 == *f && *oth as f64 as $t == *oth,
                    Num::Signed(i) => *oth as i64 == *i && *oth as i64 as $t == *oth,
                    Num::Unsigned(u) => *oth as u64 == *u && *oth as u64 as $t == *oth,
                }
            }
        }
    )*};
}

p_eq!( u8 u16 u32 u64 i128 usize i8 i16 i32 i64 u128 isize f32 f64 );

impl Num {
    /// Will this Num instance fit in the target type?
    /// Fits is defined as lossless conversion.
    /// E.g. an `f64` value of `42.0` will fit in all rust data types,
    /// but a `u16` value of `256` will not fit in a `u8`
    pub fn fits<T>(&self) -> bool
    where
        Self: NumTo<T>,
    {
        NumTo::fits(self)
    }

    /// Convert this Num instance into the destination type.
    /// If this instance is outside the bounds of the target
    /// type, it will be clamped to fit. Check `fits()` first
    /// if this is not desireable
    pub fn to<T>(&self) -> T
    where
        Self: NumTo<T>,
    {
        NumTo::to(self)
    }
}

impl From<f32> for Num {
    fn from(t: f32) -> Self {
        if t >= 0.0 && t as u64 as f32 == t {
            Num::Unsigned(t as u64)
        } else if t as i64 as f32 == t {
            Num::Signed(t as i64)
        } else {
            Num::F32(t)
        }
    }
}

impl From<f64> for Num {
    fn from(t: f64) -> Self {
        if t >= 0.0 && t as u64 as f64 == t {
            Num::Unsigned(t as u64)
        } else if t as i64 as f64 == t {
            Num::Signed(t as i64)
        } else if t as f32 as f64 == t {
            Num::F32(t as f32)
        } else {
            Num::F64(t)
        }
    }
}

macro_rules! into_num {
    ($i:ident:$e:expr => $($t:ty)*) => {$(
        impl From<$t> for Num {
            fn from($i: $t) -> Self {
                $e
            }
        }
    )*};
}

into_num!(t:Num::Signed(t as i64) => i8 i16 i32 i64 isize); // NOT i128
into_num!(t:Num::Unsigned(t as u64) => u8 u16 u32 u64 usize); // NOT u128

macro_rules! num_to {
    ($($t:ty)*) => {$(
        impl NumTo<$t> for Num {
            fn fits(&self) -> bool {
                match self {
                    Num::F32(f) => *f as $t as f32 == *f,
                    Num::F64(f) => *f as $t as f64 == *f,
                    Num::Signed(i) => *i as $t as i64 == *i,
                    Num::Unsigned(u) => *u as $t as u64 == *u,
                }
            }

            fn to(&self) -> $t {
                match &self {
                    Num::F32(f) => (*f) as $t,
                    Num::F64(f) => (*f) as $t,
                    Num::Signed(i) => (*i as i128).clamp(
                        <$t>::MIN as i128,
                        <$t>::MAX as i128,
                    ) as $t,
                    Num::Unsigned(u) => (*u as i128).clamp(
                        <$t>::MIN as i128,
                        <$t>::MAX as i128,
                    ) as $t,
                }
            }
        }
    )*};
}

num_to!(u8 u16 u32 u64 usize i8 i16 i32 i64 i128 isize f32 f64);

// have to hand-code this one because it overflows our i128
impl NumTo<u128> for Num {
    fn fits(&self) -> bool {
        match self {
            Num::F32(f) => *f as u128 as f32 == *f,
            Num::F64(f) => *f as u128 as f64 == *f,
            Num::Signed(i) => *i as u128 as i64 == *i,
            Num::Unsigned(u) => *u as u128 as u64 == *u,
        }
    }

    fn to(&self) -> u128 {
        match &self {
            Num::F32(f) => (*f) as u128,
            Num::F64(f) => (*f) as u128,
            Num::Signed(i) => (*i).clamp(0, i64::MAX) as u128,
            Num::Unsigned(u) => (*u) as u128,
        }
    }
}

#[cfg(test)]
mod num_tests {
    use super::*;

    #[test]
    fn test_to() {
        macro_rules! test_to_from {
            ($tt:ty:$($tf:ty)*) => {$({
                let _n: $tt = Num::from(<$tf>::MIN).to();
                let _n: $tt = Num::from(<$tf>::MAX).to();
            })*};
        }

        macro_rules! test_to {
            ($($t:ty)*) => {$(
                test_to_from!($t: u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);
            )*};
        }

        test_to!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);
    }
}
