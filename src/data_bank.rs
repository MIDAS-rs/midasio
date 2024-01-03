use std::mem::size_of;
use std::slice::ChunksExact;
use thiserror::Error;
use winnow::binary::{length_take, u16, u32, Endianness};
use winnow::combinator::{seq, terminated};
use winnow::error::{ContextError, ParseError};
use winnow::stream::AsChar;
use winnow::token::{take, take_while};
use winnow::Parser;

/// Possible data types stored inside a data bank
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    /// Unsigned byte.
    U8,
    /// Signed byte.
    I8,
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
    /// Returns the size of a [`DataType`] in bytes. Note that e.g.
    /// [`DataType::Struct`] doesn't have a fixed known size.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::DataType;
    ///
    /// assert_eq!(DataType::U8.size(), Some(1));
    /// assert!(DataType::Struct.size().is_none());
    /// ```
    pub fn size(&self) -> Option<usize> {
        match self {
            // If you add a new type here, remember to add it as well in TryFrom
            // unsigned below
            DataType::U8 => Some(size_of::<u8>()),
            DataType::I8 => Some(size_of::<i8>()),
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
                    11 => Ok(DataType::Bit32),
                    12 => Ok(DataType::Str),
                    // 13 missing. Array. What does that mean?
                    14 => Ok(DataType::Struct),
                    // 15 missing. Key. What does that mean?
                    // 16 missing. Link. What does that mean?
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

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to a
/// [`BankView`] fails.
#[derive(Debug, Error)]
// I am still experimenting with the error type. This allows me to hide the
// implementation details of the error type without breaking the public API.
#[error(transparent)]
pub struct TryBankViewFromBytesError(#[from] InnerBankParseError);

#[derive(Debug)]
struct InnerBankParseError {
    offset: usize,
    inner: ContextError,
}

impl std::fmt::Display for InnerBankParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsing stopped at byte offset `{}`", self.offset)?;
        if self.inner.context().next().is_some() {
            write!(f, " ({})", self.inner)?;
        }
        Ok(())
    }
}

impl std::error::Error for InnerBankParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner
            .cause()
            .map(|v| v as &(dyn std::error::Error + 'static))
    }
}

#[doc(hidden)]
impl<I> From<ParseError<I, ContextError>> for TryBankViewFromBytesError {
    fn from(e: ParseError<I, ContextError>) -> Self {
        Self(InnerBankParseError {
            offset: e.offset(),
            inner: e.into_inner(),
        })
    }
}

/// An immutable view to a 16-bit data bank.
///
/// A 16-bit data bank is defined as an 8 bytes header followed by its raw data.
/// The binary representation of a [`Bank16View`] is:
///
/// <center>
///
/// |Byte Offset|Size (in bytes)|Description|
/// |:-:|:-:|:-:|
/// |0|4|Bank name (ASCII alphanumeric)|
/// |4|2|Data type (see [`DataType`])|
/// |6|2|Data size (`n`)|
/// |8|`n`|Raw data byte slice|
///
/// </center>
///
/// # Examples
///
/// ```
/// use midasio::data_bank::{DataType, Bank16View};
///
/// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
/// let bank = Bank16View::try_from_le_bytes(bytes)?;
///
/// assert_eq!(bank.name(), "BANK");
/// assert_eq!(bank.data_type(), DataType::U8);
/// assert_eq!(bank.data_slice(), &[0xFF, 0xFF, 0xFF]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Bank16View<'a> {
    name: &'a str,
    data_type: DataType,
    data_slice: &'a [u8],
}

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

impl<'a> Bank16View<'a> {
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in little endian.
    pub fn try_from_le_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_16_view(Endianness::Little).parse(bytes)?)
    }
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in big endian.
    pub fn try_from_be_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_16_view(Endianness::Big).parse(bytes)?)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII
    /// alphanumeric characters.
    pub fn name(&self) -> &'a str {
        self.name
    }
    /// Return the data type of the bank. For a complete list of data types see
    /// [`DataType`].
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    /// Return the raw data of the data. This does not include the header nor
    /// any padding bytes.
    pub fn data_slice(&self) -> &'a [u8] {
        self.data_slice
    }
}

/// An immutable view to a 32-bit data bank.
///
/// A 32-bit data bank is defined as a 12 bytes header followed by its raw data.
/// The binary representation of a [`Bank32View`] is:
///
/// <center>
///
/// |Byte Offset|Size (in bytes)|Description|
/// |:-:|:-:|:-:|
/// |0|4|Bank name (ASCII alphanumeric)|
/// |4|4|Data type (see [`DataType`])|
/// |8|4|Data size (`n`)|
/// |12|`n`|Raw data byte slice|
///
/// </center>
///
/// # Examples
///
/// ```
/// use midasio::data_bank::{DataType, Bank32View};
///
/// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\xFF\xFF\xFF";
/// let bank = Bank32View::try_from_le_bytes(bytes)?;
///
/// assert_eq!(bank.name(), "BANK");
/// assert_eq!(bank.data_type(), DataType::U8);
/// assert_eq!(bank.data_slice(), &[0xFF, 0xFF, 0xFF]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Bank32View<'a> {
    name: &'a str,
    data_type: DataType,
    data_slice: &'a [u8],
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

impl<'a> Bank32View<'a> {
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in little endian.
    pub fn try_from_le_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_32_view(Endianness::Little).parse(bytes)?)
    }
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in big endian.
    pub fn try_from_be_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_32_view(Endianness::Big).parse(bytes)?)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII
    /// alphanumeric characters.
    pub fn name(&self) -> &'a str {
        self.name
    }
    /// Return the data type of the bank. For a complete list of data types see
    /// [`DataType`].
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    /// Return the raw data of the bank. This does not include the header nor
    /// any padding bytes.
    pub fn data_slice(&self) -> &'a [u8] {
        self.data_slice
    }
}

