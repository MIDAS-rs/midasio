use crate::{DataType, Endianness, TryDataTypeFromUnsignedError, BANK_PADDING};
use std::{error::Error, fmt, slice::ChunksExact};

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to a [`BankView`] fails.
#[derive(Debug)]
pub enum TryBankViewFromSliceError {
    /// Bank name bytes are not ASCII alphanumeric characters.
    NonAsciiName,
    /// Integer representation of the data type field does not match any known [`DataType`].
    UnknownDataType,
    /// Integer representation of the size field does not match the size of the data slice.
    SizeMismatch,
    /// Data slice size is not divisible by [`DataType::size()`].
    IncompleteData,
}
impl fmt::Display for TryBankViewFromSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryBankViewFromSliceError::NonAsciiName => {
                write!(f, "non-ASCII alphanumeric bank name")
            }
            TryBankViewFromSliceError::UnknownDataType => write!(f, "unknown data type"),
            TryBankViewFromSliceError::SizeMismatch => {
                write!(f, "size field and data slice mismatch")
            }
            TryBankViewFromSliceError::IncompleteData => write!(f, "corrupted/incomplete data"),
        }
    }
}
impl Error for TryBankViewFromSliceError {}
impl From<TryDataTypeFromUnsignedError> for TryBankViewFromSliceError {
    fn from(_: TryDataTypeFromUnsignedError) -> Self {
        TryBankViewFromSliceError::UnknownDataType
    }
}

/// An immutable view to a 16-bit data bank.
///
/// A 16-bit data bank is defines as an 8 bytes header followed by its raw data. The binary
/// representation of a 16-bit data bank is:
/// - 4 bytes bank name. Each byte is a valid ASCII alphanumeric character.
/// - 2 bytes unsigned integer representation of the [`DataType`].
/// - 2 bytes (16-bits) unsigned integer representation of the data size `n`.
/// - `n` bytes raw data.
///
/// # Examples
///
/// ```
/// todo!()
/// ```
pub struct Bank16View<'a> {
    slice: &'a [u8],
    endianness: Endianness,
}

impl<'a> Bank16View<'a> {
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn try_from_le_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        todo!()
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn try_from_be_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        todo!()
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn name(&self) -> &str {
        todo!()
    }

    /// Type of data stored in the data bank. For a complete list of the available data types
    /// supported see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn data_type(&self) -> DataType {
        todo!()
    }

    /// Return the raw data as a slice of bytes. This does not include the header of the data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn data_slice(&self) -> &[u8] {
        todo!()
    }

    /// If the length of the [`Bank16View::data_slice()`] is not a multiple of 8 bytes, the
    /// subsequent `n = Bank16View::padding()` bytes are reserved until the next multiple of 8.
    ///
    /// These bytes are not part of the data bank slice; it is simply an indication of the number
    /// of reserved bytes between this and the next data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_PADDING;
        if remainder != 0 {
            BANK_PADDING - remainder
        } else {
            0
        }
    }
}

impl<'a> IntoIterator for &'a Bank16View<'a> {
    /// The type of elements being iterated over. The length of each slice is fixed to [`DataType::size()`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;

    /// Returns an iterator over the [`BankView::data_slice()`] in chunks of size
    /// [`DataType::size()`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        let item_size = self.data_type().size().unwrap();
        self.data_slice().chunks_exact(item_size)
    }
}

/// An immutable view to a 32-bit data bank.
///
/// A 32-bit data bank is defines as an 12 bytes header followed by its raw data. The binary
/// representation of a 32-bit data bank is:
/// - 4 bytes bank name. Each byte is a valid ASCII alphanumeric character.
/// - 4 bytes unsigned integer representation of the [`DataType`].
/// - 4 bytes (32-bits) unsigned integer representation of the data size `n`.
/// - `n` bytes raw data.
///
/// # Examples
///
/// ```
/// todo!()
/// ```
pub struct Bank32View<'a> {
    slice: &'a [u8],
    endianness: Endianness,
}

