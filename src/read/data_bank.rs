use crate::{DataType, Endianness, TryDataTypeFromUnsignedError, BANK_PADDING};
use std::{error::Error, fmt, slice::ChunksExact};

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to a [`BankView`] fails.
#[derive(Clone, Copy, Debug)]
pub enum TryBankViewFromSliceError {
    /// Bank name bytes are not ASCII alphanumeric characters.
    NonAsciiName,
    /// Integer representation of the data type field does not match any known [`DataType`].
    UnknownDataType,
    /// Integer representation of the size field does not match the size of the data slice.
    SizeMismatch,
    /// Data slice length is not divisible by [`DataType::size()`].
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
        Self::UnknownDataType
    }
}

/// An immutable view to a 16-bit data bank.
///
/// A 16-bit data bank is defined as an 8 bytes header followed by its raw data. The binary
/// representation of a 16-bit data bank is:
/// - 4 bytes bank name. Each byte is a valid ASCII alphanumeric character.
/// - 2 bytes unsigned integer representation of the [`DataType`].
/// - 2 bytes (16-bits) unsigned integer representation of the data size `n`.
/// - `n` bytes raw data.
///
/// # Examples
///
/// ```
/// # use midasio::read::data_bank::TryBankViewFromSliceError;
/// # fn main() -> Result<(), TryBankViewFromSliceError> {
/// use midasio::{DataType, read::data_bank::Bank16View};
///
/// let buffer = [66, 65, 78, 75, 1, 0, 3, 0, 100, 200, 255];
/// let data_bank = Bank16View::try_from_le_bytes(&buffer)?;
///
/// assert_eq!("BANK", data_bank.name());
/// assert!(matches!(data_bank.data_type(), DataType::Byte));
/// assert_eq!([100, 200, 255], data_bank.data_slice());
/// assert_eq!(5, data_bank.padding());
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Bank16View<'a> {
    slice: &'a [u8],
    endianness: Endianness,
}

impl<'a> Bank16View<'a> {
    unsafe fn from_le_bytes_unchecked(buffer: &'a [u8]) -> Self {
        Bank16View {
            slice: buffer,
            endianness: Endianness::LittleEndian,
        }
    }
    unsafe fn from_be_bytes_unchecked(buffer: &'a [u8]) -> Self {
        Bank16View {
            slice: buffer,
            endianness: Endianness::BigEndian,
        }
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// little endian.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not a valid [`Bank16View`] with a description as to why the
    /// provided bytes are not a little endian [`Bank16View`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank16View;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 3, 0, 100, 200, 255];
    /// let data_bank = Bank16View::try_from_le_bytes(&buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_le_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        let bank = unsafe { Self::from_le_bytes_unchecked(buffer) };
        error_in_bank_view(&bank)?;
        Ok(bank)
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// big endian.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not a valid [`Bank16View`] with a description as to why the
    /// provided bytes are not a big endian [`Bank16View`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank16View;
    ///
    /// let buffer = [66, 65, 78, 75, 0, 1, 0, 3, 100, 200, 255];
    /// let data_bank = Bank16View::try_from_be_bytes(&buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_be_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        let bank = unsafe { Self::from_be_bytes_unchecked(buffer) };
        error_in_bank_view(&bank)?;
        Ok(bank)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank16View;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 3, 0, 100, 200, 255];
    /// let data_bank = Bank16View::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!("BANK", data_bank.name());
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        std::str::from_utf8(self.name_slice()).unwrap()
    }

    /// Type of data stored in the data bank. For a complete list of the available data types
    /// supported see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::{DataType, read::data_bank::Bank16View};
    ///
    /// let buffer = [66, 65, 78, 75, 6, 0, 4, 0, 100, 155, 200, 255];
    /// let data_bank = Bank16View::try_from_le_bytes(&buffer)?;
    ///
    /// assert!(matches!(data_bank.data_type(), DataType::U32));
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_type(&self) -> DataType {
        let data_type = self.data_type_slice().try_into().unwrap();
        let data_type = match self.endianness {
            Endianness::LittleEndian => u16::from_le_bytes(data_type),
            Endianness::BigEndian => u16::from_be_bytes(data_type),
        };
        DataType::try_from(data_type).unwrap()
    }

    /// Return the raw data as a slice of bytes. This does not include the header of the data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank16View;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 3, 0, 100, 200, 255];
    /// let data_bank = Bank16View::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!([100, 200, 255], data_bank.data_slice());
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_slice(&self) -> &[u8] {
        BankSlice::data_slice(self)
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
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank16View;
    ///
    /// let buffer = [66, 65, 78, 75, 6, 0, 4, 0, 100, 155, 200, 255];
    /// let data_bank = Bank16View::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!(4, data_bank.padding());
    /// # Ok(())
    /// # }
    /// ```
    pub fn padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_PADDING;
        if remainder == 0 {
            0
        } else {
            BANK_PADDING - remainder
        }
    }
}