/// An immutable view to a 32-bit bank 64-bit aligned.
///
/// A 32-bit data bank is defined as a 16 bytes header followed by its raw data.
/// The binary representation of a [`Bank32AView`] is:
///
/// <center>
///
/// |Byte Offset|Size (in bytes)|Description|
/// |:-:|:-:|:-:|
/// |0|4|Bank name (ASCII alphanumeric)|
/// |4|4|Data type (see [`DataType`])|
/// |8|4|Data size (`n`)|
/// |12|4|Reserved|
/// |16|`n`|Raw data byte slice|
///
/// </center>
///
/// # Examples
///
/// ```
/// use midasio::data_bank::{DataType, Bank32AView};
///
/// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\x01\x23\x45\x67\xFF\xFF\xFF";
/// let bank = Bank32AView::try_from_le_bytes(bytes)?;
///
/// assert_eq!(bank.name(), "BANK");
/// assert_eq!(bank.data_type(), DataType::U8);
/// assert_eq!(bank.data_slice(), &[0xFF, 0xFF, 0xFF]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Bank32AView<'a> {
    name: &'a str,
    data_type: DataType,
    data_slice: &'a [u8],
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

impl<'a> Bank32AView<'a> {
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in little endian.
    pub fn try_from_le_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_32a_view(Endianness::Little).parse(bytes)?)
    }
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in big endian.
    pub fn try_from_be_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_32a_view(Endianness::Big).parse(bytes)?)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII
    /// alphanumeric characters.
    pub fn name(&self) -> &'a str {
        self.name
    }
    /// Return the data type of the bank. For a complete list of data types see
    /// [`DataType`].
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    /// Return the raw data of the bank. This does not include the header nor
    /// any padding bytes.
    pub fn data_slice(&self) -> &'a [u8] {
        self.data_slice
    }
}

/// An immutable view to a data bank.
///
/// This enum can contain either a [`Bank16View`], a [`Bank32View`], or a
/// [`Bank32AView`]. See their respective documentation for more details.
#[derive(Clone, Copy, Debug)]
pub enum BankView<'a> {
    /// A 16-bit bank.
    B16(Bank16View<'a>),
    /// A 32-bit bank.
    B32(Bank32View<'a>),
    /// A 32-bit bank 64-bit aligned.
    B32A(Bank32AView<'a>),
}

impl<'a> From<Bank16View<'a>> for BankView<'a> {
    fn from(bank: Bank16View<'a>) -> Self {
        Self::B16(bank)
    }
}
impl<'a> From<Bank32View<'a>> for BankView<'a> {
    fn from(bank: Bank32View<'a>) -> Self {
        Self::B32(bank)
    }
}
impl<'a> From<Bank32AView<'a>> for BankView<'a> {
    fn from(bank: Bank32AView<'a>) -> Self {
        Self::B32A(bank)
    }
}

impl<'a> BankView<'a> {
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII
    /// alphanumeric characters.
    pub fn name(&self) -> &'a str {
        match self {
            BankView::B16(bank) => bank.name(),
            BankView::B32(bank) => bank.name(),
            BankView::B32A(bank) => bank.name(),
        }
    }
    /// Return the data type of the bank. For a complete list of data types see
    /// [`DataType`].
    pub fn data_type(&self) -> DataType {
        match self {
            BankView::B16(bank) => bank.data_type(),
            BankView::B32(bank) => bank.data_type(),
            BankView::B32A(bank) => bank.data_type(),
        }
    }
    /// Return the raw data of the bank. This does not include the header nor
    /// any padding bytes.
    pub fn data_slice(&self) -> &'a [u8] {
        match self {
            BankView::B16(bank) => bank.data_slice(),
            BankView::B32(bank) => bank.data_slice(),
            BankView::B32A(bank) => bank.data_slice(),
        }
    }
    /// In a MIDAS file, data banks are padded to a multiple of 8 bytes. This
    /// method returns the number of padding bytes that would be required to
    /// make the data area of this bank a multiple of 8 bytes.
    pub fn required_padding(&self) -> usize {
        const BANK_DATA_ALIGNMENT: usize = 8;
        let remainder = self.data_slice().len() % BANK_DATA_ALIGNMENT;
        if remainder == 0 {
            0
        } else {
            BANK_DATA_ALIGNMENT - remainder
        }
    }
}

impl<'a> IntoIterator for BankView<'a> {
    /// The type of elements being iterated over. The length of each slice is
    /// fixed to [`DataType::size`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;
    /// Returns an iterator over the [`BankView::data_slice`] in chunks of
    /// [`DataType::size`]. Iterate over individual bytes if the data type size
    /// is [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::{Bank16View, BankView};
    ///
    /// let bytes = b"BANK\x04\x00\x04\x00\xFF\xFF\xFF\xFF";
    /// let bank = BankView::B16(Bank16View::try_from_le_bytes(bytes)?);
    ///
    /// assert_eq!(bank.into_iter().count(), 2);
    /// for u16_slice in bank {
    ///     assert_eq!(u16_slice, &[0xFF, 0xFF]);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct doesn't have a fixed size,
        //iterate over individual bytes.
        let item_size = self.data_type().size().unwrap_or(1);
        self.data_slice().chunks_exact(item_size)
    }
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
                    assert_eq!(DataType::try_from([<11 $num_type>])?, DataType::Bit32);
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
        assert_eq!(DataType::Bit32.size(), Some(4));
        assert_eq!(DataType::Str.size(), None);
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
