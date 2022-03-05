//! A Rust library to read binary MIDAS files.
//!
//! Midasio provides utilities to iterate over the MIDAS events in a file, iterate over the data
//! banks in a MIDAS event, and extract the raw data from the data banks.

use std::{error::Error, fmt, mem::size_of};

/// Read a MIDAS file without modifying its contents.
pub mod read;

/// Possible data types stored inside a data bank
#[derive(Debug)]
pub enum Type {
    /// Unsigned byte.
    Byte,
    /// Signed byte.
    I8,
    /// Unsigned byte.
    U8,
    /// Unsigned 16-bits integer.
    U16,
    /// Signed 16-bits integer.
    I16,
    /// Unsigned 32-bits integer.
    U32,
    /// Signed 32-bits integer.
    I32,
    /// Single byte boolean.
    Bool,
    /// 32-bits floating-point number.
    F32,
    /// 64-bits floating-point number.
    F64,
    /// User-defined structure with fixed size in bytes.
    Struct,
}

impl Type {
    /// Returns the size of a [`Type`] in bytes. Note that e.g. [`Type::Struct`] doesn't have a
    /// fixed known size; it is determined by the user.
    pub fn size(&self) -> Option<usize> {
        match *self {
            Type::Byte => Some(size_of::<u8>()),
            Type::I8 => Some(size_of::<i8>()),
            Type::U8 => Some(size_of::<u8>()),
            Type::U16 => Some(size_of::<u16>()),
            Type::I16 => Some(size_of::<i16>()),
            Type::U32 => Some(size_of::<u32>()),
            Type::I32 => Some(size_of::<i32>()),
            Type::Bool => Some(size_of::<bool>()),
            Type::F32 => Some(size_of::<f32>()),
            Type::F64 => Some(size_of::<f64>()),
            Type::Struct => None,
        }
    }
}

/// The error type returned when conversion from unsigned integer to [`Type`] fails.
#[derive(Debug)]
pub struct TryTypeFromUnsignedError;
impl fmt::Display for TryTypeFromUnsignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conversion from unknown value attempted")
    }
}
impl Error for TryTypeFromUnsignedError {}

// Implement conversion from any unsigned integer type
macro_rules! impl_try_type_from {
    ($num_type:ty) => {
        impl TryFrom<$num_type> for Type {
            type Error = TryTypeFromUnsignedError;

            fn try_from(num: $num_type) -> Result<Self, Self::Error> {
                match num {
                    1 => Ok(Type::Byte),
                    2 => Ok(Type::I8),
                    3 => Ok(Type::U8),
                    4 => Ok(Type::U16),
                    5 => Ok(Type::I16),
                    6 => Ok(Type::U32),
                    7 => Ok(Type::I32),
                    8 => Ok(Type::Bool),
                    9 => Ok(Type::F32),
                    10 => Ok(Type::F64),
                    14 => Ok(Type::Struct),
                    _ => Err(TryTypeFromUnsignedError),
                }
            }
        }
    };

    ($first:ty, $($rest:ty),+ $(,)?) => {
        impl_try_type_from!($first);
        impl_try_type_from!($($rest),+);
    };
}
impl_try_type_from!(u16, u32);

#[cfg(test)]
mod tests;
