//! A Rust library to read binary MIDAS files.
//!
//! Midasio provides utilities to iterate over the MIDAS events in a file, iterate over the data
//! banks in a MIDAS event, and extract the raw data from the data banks.

use std::{error::Error, fmt, mem::size_of};

// If the size of the data inside a bank is not a multiple of BANK_PADDING, the subsequent bytes up
// until the next multiple of 8 are filled with random values.
const BANK_PADDING: usize = 8;

/// Read a MIDAS file without modifying its contents.
pub mod read;

/// Possible data types stored inside a data bank
#[derive(Clone, Copy, Debug)]
pub enum DataType {
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

impl DataType {
    /// Returns the size of a [`DataType`] in bytes. Note that e.g. [`DataType::Struct`] doesn't have a
    /// fixed known size; it is determined by the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::DataType;
    ///
    /// assert_eq!(DataType::Byte.size().unwrap(), 1);
    /// assert!(DataType::Struct.size().is_none());
    /// ```
    pub fn size(&self) -> Option<usize> {
        match *self {
            DataType::Byte => Some(size_of::<u8>()),
            DataType::I8 => Some(size_of::<i8>()),
            DataType::U8 => Some(size_of::<u8>()),
            DataType::U16 => Some(size_of::<u16>()),
            DataType::I16 => Some(size_of::<i16>()),
            DataType::U32 => Some(size_of::<u32>()),
            DataType::I32 => Some(size_of::<i32>()),
            DataType::Bool => Some(size_of::<bool>()),
            DataType::F32 => Some(size_of::<f32>()),
            DataType::F64 => Some(size_of::<f64>()),
            DataType::Struct => None,
        }
    }
}

/// The error type returned when conversion from unsigned integer to [`DataType`] fails.
#[derive(Debug)]
pub struct TryDataTypeFromUnsignedError;
impl fmt::Display for TryDataTypeFromUnsignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conversion from unknown value attempted")
    }
}
impl Error for TryDataTypeFromUnsignedError {}

// Implement conversion from any unsigned integer type
macro_rules! impl_try_type_from {
    ($num_type:ty) => {
        impl TryFrom<$num_type> for DataType {
            type Error = TryDataTypeFromUnsignedError;

            fn try_from(num: $num_type) -> Result<Self, Self::Error> {
                match num {
                    1 => Ok(DataType::Byte),
                    2 => Ok(DataType::I8),
                    3 => Ok(DataType::U8),
                    4 => Ok(DataType::U16),
                    5 => Ok(DataType::I16),
                    6 => Ok(DataType::U32),
                    7 => Ok(DataType::I32),
                    8 => Ok(DataType::Bool),
                    9 => Ok(DataType::F32),
                    10 => Ok(DataType::F64),
                    14 => Ok(DataType::Struct),
                    _ => Err(TryDataTypeFromUnsignedError),
                }
            }
        }
    };

    ($first:ty, $($rest:ty),+ $(,)?) => {
        impl_try_type_from!($first);
        impl_try_type_from!($($rest),+);
    };
}
impl_try_type_from!(u8, u16, u32, u64, u128);

#[derive(Clone, Copy, Debug)]
enum Endianness {
    LittleEndian,
    BigEndian,
}

#[cfg(test)]
mod tests;