impl<'a> IntoIterator for &'a Bank16View<'a> {
    /// The type of elements being iterated over. The length of each slice is fixed to [`DataType::size()`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;

    /// Returns an iterator over the [`BankView::data_slice()`] in chunks of size
    /// [`DataType::size()`]. Iterate over individual bytes if the size is [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank16View;
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 4, 0, 100, 155, 100, 155];
    /// let data_bank = Bank16View::try_from_le_bytes(&buffer)?;
    /// let iter = data_bank.into_iter();
    ///
    /// assert_eq!(2, iter.count());
    ///
    /// for u16_slice in &data_bank {
    ///     assert_eq!([100, 155], u16_slice);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct don't have a fixed size, iterate over bytes.
        let item_size = self.data_type().size().unwrap_or(1);
        self.data_slice().chunks_exact(item_size)
    }
}

/// An immutable view to a 32-bit data bank.
///
/// A 32-bit data bank is defined as a 12 bytes header followed by its raw data. The binary
/// representation of a 32-bit data bank is:
/// - 4 bytes bank name. Each byte is a valid ASCII alphanumeric character.
/// - 4 bytes unsigned integer representation of the [`DataType`].
/// - 4 bytes (32-bits) unsigned integer representation of the data size `n`.
/// - `n` bytes raw data.
///
/// # Examples
///
/// ```
/// # use midasio::read::data_bank::TryBankViewFromSliceError;
/// # fn main() -> Result<(), TryBankViewFromSliceError> {
/// use midasio::{DataType, read::data_bank::Bank32View};
///
/// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 3, 0, 0, 0, 100, 200, 255];
/// let data_bank = Bank32View::try_from_le_bytes(&buffer)?;
///
/// assert_eq!("BANK", data_bank.name());
/// assert!(matches!(data_bank.data_type(), DataType::Byte));
/// assert_eq!([100, 200, 255], data_bank.data_slice());
/// assert_eq!(5, data_bank.padding());
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Bank32View<'a> {
    slice: &'a [u8],
    endianness: Endianness,
}

impl<'a> Bank32View<'a> {
    unsafe fn from_le_bytes_unchecked(buffer: &'a [u8]) -> Self {
        Bank32View {
            slice: buffer,
            endianness: Endianness::LittleEndian,
        }
    }
    unsafe fn from_be_bytes_unchecked(buffer: &'a [u8]) -> Self {
        Bank32View {
            slice: buffer,
            endianness: Endianness::BigEndian,
        }
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// little endian.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not a valid [`Bank32View`] with a description as to why the
    /// provided bytes are not a little endian [`Bank32View`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32View;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 3, 0, 0, 0, 100, 200, 255];
    /// let data_bank = Bank32View::try_from_le_bytes(&buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_le_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        let bank = unsafe { Self::from_le_bytes_unchecked(buffer) };
        error_in_bank_view(&bank)?;
        Ok(bank)
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// big endian.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not a valid [`Bank32View`] with a description as to why the
    /// provided bytes are not a big endian [`Bank32View`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32View;
    ///
    /// let buffer = [66, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 3, 100, 200, 255];
    /// let data_bank = Bank32View::try_from_be_bytes(&buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_be_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        let bank = unsafe { Self::from_be_bytes_unchecked(buffer) };
        error_in_bank_view(&bank)?;
        Ok(bank)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32View;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 3, 0, 0, 0, 100, 200, 255];
    /// let data_bank = Bank32View::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!("BANK", data_bank.name());
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        std::str::from_utf8(self.name_slice()).unwrap()
    }

    /// Type of data stored in the data bank. For a complete list of the available data types
    /// supported see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::{DataType, read::data_bank::Bank32View};
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 2, 0, 0, 0, 100, 200];
    /// let data_bank = Bank32View::try_from_le_bytes(&buffer)?;
    ///
    /// assert!(matches!(data_bank.data_type(), DataType::U16));
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_type(&self) -> DataType {
        let data_type = self.data_type_slice().try_into().unwrap();
        let data_type = match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(data_type),
            Endianness::BigEndian => u32::from_be_bytes(data_type),
        };
        DataType::try_from(data_type).unwrap()
    }

    /// Return the raw data as a slice of bytes. This does not include the header of the data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32View;
    ///
    /// let buffer = [66, 65, 78, 75, 6, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    /// let data_bank = Bank32View::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!([100, 155, 200, 255], data_bank.data_slice());
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_slice(&self) -> &[u8] {
        BankSlice::data_slice(self)
    }

    /// If the length of the [`Bank32View::data_slice()`] is not a multiple of 8 bytes, the
    /// subsequent `n = Bank32View::padding()` bytes are reserved until the next multiple of 8.
    ///
    /// These bytes are not part of the data bank slice; it is simply an indication of the number
    /// of reserved bytes between this and the next data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32View;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 3, 0, 0, 0, 100, 200, 255];
    /// let data_bank = Bank32View::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!(5, data_bank.padding());
    /// # Ok(())
    /// # }
    /// ```
    pub fn padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_PADDING;
        if remainder == 0 {
            0
        } else {
            BANK_PADDING - remainder
        }
    }
}

