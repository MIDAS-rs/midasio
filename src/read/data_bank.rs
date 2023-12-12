use crate::DataType;
use std::slice::ChunksExact;
use thiserror::Error;
use winnow::binary::{length_take, u16, u32, Endianness};
use winnow::combinator::terminated;
use winnow::error::{ContextError, ParseError, StrContext};
use winnow::stream::AsChar;
use winnow::token::take_while;
use winnow::Parser;

// The data area of a bank must have a length that is a multiple of 8 bytes. If
// the data slice is not a multiple of 8 bytes, then the data bank is padded
// with random bytes to make it a multiple of 8 bytes.
const BANK_DATA_ALIGNMENT: usize = 8;

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
        write!(f, "parsing failed at byte offset `{}`", self.offset)?;
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
#[derive(Clone, Copy, Debug)]
pub struct Bank16View<'a> {
    name: &'a str,
    data_type: DataType,
    data_slice: &'a [u8],
}

fn bank_16_view<'a>(endian: Endianness) -> impl Parser<&'a [u8], Bank16View<'a>, ContextError> {
    move |input: &mut &'a [u8]| {
        let (name, data_type) = (
            take_while(4, AsChar::is_alphanum)
                // SAFETY: All 4 bytes are ASCII alphanumeric.
                .map(|b: &[u8]| unsafe { std::str::from_utf8_unchecked(b) })
                .context(StrContext::Label("bank name")),
            u16(endian)
                .try_map(DataType::try_from)
                .context(StrContext::Label("data type")),
        )
            .parse_next(input)?;
        let data_slice = length_take(u16(endian))
            .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0)
            .context(StrContext::Label("data slice"))
            .parse_next(input)?;

        Ok(Bank16View {
            name,
            data_type,
            data_slice,
        })
    }
}

impl<'a> Bank16View<'a> {
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank16View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = Bank16View::try_from_le_bytes(bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_le_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_16_view(Endianness::Little).parse(bytes)?)
    }
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank16View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x00\x01\x00\x03\xFF\xFF\xFF";
    /// let bank = Bank16View::try_from_be_bytes(bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_be_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_16_view(Endianness::Big).parse(bytes)?)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII
    /// alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank16View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = Bank16View::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.name(), "BANK");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &'a str {
        self.name
    }
    /// Return the data type of the data bank. For a complete list of data types
    /// see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::{DataType, data_bank::Bank16View};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = Bank16View::try_from_le_bytes(bytes)?;
    ///
    /// assert!(matches!(bank.data_type(), DataType::Byte));
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    /// Return the raw data of the data bank. This does not include the header
    /// nor any padding bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank16View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = Bank16View::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.data_slice(), &[0xFF, 0xFF, 0xFF]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_slice(&self) -> &'a [u8] {
        self.data_slice
    }
    /// In a MIDAS file, data banks are padded to a multiple of 8 bytes. This
    /// method returns the number of padding bytes that would be required to
    /// make the data area of this data bank a multiple of 8 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank16View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = Bank16View::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.required_padding(), 5);
    /// # Ok(())
    /// # }
    /// ```
    pub fn required_padding(&self) -> usize {
        let remainder = self.data_slice.len() % BANK_DATA_ALIGNMENT;
        if remainder == 0 {
            0
        } else {
            BANK_DATA_ALIGNMENT - remainder
        }
    }
}

impl<'a> IntoIterator for Bank16View<'a> {
    /// The type of elements being iterated over. The length of each slice is
    /// fixed to [`DataType::size`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;
    /// Returns an iterator over the [`Bank16View::data_slice`] in chunks of
    /// [`DataType::size`]. Iterate over individual bytes if the data type size
    /// is [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank16View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x04\x00\x04\x00\xFF\xFF\xFF\xFF";
    /// let bank = Bank16View::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.into_iter().count(), 2);
    /// for u16_slice in bank {
    ///     assert_eq!(u16_slice, &[0xFF, 0xFF]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct doesn't have a fixed size,
        //iterate over individual bytes.
        let item_size = self.data_type.size().unwrap_or(1);
        self.data_slice.chunks_exact(item_size)
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
#[derive(Clone, Copy, Debug)]
pub struct Bank32View<'a> {
    name: &'a str,
    data_type: DataType,
    data_slice: &'a [u8],
}

