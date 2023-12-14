//! A Rust library to read binary MIDAS files.
//!
//! Midasio provides utilities to iterate over the MIDAS events in a file, iterate over the data
//! banks in a MIDAS event, and extract the raw data from the data banks.

use std::{error::Error, fmt, mem::size_of};

pub use crate::read::data_bank;
pub use crate::read::event;

//MIDAS File constants
const ODB_ID_LENGTH: usize = 2;
const ODB_MI_LENGTH: usize = 2;
const ODB_RUN_NUMBER_LENGTH: usize = 4;
const ODB_TIME_STAMP_LENGTH: usize = 4;
const ODB_SIZE_LENGTH: usize = 4;
const ODB_HEADER_LENGTH: usize =
    ODB_ID_LENGTH + ODB_MI_LENGTH + ODB_RUN_NUMBER_LENGTH + ODB_TIME_STAMP_LENGTH + ODB_SIZE_LENGTH;
const BOR_ID: u16 = 32768;
const ODB_MI: u16 = 18765;
const EOR_ID: u16 = 32769;

// MIDAS Event constants
// Length in bytes of each field
const EVENT_ID_LENGTH: usize = 2;
const EVENT_TRIGGER_MASK_LENGTH: usize = 2;
const EVENT_SERIAL_NUMBER_LENGTH: usize = 4;
const EVENT_TIME_STAMP_LENGTH: usize = 4;
const EVENT_SIZE_LENGTH: usize = 4;
const EVENT_ALL_BANKS_SIZE_LENGTH: usize = 4;
const EVENT_FLAGS_LENGTH: usize = 4;
const EVENT_HEADER_LENGTH: usize = EVENT_ID_LENGTH
    + EVENT_TRIGGER_MASK_LENGTH
    + EVENT_SERIAL_NUMBER_LENGTH
    + EVENT_TIME_STAMP_LENGTH
    + EVENT_SIZE_LENGTH
    + EVENT_ALL_BANKS_SIZE_LENGTH
    + EVENT_FLAGS_LENGTH;

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
    /// Four bytes boolean.
    Bool,
    /// 32-bits floating-point number.
    F32,
    /// 64-bits floating-point number.
    F64,
    /// 32-bits bitfield.
    Bit32,
    /// Zero-terminated string.
    Str,
    /// User-defined structure with fixed size in bytes.
    Struct,
    /// Signed 64-bits integer.
    I64,
    /// Unsigned 64-bits integer.
    U64,
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
            // If you add a new type here, remember to add it as well in TryFrom unsigned below
            DataType::Byte => Some(size_of::<u8>()),
            DataType::I8 => Some(size_of::<i8>()),
            DataType::U8 => Some(size_of::<u8>()),
            DataType::U16 => Some(size_of::<u16>()),
            DataType::I16 => Some(size_of::<i16>()),
            DataType::U32 => Some(size_of::<u32>()),
            DataType::I32 => Some(size_of::<i32>()),
            DataType::Bool => Some(4), // 4 bytes bool instead of normal 1 byte
            DataType::F32 => Some(size_of::<f32>()),
            DataType::F64 => Some(size_of::<f64>()),
            DataType::Bit32 => Some(4), // 4 bytes = 32 bits
            DataType::Str => None,
            DataType::Struct => None,
            DataType::I64 => Some(size_of::<i64>()),
            DataType::U64 => Some(size_of::<u64>()),
        }
    }
}

/// The error type returned when conversion from unsigned integer to [`DataType`] fails.
#[derive(Clone, Copy, Debug)]
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
                    11 => Ok(DataType::Bit32),
                    12 => Ok(DataType::Str),
                    // 13 missing. Array. What does that mean?
                    14 => Ok(DataType::Struct),
                    // 15 missing. Key. What does that mean?
                    // 16 missing. Link. What does that mean?
                    17 => Ok(DataType::I64),
                    18 => Ok(DataType::U64),
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

// Possible variants of data banks.
#[derive(Clone, Copy, Debug)]
enum BankType {
    B16,
    B32,
    B32A,
}
impl TryFrom<u32> for BankType {
    type Error = TryBankTypeFromUnsignedError;
    // These are the unsigned integer representation of the [`BankType`] in the flags field of the
    // MIDAS events.
    fn try_from(num: u32) -> Result<Self, Self::Error> {
        match num {
            1 => Ok(BankType::B16),
            17 => Ok(BankType::B32),
            49 => Ok(BankType::B32A),
            _ => Err(TryBankTypeFromUnsignedError),
        }
    }
}
// The error type returned when conversion from unsigned integer to [`BankType`] fails.
#[derive(Clone, Copy, Debug)]
struct TryBankTypeFromUnsignedError;
impl fmt::Display for TryBankTypeFromUnsignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conversion from unknown value attempted")
    }
}

#[cfg(test)]
mod tests;