impl<'a> IntoIterator for &'a Bank32View<'a> {
    /// The type of elements being iterated over. The length of each slice is fixed to [`DataType::size()`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;

    /// Returns an iterator over the [`BankView::data_slice()`] in chunks of size
    /// [`DataType::size()`]. Iterate over individual bytes if the size is [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32View;
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 100, 155];
    /// let data_bank = Bank32View::try_from_le_bytes(&buffer)?;
    /// let iter = data_bank.into_iter();
    /// assert_eq!(2, iter.count());
    ///
    /// for u16_slice in &data_bank {
    ///     assert_eq!([100, 155], u16_slice);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct don't have a fixed size, iterate over bytes.
        let item_size = self.data_type().size().unwrap_or(1);
        self.data_slice().chunks_exact(item_size)
    }
}

/// An immutable view to a 32-bit bank 64-bit aligned.
///
/// A 32-bit data bank is defined as a 16 bytes header followed by its raw data. The binary
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
/// # use midasio::read::data_bank::TryBankViewFromSliceError;
/// # fn main() -> Result<(), TryBankViewFromSliceError> {
/// use midasio::{DataType, read::data_bank::Bank32AView};
///
/// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 100, 200, 255];
/// let data_bank = Bank32AView::try_from_le_bytes(&buffer)?;
///
/// assert_eq!("BANK", data_bank.name());
/// assert!(matches!(data_bank.data_type(), DataType::Byte));
/// assert_eq!([100, 200, 255], data_bank.data_slice());
/// assert_eq!(5, data_bank.padding());
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Bank32AView<'a> {
    slice: &'a [u8],
    endianness: Endianness,
}