impl<'a> Bank32View<'a> {
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn try_from_le_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        todo!()
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn try_from_be_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        todo!()
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn name(&self) -> &str {
        todo!()
    }

    /// Type of data stored in the data bank. For a complete list of the available data types
    /// supported see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn data_type(&self) -> DataType {
        todo!()
    }

    /// Return the raw data as a slice of bytes. This does not include the header of the data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn data_slice(&self) -> &[u8] {
        todo!()
    }

    /// If the length of the [`Bank16View::data_slice()`] is not a multiple of 8 bytes, the
    /// subsequent `n = Bank16View::padding()` bytes are reserved until the next multiple of 8.
    ///
    /// These bytes are not part of the data bank slice; it is simply an indication of the number
    /// of reserved bytes between this and the next data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_PADDING;
        if remainder != 0 {
            BANK_PADDING - remainder
        } else {
            0
        }
    }
}

impl<'a> IntoIterator for &'a Bank32View<'a> {
    /// The type of elements being iterated over. The length of each slice is fixed to [`DataType::size()`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;

    /// Returns an iterator over the [`BankView::data_slice()`] in chunks of size
    /// [`DataType::size()`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        let item_size = self.data_type().size().unwrap();
        self.data_slice().chunks_exact(item_size)
    }
}

/// An immutable view to a 32-bit bank 64-bit aligned.
///
/// A 32-bit data bank is defines as an 16 bytes header followed by its raw data. The binary
/// representation of a 32-bit data bank 64-bit aligned is:
/// - 4 bytes bank name. Each byte is a valid ASCII alphanumeric character.
/// - 4 bytes unsigned integer representation of the [`DataType`].
/// - 4 bytes (32-bits) unsigned integer representation of the data size `n`.
/// - 4 bytes reserved.
/// - `n` bytes raw data.
///
/// # Examples
///
/// ```
/// todo!()
/// ```
pub struct Bank32AView<'a> {
    slice: &'a [u8],
    endianness: Endianness,
}

impl<'a> Bank32AView<'a> {
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn try_from_le_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        todo!()
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn try_from_be_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        todo!()
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn name(&self) -> &str {
        todo!()
    }

    /// Type of data stored in the data bank. For a complete list of the available data types
    /// supported see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn data_type(&self) -> DataType {
        todo!()
    }

    /// Return the raw data as a slice of bytes. This does not include the header of the data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn data_slice(&self) -> &[u8] {
        todo!()
    }

    /// If the length of the [`Bank16View::data_slice()`] is not a multiple of 8 bytes, the
    /// subsequent `n = Bank16View::padding()` bytes are reserved until the next multiple of 8.
    ///
    /// These bytes are not part of the data bank slice; it is simply an indication of the number
    /// of reserved bytes between this and the next data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_PADDING;
        if remainder != 0 {
            BANK_PADDING - remainder
        } else {
            0
        }
    }
}

impl<'a> IntoIterator for &'a Bank32AView<'a> {
    /// The type of elements being iterated over. The length of each slice is fixed to [`DataType::size()`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;

    /// Returns an iterator over the [`BankView::data_slice()`] in chunks of size
    /// [`DataType::size()`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        let item_size = self.data_type().size().unwrap();
        self.data_slice().chunks_exact(item_size)
    }
}