fn bank_32_view<'a>(endian: Endianness) -> impl Parser<&'a [u8], Bank32View<'a>, ContextError> {
    move |input: &mut &'a [u8]| {
        let (name, data_type) = (
            take_while(4, AsChar::is_alphanum)
                // SAFETY: All 4 bytes are ASCII alphanumeric.
                .map(|b: &[u8]| unsafe { std::str::from_utf8_unchecked(b) })
                .context(StrContext::Label("bank name")),
            u32(endian)
                .try_map(DataType::try_from)
                .context(StrContext::Label("data type")),
        )
            .parse_next(input)?;
        let data_slice = length_take(u32(endian))
            .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0)
            .context(StrContext::Label("data slice"))
            .parse_next(input)?;

        Ok(Bank32View {
            name,
            data_type,
            data_slice,
        })
    }
}

impl<'a> Bank32View<'a> {
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32View::try_from_le_bytes(bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_le_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_32_view(Endianness::Little).parse(bytes)?)
    }
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x00\x00\x00\x01\x00\x00\x00\x03\xFF\xFF\xFF";
    /// let bank = Bank32View::try_from_be_bytes(bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_be_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_32_view(Endianness::Big).parse(bytes)?)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII
    /// alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32View::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.name(), "BANK");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &'a str {
        self.name
    }
    /// Return the data type of the data bank. For a complete list of data types
    /// see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::{DataType, data_bank::Bank32View};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32View::try_from_le_bytes(bytes)?;
    ///
    /// assert!(matches!(bank.data_type(), DataType::Byte));
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    /// Return the raw data of the data bank. This does not include the header
    /// nor any padding bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32View::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.data_slice(), &[0xFF, 0xFF, 0xFF]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_slice(&self) -> &'a [u8] {
        self.data_slice
    }
    /// In a MIDAS file, data banks are padded to a multiple of 8 bytes. This
    /// method returns the number of padding bytes that would be required to
    /// make the data area of this data bank a multiple of 8 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32View::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.required_padding(), 5);
    /// # Ok(())
    /// # }
    /// ```
    pub fn required_padding(&self) -> usize {
        let remainder = self.data_slice.len() % BANK_DATA_ALIGNMENT;
        if remainder == 0 {
            0
        } else {
            BANK_DATA_ALIGNMENT - remainder
        }
    }
}

impl<'a> IntoIterator for Bank32View<'a> {
    /// The type of elements being iterated over. The length of each slice is
    /// fixed to [`DataType::size`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;
    /// Returns an iterator over the [`Bank32View::data_slice`] in chunks of
    /// [`DataType::size`]. Iterate over individual bytes if the data type size
    /// is [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32View;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x04\x00\x00\x00\x04\x00\x00\x00\xFF\xFF\xFF\xFF";
    /// let bank = Bank32View::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.into_iter().count(), 2);
    /// for u16_slice in bank {
    ///     assert_eq!(u16_slice, &[0xFF, 0xFF]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct doesn't have a fixed size,
        //iterate over individual bytes.
        let item_size = self.data_type.size().unwrap_or(1);
        self.data_slice.chunks_exact(item_size)
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
#[derive(Clone, Copy, Debug)]
pub struct Bank32AView<'a> {
    name: &'a str,
    data_type: DataType,
    data_slice: &'a [u8],
}

fn bank_32a_view<'a>(endian: Endianness) -> impl Parser<&'a [u8], Bank32AView<'a>, ContextError> {
    move |input: &mut &'a [u8]| {
        let (name, data_type) = (
            take_while(4, AsChar::is_alphanum)
                // SAFETY: All 4 bytes are ASCII alphanumeric.
                .map(|b: &[u8]| unsafe { std::str::from_utf8_unchecked(b) })
                .context(StrContext::Label("bank name")),
            u32(endian)
                .try_map(DataType::try_from)
                .context(StrContext::Label("data type")),
        )
            .parse_next(input)?;
        let data_slice = length_take(terminated(
            u32(endian),
            b"\x00\x00\x00\x00".context(StrContext::Label("bank header reserved bytes")),
        ))
        .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0)
        .context(StrContext::Label("data slice"))
        .parse_next(input)?;

        Ok(Bank32AView {
            name,
            data_type,
            data_slice,
        })
    }
}