impl<'a> Bank32AView<'a> {
    unsafe fn from_le_bytes_unchecked(buffer: &'a [u8]) -> Self {
        Bank32AView {
            slice: buffer,
            endianness: Endianness::LittleEndian,
        }
    }
    unsafe fn from_be_bytes_unchecked(buffer: &'a [u8]) -> Self {
        Bank32AView {
            slice: buffer,
            endianness: Endianness::BigEndian,
        }
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// little endian.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not a valid [`Bank32AView`] with a description as to why the
    /// provided bytes are not a little endian [`Bank32AView`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32AView;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 100, 200, 255];
    /// let data_bank = Bank32AView::try_from_le_bytes(&buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_le_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        let bank = unsafe { Self::from_le_bytes_unchecked(buffer) };
        error_in_bank_view(&bank)?;
        Ok(bank)
    }
    /// Create a native view to the underlying data bank from its representation as a byte slice in
    /// big endian.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not a valid [`Bank32AView`] with a description as to why the
    /// provided bytes are not a big endian [`Bank32AView`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32AView;
    ///
    /// let buffer = [66, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 100, 200, 255];
    /// let data_bank = Bank32AView::try_from_be_bytes(&buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_be_bytes(buffer: &'a [u8]) -> Result<Self, TryBankViewFromSliceError> {
        let bank = unsafe { Self::from_be_bytes_unchecked(buffer) };
        error_in_bank_view(&bank)?;
        Ok(bank)
    }
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32AView;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 100, 200, 255];
    /// let data_bank = Bank32AView::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!("BANK", data_bank.name());
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        std::str::from_utf8(self.name_slice()).unwrap()
    }

    /// Type of data stored in the data bank. For a complete list of the available data types
    /// supported see [`DataType`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::{DataType, read::data_bank::Bank32AView};
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 100, 200];
    /// let data_bank = Bank32AView::try_from_le_bytes(&buffer)?;
    ///
    /// assert!(matches!(data_bank.data_type(), DataType::U16));
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_type(&self) -> DataType {
        let data_type = self.data_type_slice().try_into().unwrap();
        let data_type = match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(data_type),
            Endianness::BigEndian => u32::from_be_bytes(data_type),
        };
        DataType::try_from(data_type).unwrap()
    }

    /// Return the raw data as a slice of bytes. This does not include the header of the data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32AView;
    ///
    /// let buffer = [66, 65, 78, 75, 6, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255];
    /// let data_bank = Bank32AView::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!([100, 155, 200, 255], data_bank.data_slice());
    /// # Ok(())
    /// # }
    /// ```
    pub fn data_slice(&self) -> &[u8] {
        BankSlice::data_slice(self)
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
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32AView;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 100, 200, 255];
    /// let data_bank = Bank32AView::try_from_le_bytes(&buffer)?;
    ///
    /// assert_eq!(5, data_bank.padding());
    /// # Ok(())
    /// # }
    /// ```
    pub fn padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_PADDING;
        if remainder == 0 {
            0
        } else {
            BANK_PADDING - remainder
        }
    }
}

impl<'a> IntoIterator for &'a Bank32AView<'a> {
    /// The type of elements being iterated over. The length of each slice is fixed to [`DataType::size()`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;

    /// Returns an iterator over the [`BankView::data_slice()`] in chunks of size
    /// [`DataType::size()`]. Iterate over individual bytes if the size is [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::Bank32AView;
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 100, 155];
    /// let data_bank = Bank32AView::try_from_le_bytes(&buffer)?;
    /// let iter = data_bank.into_iter();
    /// assert_eq!(2, iter.count());
    ///
    /// for u16_slice in &data_bank {
    ///     assert_eq!([100, 155], u16_slice);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct don't have a fixed size, iterate over bytes.
        let item_size = self.data_type().size().unwrap_or(1);
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
/// # use midasio::read::data_bank::TryBankViewFromSliceError;
/// # fn main() -> Result<(), TryBankViewFromSliceError> {
/// use midasio::read::data_bank::{BankView, Bank16View, Bank32View, Bank32AView};
///
/// let buffer = [66, 65, 78, 75, 1, 0, 1, 0, 100];
/// let bank_16 = Bank16View::try_from_le_bytes(&buffer)?;
///
/// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
/// let bank_32 = Bank32View::try_from_le_bytes(&buffer)?;
///
/// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255];
/// let bank_32a = Bank32AView::try_from_le_bytes(&buffer)?;
///
/// let bank_16 = BankView::B16(bank_16);
/// let bank_32 = BankView::B32(bank_32);
/// let bank_32a = BankView::B32A(bank_32a);
///
/// assert_eq!("BANK", bank_16.name());
///
/// assert_eq!(bank_16.is_b16(), true);
/// assert_eq!(bank_16.is_b32(), false);
/// assert_eq!(bank_32a.is_b32a(), true);
/// # Ok(())
/// # }
/// ```
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
    /// Return the name of the data bank. This is guaranteed to be 4 ASCII alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::{BankView, Bank16View};
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 1, 0, 100];
    /// let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer)?);
    ///
    /// assert_eq!("BANK", bank_16.name());
    /// # Ok(())
    /// # }
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
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::{BankView, Bank16View};
    /// use midasio::DataType;
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 1, 0, 100];
    /// let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer)?);
    ///
    /// assert!(matches!(bank_16.data_type(), DataType::Byte));
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

    /// Return the raw data as a slice of bytes. This does not include the header of the data bank.
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::{BankView, Bank16View};
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 3, 0, 100, 200, 255];
    /// let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer)?);
    ///
    /// assert_eq!([100, 200, 255], bank_16.data_slice());
    /// # Ok(())
    /// # }
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
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::{BankView, Bank16View};
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 3, 0, 100, 200, 255];
    /// let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer)?);
    ///
    /// assert_eq!(5, bank_16.padding());
    /// # Ok(())
    /// # }
    /// ```
    pub fn padding(&self) -> usize {
        let remainder = self.data_slice().len() % BANK_PADDING;
        if remainder == 0 {
            0
        } else {
            BANK_PADDING - remainder
        }
    }

    /// Returns [`true`] if this data bank is a [`Bank16View`], and [`false`] otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::{BankView, Bank16View, Bank32View, Bank32AView};
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 1, 0, 100];
    /// let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer)?);
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    /// let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer)?);
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255];
    /// let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer)?);
    ///
    /// assert_eq!(bank_16.is_b16(), true);
    /// assert_eq!(bank_32.is_b16(), false);
    /// assert_eq!(bank_32a.is_b16(), false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_b16(&self) -> bool {
        matches!(self, BankView::B16(_))
    }

    /// Returns [`true`] if this data bank is a [`Bank32View`], and [`false`] otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::{BankView, Bank16View, Bank32View, Bank32AView};
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 1, 0, 100];
    /// let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer)?);
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    /// let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer)?);
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255];
    /// let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer)?);
    ///
    /// assert_eq!(bank_16.is_b32(), false);
    /// assert_eq!(bank_32.is_b32(), true);
    /// assert_eq!(bank_32a.is_b32(), false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_b32(&self) -> bool {
        matches!(self, BankView::B32(_))
    }

    /// Returns [`true`] if this data bank is a [`Bank32AView`], and [`false`] otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::{BankView, Bank16View, Bank32View, Bank32AView};
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 1, 0, 100];
    /// let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer)?);
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    /// let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer)?);
    ///
    /// let buffer = [66, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255];
    /// let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer)?);
    ///
    /// assert_eq!(bank_16.is_b32a(), false);
    /// assert_eq!(bank_32.is_b32a(), false);
    /// assert_eq!(bank_32a.is_b32a(), true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_b32a(&self) -> bool {
        matches!(self, BankView::B32A(_))
    }
}