/// An immutable view to a data bank.
///
/// This enum can contain either a [`Bank16View`], a [`Bank32View`], or a [`Bank32AView`]. See
/// their respective documentation for more details.
///
/// # Examples
///
/// ```
/// todo!()
/// ```
pub enum BankView<'a> {
    /// A 16-bit bank.
    B16(Bank16View<'a>),
    /// A 32-bit bank.
    B32(Bank32View<'a>),
    /// A 32-bit bank 64-bit aligned.
    B32A(Bank32AView<'a>),
}

impl<'a> BankView<'a> {
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn name(&self) -> &str {
        match self {
            BankView::B16(bank) => bank.name(),
            BankView::B32(bank) => bank.name(),
            BankView::B32A(bank) => bank.name(),
        }
    }

    /// Type of data stored in the data bank. For a complete list of the available data types
    /// supported see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn data_type(&self) -> DataType {
        match self {
            BankView::B16(bank) => bank.data_type(),
            BankView::B32(bank) => bank.data_type(),
            BankView::B32A(bank) => bank.data_type(),
        }
    }

    /// Return the raw data as a slice of bytes. This does not include the header of the data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn data_slice(&self) -> &[u8] {
        match self {
            BankView::B16(bank) => bank.data_slice(),
            BankView::B32(bank) => bank.data_slice(),
            BankView::B32A(bank) => bank.data_slice(),
        }
    }

    /// If the length of the [`BankView::data_slice()`] is not a multiple of 8 bytes, the
    /// subsequent `n = BankView::padding()` bytes are reserved until the next multiple of 8.
    ///
    /// These bytes are not part of the data bank slice; it is simply an indication of the number
    /// of reserved bytes between this and the next data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    pub fn padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_PADDING;
        if remainder != 0 {
            BANK_PADDING - remainder
        } else {
            0
        }
    }
}

impl<'a> IntoIterator for &'a BankView<'a> {
    /// The type of elements being iterated over. The length of each slice is fixed to [`DataType::size()`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;

    /// Returns an iterator over the [`BankView::data_slice()`] in chunks of size
    /// [`DataType::size()`].
    ///
    /// # Examples
    ///
    /// ```
    /// todo!()
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        let item_size = self.data_type().size().unwrap();
        self.data_slice().chunks_exact(item_size)
    }
}

// Private trait to represent structures that are data banks. These methods simplify the runtime
// checks required to validate the correctness of a data bank.
//
// In general, implementors of this trait should have (in their constructors) runtime checks to
// validate the following (per MIDAS documentation):
// 1) The size of the raw slice is at least as big as the header.
// 2) All characters from the "bank name" are valid ASCII alphanumeric.
// 3) The "type" field should be a valid unsigned integer representation of the MIDAS-supported
//    types (see [`Type`] enum and the valid TryFrom conversions).
// 4) The "size" field correctly matches the size of the data in the bank.
// 5) The data slice should contain an integer number of [`DataType`] items.
trait BankSlice {
    // Number of ASCII alphanumeric characters that determine the name of the data bank.
    const NAME_LENGTH: usize;
    // Number of bytes that represent the [`DataType`] field in the data bank.
    const TYPE_LENGTH: usize;
    // Number of bytes that represent the size field in the data bank.
    const SIZE_LENGTH: usize;
    // Number of bytes reserved after the bank name, type, and size fields in the header of the
    // data bank.
    const FOOTER_LENGTH: usize;

    // Return a complete slice of bytes that represent a data bank (header plus data).
    fn data_bank_slice(&self) -> &[u8];

    // Return the slice of bytes in the data bank which represent its name.
    fn name_slice(&self) -> &[u8] {
        &self.data_bank_slice()[..Self::NAME_LENGTH]
    }

    // Return the slice of bytes in the data bank which represent the data type.
    fn data_type_slice(&self) -> &[u8] {
        let offset = Self::NAME_LENGTH;
        &self.data_bank_slice()[offset..][..Self::TYPE_LENGTH]
    }

    // Return the slice of bytes in the bank which represent the data size.
    fn data_size_slice(&self) -> &[u8] {
        let offset = Self::NAME_LENGTH + Self::TYPE_LENGTH;
        &self.data_bank_slice()[offset..][..Self::SIZE_LENGTH]
    }

    // Return the reserved bytes between the header and the data.
    fn header_footer_slice(&self) -> &[u8] {
        let offset = Self::NAME_LENGTH + Self::TYPE_LENGTH + Self::SIZE_LENGTH;
        &self.data_bank_slice()[offset..][..Self::FOOTER_LENGTH]
    }

    // Return the actual data slice.
    fn data_slice(&self) -> &[u8] {
        let offset =
            Self::NAME_LENGTH + Self::TYPE_LENGTH + Self::SIZE_LENGTH + Self::FOOTER_LENGTH;
        &self.data_bank_slice()[offset..]
    }
}

#[cfg(test)]
mod tests;