impl<'a> Bank32AView<'a> {
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32AView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32AView::try_from_le_bytes(bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_le_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_32a_view(Endianness::Little).parse(bytes)?)
    }
    /// Create a native view to the underlying data bank from its representation
    /// as a byte slice in big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32AView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x00\x00\x00\x01\x00\x00\x00\x03\x00\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32AView::try_from_be_bytes(bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_be_bytes(bytes: &'a [u8]) -> Result<Self, TryBankViewFromBytesError> {
        Ok(bank_32a_view(Endianness::Big).parse(bytes)?)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII
    /// alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32AView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32AView::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.name(), "BANK");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &'a str {
        self.name
    }
    /// Return the data type of the data bank. For a complete list of data types
    /// see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::{DataType, data_bank::Bank32AView};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32AView::try_from_le_bytes(bytes)?;
    ///
    /// assert!(matches!(bank.data_type(), DataType::Byte));
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    /// Return the raw data of the data bank. This does not include the header
    /// nor any padding bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32AView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32AView::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.data_slice(), &[0xFF, 0xFF, 0xFF]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_slice(&self) -> &'a [u8] {
        self.data_slice
    }
    /// In a MIDAS file, data banks are padded to a multiple of 8 bytes. This
    /// method returns the number of padding bytes that would be required to
    /// make the data area of this data bank a multiple of 8 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32AView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = Bank32AView::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.required_padding(), 5);
    /// # Ok(())
    /// # }
    /// ```
    pub fn required_padding(&self) -> usize {
        let remainder = self.data_slice.len() % BANK_DATA_ALIGNMENT;
        if remainder == 0 {
            0
        } else {
            BANK_DATA_ALIGNMENT - remainder
        }
    }
}