impl<'a> IntoIterator for &'a BankView<'a> {
    /// The type of elements being iterated over. The length of each slice is fixed to [`DataType::size()`].
    type Item = &'a [u8];
    type IntoIter = ChunksExact<'a, u8>;

    /// Returns an iterator over the [`BankView::data_slice()`] in chunks of size
    /// [`DataType::size()`]. Iterate over individual bytes if the size is [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use midasio::read::data_bank::TryBankViewFromSliceError;
    /// # fn main() -> Result<(), TryBankViewFromSliceError> {
    /// use midasio::read::data_bank::{BankView, Bank32View};
    ///
    /// let buffer = [66, 65, 78, 75, 1, 0, 0, 0, 4, 0, 0, 0, 100, 100, 100, 100];
    /// let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer)?);
    /// let iter = bank_32.into_iter();
    /// assert_eq!(4, iter.count());
    ///
    /// for u8_slice in &bank_32 {
    ///     assert_eq!([100], u8_slice);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        //If the underlying object e.g. struct don't have a fixed size, iterate over bytes.
        let item_size = self.data_type().size().unwrap_or(1);
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
pub(in crate::read) trait BankSlice {
    // Number of ASCII alphanumeric characters that determine the name of the data bank.
    const NAME_LENGTH: usize;
    // Number of bytes that represent the [`DataType`] field in the data bank.
    const TYPE_LENGTH: usize;
    // Number of bytes that represent the size field in the data bank.
    const SIZE_LENGTH: usize;
    // Number of bytes reserved after the bank name, type, and size fields in the header of the
    // data bank.
    const FOOTER_LENGTH: usize;

    const HEADER_LENGTH: usize =
        Self::NAME_LENGTH + Self::TYPE_LENGTH + Self::SIZE_LENGTH + Self::FOOTER_LENGTH;

    // Return a complete slice of bytes that represent a data bank (header plus data).
    fn data_bank_slice(&self) -> &[u8];
    // Return the endianness of the bank
    fn endianness(&self) -> Endianness;

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

impl BankSlice for Bank16View<'_> {
    const NAME_LENGTH: usize = crate::B16_NAME_LENGTH;
    const TYPE_LENGTH: usize = crate::B16_DATA_TYPE_LENGTH;
    const SIZE_LENGTH: usize = crate::B16_SIZE_LENGTH;
    const FOOTER_LENGTH: usize = crate::B16_RESERVED_LENGTH;

