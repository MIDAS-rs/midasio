use std::mem::size_of;
use std::slice::ChunksExact;
use thiserror::Error;
use winnow::binary::{length_take, u16, u32, Endianness};
use winnow::combinator::{seq, terminated};
use winnow::error::{ContextError, ParseError};
use winnow::stream::AsChar;
use winnow::token::{take, take_while};
use winnow::Parser;

impl DataType {
    /// Returns the size in bytes of an element of type [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::DataType;
    ///
    /// assert_eq!(DataType::U8.size(), Some(1));
    /// assert!(DataType::Struct.size().is_none());
    /// ```
    fn size(&self) -> Option<usize> {
        match self {
            DataType::U8 => Some(size_of::<u8>()),
            DataType::I8 => Some(size_of::<i8>()),
            DataType::U16 => Some(size_of::<u16>()),
            DataType::I16 => Some(size_of::<i16>()),
            DataType::U32 => Some(size_of::<u32>()),
            DataType::I32 => Some(size_of::<i32>()),
            DataType::Bool => Some(4), // 4 bytes bool instead of normal 1 byte
            DataType::F32 => Some(size_of::<f32>()),
            DataType::F64 => Some(size_of::<f64>()),
            DataType::Str => None,
            DataType::Array => None,
            DataType::Struct => None,
            DataType::I64 => Some(size_of::<i64>()),
            DataType::U64 => Some(size_of::<u64>()),
        }
    }
}

/// The error type returned when conversion from unsigned integer to
/// [`DataType`] fails.
#[derive(Debug, Error)]
#[error("unknown conversion from unsigned `{input}` to DataType")]
pub struct TryDataTypeFromUnsignedError {
    // Conversion is possible from every unsigned integer type. So just store
    // as the largest one.
    input: u128,
}

// Conversion from any unsigned integer type
macro_rules! impl_try_type_from {
    ($num_type:ty) => {
        impl TryFrom<$num_type> for DataType {
            type Error = TryDataTypeFromUnsignedError;

            fn try_from(num: $num_type) -> Result<Self, Self::Error> {
                match num {
                    1 => Ok(DataType::U8),
                    2 => Ok(DataType::I8),
                    3 => Ok(DataType::U8),
                    4 => Ok(DataType::U16),
                    5 => Ok(DataType::I16),
                    6 => Ok(DataType::U32),
                    7 => Ok(DataType::I32),
                    8 => Ok(DataType::Bool),
                    9 => Ok(DataType::F32),
                    10 => Ok(DataType::F64),
                    11 => Ok(DataType::U32),
                    12 => Ok(DataType::Str),
                    13 => Ok(DataType::Array),
                    14 => Ok(DataType::Struct),
                    15 => Ok(DataType::Str), // ODB Key
                    16 => Ok(DataType::Str), // ODB Link
                    17 => Ok(DataType::I64),
                    18 => Ok(DataType::U64),
                    _ => Err(TryDataTypeFromUnsignedError {
                        input: u128::from(num),
                    }),
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

pub(crate) fn bank_16_view<'a>(
    endian: Endianness,
) -> impl Parser<&'a [u8], Bank16View<'a>, ContextError> {
    seq! {Bank16View {
        name: take_while(4, AsChar::is_alphanum)
            // SAFETY: All 4 bytes are ASCII alphanumeric.
            .map(|b: &[u8]| unsafe { std::str::from_utf8_unchecked(b) }),
        data_type: u16(endian).try_map(DataType::try_from),
        data_slice: length_take(u16(endian))
            .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0),
    }}
}

pub(crate) fn bank_32_view<'a>(
    endian: Endianness,
) -> impl Parser<&'a [u8], Bank32View<'a>, ContextError> {
    seq! {Bank32View {
        name: take_while(4, AsChar::is_alphanum)
            // SAFETY: All 4 bytes are ASCII alphanumeric.
            .map(|b: &[u8]| unsafe { std::str::from_utf8_unchecked(b) }),
        data_type: u32(endian).try_map(DataType::try_from),
        data_slice: length_take(u32(endian))
            .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0),
    }}
}

pub(crate) fn bank_32a_view<'a>(
    endian: Endianness,
) -> impl Parser<&'a [u8], Bank32AView<'a>, ContextError> {
    seq! {Bank32AView {
        name: take_while(4, AsChar::is_alphanum)
            // SAFETY: All 4 bytes are ASCII alphanumeric.
            .map(|b: &[u8]| unsafe { std::str::from_utf8_unchecked(b) }),
        data_type: u32(endian).try_map(DataType::try_from),
        data_slice: length_take(terminated(u32(endian), take(4usize)))
            .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0),
    }}
}