impl<'a> IntoIterator for Bank32AView<'a> {
    /// The type of elements being iterated over. The length of each slice is
    /// fixed to [`DataType::size`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;
    /// Returns an iterator over the [`Bank32AView::data_slice`] in chunks of
    /// [`DataType::size`]. Iterate over individual bytes if the data type size
    /// is [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::Bank32AView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x04\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF\xFF";
    /// let bank = Bank32AView::try_from_le_bytes(bytes)?;
    ///
    /// assert_eq!(bank.into_iter().count(), 2);
    /// for u16_slice in bank {
    ///     assert_eq!(u16_slice, &[0xFF, 0xFF]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct doesn't have a fixed size,
        //iterate over individual bytes.
        let item_size = self.data_type.size().unwrap_or(1);
        self.data_slice.chunks_exact(item_size)
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

impl<'a> BankView<'a> {
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII
    /// alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::{Bank16View, BankView};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = BankView::B16(Bank16View::try_from_le_bytes(bytes)?);
    ///
    /// assert_eq!(bank.name(), "BANK");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &'a str {
        match self {
            BankView::B16(bank) => bank.name(),
            BankView::B32(bank) => bank.name(),
            BankView::B32A(bank) => bank.name(),
        }
    }
    /// Return the data type of the data bank. For a complete list of data types
    /// see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::{DataType, data_bank::{Bank16View, BankView}};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = BankView::B16(Bank16View::try_from_le_bytes(bytes)?);
    ///
    /// assert!(matches!(bank.data_type(), DataType::Byte));
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_type(&self) -> DataType {
        match self {
            BankView::B16(bank) => bank.data_type(),
            BankView::B32(bank) => bank.data_type(),
            BankView::B32A(bank) => bank.data_type(),
        }
    }
    /// Return the raw data of the data bank. This does not include the header
    /// nor any padding bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::{Bank16View, BankView};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = BankView::B16(Bank16View::try_from_le_bytes(bytes)?);
    ///
    /// assert_eq!(bank.data_slice(), &[0xFF, 0xFF, 0xFF]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_slice(&self) -> &'a [u8] {
        match self {
            BankView::B16(bank) => bank.data_slice(),
            BankView::B32(bank) => bank.data_slice(),
            BankView::B32A(bank) => bank.data_slice(),
        }
    }
    /// In a MIDAS file, data banks are padded to a multiple of 8 bytes. This
    /// method returns the number of padding bytes that would be required to
    /// make the data area of this data bank a multiple of 8 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::{Bank16View, BankView};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = BankView::B16(Bank16View::try_from_le_bytes(bytes)?);
    ///
    /// assert_eq!(bank.required_padding(), 5);
    /// # Ok(())
    /// # }
    /// ```
    pub fn required_padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_DATA_ALIGNMENT;
        if remainder == 0 {
            0
        } else {
            BANK_DATA_ALIGNMENT - remainder
        }
    }
    /// Returns [`true`] if this data bank is a [`Bank16View`], and [`false`]
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::{Bank16View, BankView};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x03\x00\xFF\xFF\xFF";
    /// let bank = BankView::B16(Bank16View::try_from_le_bytes(bytes)?);
    ///
    /// assert!(bank.is_b16());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_b16(&self) -> bool {
        matches!(self, BankView::B16(_))
    }
    /// Returns [`true`] if this data bank is a [`Bank32View`], and [`false`]
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::{Bank32View, BankView};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = BankView::B32(Bank32View::try_from_le_bytes(bytes)?);
    ///
    /// assert!(bank.is_b32());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_b32(&self) -> bool {
        matches!(self, BankView::B32(_))
    }
    /// Returns [`true`] if this data bank is a [`Bank32AView`], and [`false`]
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::data_bank::{Bank32AView, BankView};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x01\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF";
    /// let bank = BankView::B32A(Bank32AView::try_from_le_bytes(bytes)?);
    ///
    /// assert!(bank.is_b32a());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_b32a(&self) -> bool {
        matches!(self, BankView::B32A(_))
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes = b"BANK\x04\x00\x04\x00\xFF\xFF\xFF\xFF";
    /// let bank = BankView::B16(Bank16View::try_from_le_bytes(bytes)?);
    ///
    /// assert_eq!(bank.into_iter().count(), 2);
    /// for u16_slice in bank {
    ///     assert_eq!(u16_slice, &[0xFF, 0xFF]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct doesn't have a fixed size,
        //iterate over individual bytes.
        let item_size = self.data_type().size().unwrap_or(1);
        self.data_slice().chunks_exact(item_size)
    }
}

#[cfg(test)]
mod tests;

//-------- To be deleted after the winnow re-write is complete. ---------------
pub(in crate::read) trait BankSlice<'a> {
    const NAME_LENGTH: usize;
    const TYPE_LENGTH: usize;
    const SIZE_LENGTH: usize;
    const FOOTER_LENGTH: usize;
    const HEADER_LENGTH: usize =
        Self::NAME_LENGTH + Self::TYPE_LENGTH + Self::SIZE_LENGTH + Self::FOOTER_LENGTH;
}

impl<'a> BankSlice<'a> for Bank16View<'a> {
    const NAME_LENGTH: usize = crate::B16_NAME_LENGTH;
    const TYPE_LENGTH: usize = crate::B16_DATA_TYPE_LENGTH;
    const SIZE_LENGTH: usize = crate::B16_SIZE_LENGTH;
    const FOOTER_LENGTH: usize = crate::B16_RESERVED_LENGTH;
}

impl<'a> BankSlice<'a> for Bank32View<'a> {
    const NAME_LENGTH: usize = crate::B32_NAME_LENGTH;
    const TYPE_LENGTH: usize = crate::B32_DATA_TYPE_LENGTH;
    const SIZE_LENGTH: usize = crate::B32_SIZE_LENGTH;
    const FOOTER_LENGTH: usize = crate::B32_RESERVED_LENGTH;
}

impl<'a> BankSlice<'a> for Bank32AView<'a> {
    const NAME_LENGTH: usize = crate::B32A_NAME_LENGTH;
    const TYPE_LENGTH: usize = crate::B32A_DATA_TYPE_LENGTH;
    const SIZE_LENGTH: usize = crate::B32A_SIZE_LENGTH;
    const FOOTER_LENGTH: usize = crate::B32A_RESERVED_LENGTH;
}