    fn data_bank_slice(&self) -> &[u8] {
        self.slice
    }
    fn endianness(&self) -> Endianness {
        self.endianness
    }
}

impl BankSlice for Bank32View<'_> {
    const NAME_LENGTH: usize = crate::B32_NAME_LENGTH;
    const TYPE_LENGTH: usize = crate::B32_DATA_TYPE_LENGTH;
    const SIZE_LENGTH: usize = crate::B32_SIZE_LENGTH;
    const FOOTER_LENGTH: usize = crate::B32_RESERVED_LENGTH;

    fn data_bank_slice(&self) -> &[u8] {
        self.slice
    }
    fn endianness(&self) -> Endianness {
        self.endianness
    }
}

impl BankSlice for Bank32AView<'_> {
    const NAME_LENGTH: usize = crate::B32A_NAME_LENGTH;
    const TYPE_LENGTH: usize = crate::B32A_DATA_TYPE_LENGTH;
    const SIZE_LENGTH: usize = crate::B32A_SIZE_LENGTH;
    const FOOTER_LENGTH: usize = crate::B32A_RESERVED_LENGTH;

    fn data_bank_slice(&self) -> &[u8] {
        self.slice
    }
    fn endianness(&self) -> Endianness {
        self.endianness
    }
}

fn are_all_ascii_alphanumeric(slice: &[u8]) -> bool {
    for letter in slice {
        if !letter.is_ascii_alphanumeric() {
            return false;
        }
    }
    true
}

fn error_in_bank_view<T: BankSlice>(bank: &T) -> Result<(), TryBankViewFromSliceError> {
    if bank.data_bank_slice().len() < T::HEADER_LENGTH {
        return Err(TryBankViewFromSliceError::SizeMismatch);
    }

    match (T::SIZE_LENGTH, T::TYPE_LENGTH) {
        (4, 4) => {
            let size = match bank.endianness() {
                Endianness::LittleEndian => {
                    u32::from_le_bytes(bank.data_size_slice().try_into().unwrap())
                }
                Endianness::BigEndian => {
                    u32::from_be_bytes(bank.data_size_slice().try_into().unwrap())
                }
            };
            if bank.data_slice().len() != size.try_into().unwrap() {
                return Err(TryBankViewFromSliceError::SizeMismatch);
            }
            let data_type = match bank.endianness() {
                Endianness::LittleEndian => {
                    u32::from_le_bytes(bank.data_type_slice().try_into().unwrap())
                }
                Endianness::BigEndian => {
                    u32::from_be_bytes(bank.data_type_slice().try_into().unwrap())
                }
            };
            let data_type = match DataType::try_from(data_type) {
                Ok(data_type) => data_type,
                Err(error) => return Err(error.into()),
            };
            if let Some(type_size) = data_type.size() {
                if size % u32::try_from(type_size).unwrap() != 0 {
                    return Err(TryBankViewFromSliceError::IncompleteData);
                }
            }
        }
        (2, 2) => {
            let size = match bank.endianness() {
                Endianness::LittleEndian => {
                    u16::from_le_bytes(bank.data_size_slice().try_into().unwrap())
                }
                Endianness::BigEndian => {
                    u16::from_be_bytes(bank.data_size_slice().try_into().unwrap())
                }
            };
            if bank.data_slice().len() != size.into() {
                return Err(TryBankViewFromSliceError::SizeMismatch);
            }
            let data_type = match bank.endianness() {
                Endianness::LittleEndian => {
                    u16::from_le_bytes(bank.data_type_slice().try_into().unwrap())
                }
                Endianness::BigEndian => {
                    u16::from_be_bytes(bank.data_type_slice().try_into().unwrap())
                }
            };
            let data_type = match DataType::try_from(data_type) {
                Ok(data_type) => data_type,
                Err(error) => return Err(error.into()),
            };
            if let Some(type_size) = data_type.size() {
                if size % u16::try_from(type_size).unwrap() != 0 {
                    return Err(TryBankViewFromSliceError::IncompleteData);
                }
            }
        }
        _ => unreachable!(),
    }

    if !are_all_ascii_alphanumeric(bank.name_slice()) {
        return Err(TryBankViewFromSliceError::NonAsciiName);
    }

    Ok(())
}

#[cfg(test)]
mod tests;
