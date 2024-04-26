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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! data_type_tests_for {
        ($num_type:ty) => {
            paste::paste! {
                #[test]
                fn [<data_type_try_from_ $num_type>]() -> Result<(), Box<dyn std::error::Error>> {
                    assert_eq!(DataType::try_from([<1 $num_type>])?, DataType::U8);
                    assert_eq!(DataType::try_from([<2 $num_type>])?, DataType::I8);
                    assert_eq!(DataType::try_from([<3 $num_type>])?, DataType::U8);
                    assert_eq!(DataType::try_from([<4 $num_type>])?, DataType::U16);
                    assert_eq!(DataType::try_from([<5 $num_type>])?, DataType::I16);
                    assert_eq!(DataType::try_from([<6 $num_type>])?, DataType::U32);
                    assert_eq!(DataType::try_from([<7 $num_type>])?, DataType::I32);
                    assert_eq!(DataType::try_from([<8 $num_type>])?, DataType::Bool);
                    assert_eq!(DataType::try_from([<9 $num_type>])?, DataType::F32);
                    assert_eq!(DataType::try_from([<10 $num_type>])?, DataType::F64);
                    assert_eq!(DataType::try_from([<11 $num_type>])?, DataType::U32);
                    assert_eq!(DataType::try_from([<12 $num_type>])?, DataType::Str);
                    // TID_ARRAY. Not supported yet.
                    assert!(DataType::try_from([<13 $num_type>]).is_err());
                    assert_eq!(DataType::try_from([<14 $num_type>])?, DataType::Struct);
                    // TID_KEY. Not supported yet.
                    assert!(DataType::try_from([<15 $num_type>]).is_err());
                    // TID_LINK. Not supported yet.
                    assert!(DataType::try_from([<16 $num_type>]).is_err());
                    assert_eq!(DataType::try_from([<17 $num_type>])?, DataType::I64);
                    assert_eq!(DataType::try_from([<18 $num_type>])?, DataType::U64);
                    for i in [<19 $num_type>]..=255 {
                        assert!(DataType::try_from(i).is_err());
                    }
                    Ok(())
                }
            }
        };

        ($first:ty, $($rest:ty),+ $(,)?) => {
            data_type_tests_for!($first);
            data_type_tests_for!($($rest),+);
        };
    }
    data_type_tests_for!(u8, u16, u32, u64, u128);

    #[test]
    fn data_type_size() {
        assert_eq!(DataType::U8.size(), Some(1));
        assert_eq!(DataType::I8.size(), Some(1));
        assert_eq!(DataType::U16.size(), Some(2));
        assert_eq!(DataType::I16.size(), Some(2));
        assert_eq!(DataType::U32.size(), Some(4));
        assert_eq!(DataType::I32.size(), Some(4));
        assert_eq!(DataType::Bool.size(), Some(4));
        assert_eq!(DataType::F32.size(), Some(4));
        assert_eq!(DataType::F64.size(), Some(8));
        assert_eq!(DataType::Str.size(), None);
        assert_eq!(DataType::Array.size(), None);
        assert_eq!(DataType::Struct.size(), None);
        assert_eq!(DataType::I64.size(), Some(8));
        assert_eq!(DataType::U64.size(), Some(8));
    }

    #[test]
    fn bank_16_view_try_from_le_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x04\x00\x02\x00\x34\x12";
        let bank = Bank16View::try_from_le_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U16);
        assert_eq!(bank.data_slice(), &[0x34, 0x12]);

        let bytes = b"NAME\x01\x00\x00\x00";
        let bank = Bank16View::try_from_le_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[]);

        Ok(())
    }

    #[test]
    fn bank_16_view_try_from_be_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x00\x04\x00\x02\x12\x34";
        let bank = Bank16View::try_from_be_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U16);
        assert_eq!(bank.data_slice(), &[0x12, 0x34]);

        let bytes = b"NAME\x00\x01\x00\x00";
        let bank = Bank16View::try_from_be_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[]);

        Ok(())
    }

    #[test]
    fn bank_16_view_invalid_name() {
        for name in [".AME", "N.ME", "NA.E", "NAM."] {
            let bytes = [name.as_bytes(), b"\x04\x00\x02\x00\x34\x12"].concat();
            let result = Bank16View::try_from_le_bytes(&bytes);
            assert!(result.is_err());

            let bytes = [name.as_bytes(), b"\x00\x04\x00\x02\x12\x34"].concat();
            let result = Bank16View::try_from_be_bytes(&bytes);
            assert!(result.is_err());
        }
    }

    #[test]
    fn bank_16_view_invalid_data_type() {
        let bytes = b"NAME\xFF\xFF\x02\x00\x34\x12";
        let result = Bank16View::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\xFF\xFF\x00\x02\x12\x34";
        let result = Bank16View::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_16_view_invalid_data_slice_non_integer_number_of_elements() {
        let bytes = b"NAME\x04\x00\x03\x00\x56\x34\x12";
        let result = Bank16View::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x04\x00\x03\x12\x34\x56";
        let result = Bank16View::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_16_view_invalid_data_slice_length() {
        let bytes = b"NAME\x01\x00\x02\x00\x12";
        let result = Bank16View::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x01\x00\x02\x00\x56\x34\x12";
        let result = Bank16View::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x01\x00\x02\x34";
        let result = Bank16View::try_from_be_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x01\x00\x02\x12\x34\x56";
        let result = Bank16View::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_32_view_try_from_le_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x04\x00\x00\x00\x02\x00\x00\x00\x34\x12";
        let bank = Bank32View::try_from_le_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U16);
        assert_eq!(bank.data_slice(), &[0x34, 0x12]);

        let bytes = b"NAME\x01\x00\x00\x00\x00\x00\x00\x00";
        let bank = Bank32View::try_from_le_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[]);

        Ok(())
    }

    #[test]
    fn bank_32_view_try_from_be_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x00\x00\x00\x04\x00\x00\x00\x02\x12\x34";
        let bank = Bank32View::try_from_be_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U16);
        assert_eq!(bank.data_slice(), &[0x12, 0x34]);

        let bytes = b"NAME\x00\x00\x00\x01\x00\x00\x00\x00";
        let bank = Bank32View::try_from_be_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[]);

        Ok(())
    }

    #[test]
    fn bank_32_view_invalid_name() {
        for name in [".AME", "N.ME", "NA.E", "NAM."] {
            let bytes = [name.as_bytes(), b"\x01\x00\x00\x00\x01\x00\x00\x00\x12"].concat();
            let result = Bank32View::try_from_le_bytes(&bytes);
            assert!(result.is_err());

            let bytes = [name.as_bytes(), b"\x00\x00\x00\x01\x00\x00\x00\x01\x12"].concat();
            let result = Bank32View::try_from_be_bytes(&bytes);
            assert!(result.is_err());
        }
    }

    #[test]
    fn bank_32_view_invalid_data_type() {
        let bytes = b"NAME\xFF\xFF\xFF\xFF\x01\x00\x00\x00\x12";
        let result = Bank32View::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\xFF\xFF\xFF\xFF\x00\x00\x00\x01\x12";
        let result = Bank32View::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_32_view_invalid_data_slice_non_integer_number_of_elements() {
        let bytes = b"NAME\x04\x00\x00\x00\x03\x00\x00\x00\x56\x34\x12";
        let result = Bank32View::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x00\x00\x04\x00\x00\x00\x03\x12\x34\x56";
        let result = Bank32View::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_32_view_invalid_data_slice_length() {
        let bytes = b"NAME\x01\x00\x00\x00\x02\x00\x00\x00\x12";
        let result = Bank32View::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x01\x00\x00\x00\x02\x00\x00\x00\x56\x34\x12";
        let result = Bank32View::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x00\x00\x01\x00\x00\x00\x02\x12";
        let result = Bank32View::try_from_be_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x00\x00\x01\x00\x00\x00\x02\x12\x34\x56";
        let result = Bank32View::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_32a_view_try_from_le_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x04\x00\x00\x00\x02\x00\x00\x00\xFF\xFF\xFF\xFF\x34\x12";
        let bank = Bank32AView::try_from_le_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U16);
        assert_eq!(bank.data_slice(), &[0x34, 0x12]);

        let bytes = b"NAME\x01\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF\xFF";
        let bank = Bank32AView::try_from_le_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[]);

        Ok(())
    }

    #[test]
    fn bank_32a_view_try_from_be_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x00\x00\x00\x04\x00\x00\x00\x02\xFF\xFF\xFF\xFF\x12\x34";
        let bank = Bank32AView::try_from_be_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U16);
        assert_eq!(bank.data_slice(), &[0x12, 0x34]);

        let bytes = b"NAME\x00\x00\x00\x01\x00\x00\x00\x00\xFF\xFF\xFF\xFF";
        let bank = Bank32AView::try_from_be_bytes(bytes)?;
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[]);

        Ok(())
    }

    #[test]
    fn bank_32a_view_invalid_name() {
        for name in [".AME", "N.ME", "NA.E", "NAM."] {
            let bytes = [
                name.as_bytes(),
                b"\x01\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x12",
            ]
            .concat();
            let result = Bank32AView::try_from_le_bytes(&bytes);
            assert!(result.is_err());

            let bytes = [
                name.as_bytes(),
                b"\x00\x00\x00\x01\x00\x00\x00\x01\x00\x00\x00\x00\x12",
            ]
            .concat();
            let result = Bank32AView::try_from_be_bytes(&bytes);
            assert!(result.is_err());
        }
    }

    #[test]
    fn bank_32a_view_invalid_data_type() {
        let bytes = b"NAME\xFF\xFF\xFF\xFF\x01\x00\x00\x00\x00\x00\x00\x00\x12";
        let result = Bank32AView::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\xFF\xFF\xFF\xFF\x00\x00\x00\x01\x00\x00\x00\x00\x12";
        let result = Bank32AView::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_32a_view_invalid_data_slice_non_integer_number_of_elements() {
        let bytes = b"NAME\x04\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\x56\x34\x12";
        let result = Bank32AView::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x00\x00\x04\x00\x00\x00\x03\x00\x00\x00\x00\x12\x34\x56";
        let result = Bank32AView::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_32a_view_invalid_data_slice_length() {
        let bytes = b"NAME\x01\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x12";
        let result = Bank32AView::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x01\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x56\x34\x12";
        let result = Bank32AView::try_from_le_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x00\x00\x01\x00\x00\x00\x02\x00\x00\x00\x00\x12";
        let result = Bank32AView::try_from_be_bytes(bytes);
        assert!(result.is_err());

        let bytes = b"NAME\x00\x00\x00\x01\x00\x00\x00\x02\x00\x00\x00\x00\x12\x34\x56";
        let result = Bank32AView::try_from_be_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bank_view_from_bank_16_view() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x01\x00\x01\x00\xFF";
        let bank: BankView = Bank16View::try_from_le_bytes(bytes)?.into();
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[0xFF]);

        Ok(())
    }

    #[test]
    fn bank_view_from_bank_32_view() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x01\x00\x00\x00\x01\x00\x00\x00\xFF";
        let bank: BankView = Bank32View::try_from_le_bytes(bytes)?.into();
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[0xFF]);

        Ok(())
    }

    #[test]
    fn bank_view_from_bank_32a_view() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x01\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\xFF";
        let bank: BankView = Bank32AView::try_from_le_bytes(bytes)?.into();
        assert_eq!(bank.name(), "NAME");
        assert_eq!(bank.data_type(), DataType::U8);
        assert_eq!(bank.data_slice(), &[0xFF]);

        Ok(())
    }

    #[test]
    fn bank_view_required_padding() -> Result<(), Box<dyn std::error::Error>> {
        let mut bytes = b"NAME\x01\x00\x00\x00".to_vec();
        let bank: BankView = Bank16View::try_from_le_bytes(&bytes)?.into();
        assert_eq!(bank.required_padding(), 0);

        for n in 1..=8 {
            bytes[6] += 1;
            bytes.push(0xFF);
            let bank: BankView = Bank16View::try_from_le_bytes(&bytes)?.into();

            assert_eq!(bank.required_padding(), 8 - n);
        }

        Ok(())
    }

    #[test]
    fn bank_view_into_iter() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"NAME\x04\x00\x04\x00\x12\x34\x56\x78";
        let bank: BankView = Bank16View::try_from_le_bytes(bytes)?.into();
        let mut iter = bank.into_iter();

        assert_eq!(iter.next(), Some(&[0x12, 0x34][..]));
        assert_eq!(iter.next(), Some(&[0x56, 0x78][..]));
        assert_eq!(iter.next(), None);
        Ok(())
    }
}
