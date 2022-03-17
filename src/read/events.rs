use crate::read::data_banks::{Bank16View, Bank32AView, Bank32View, BankSlice, BankView};
use crate::{BankType, Endianness};
use crate::{
    EVENT_ALL_BANKS_SIZE_LENGTH, EVENT_FLAGS_LENGTH, EVENT_ID_LENGTH, EVENT_SERIAL_NUMBER_LENGTH,
    EVENT_SIZE_LENGTH, EVENT_TIME_STAMP_LENGTH, EVENT_TRIGGER_MASK_LENGTH,
};
use std::{error::Error, fmt};

/// An iterator over a slice in (non-overlapping) [`Bank16View`]s, starting at the beginning of the
/// slice.
///
/// When the slice len is not evenly divided by the data bank view (plus its padding bytes), the
/// last slice of the iteration can be obtained with the [`Bank16Views::remainder()`].
///
/// # Examples
///
/// ```
/// use midasio::read::events::Bank16Views;
///
/// let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6];
/// let padding = [0u8, 0, 1, 1];
/// let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
/// let mut banks = Bank16Views::from_be_bytes(&banks);
///
/// banks.next();
/// assert_eq!([1, 1], banks.remainder());
/// ```
pub struct Bank16Views<'a> {
    curr: usize,
    slice: &'a [u8],
    endianness: Endianness,
}
impl<'a> Bank16Views<'a> {
    /// Create an iterator over a slice where the underlying [`Bank16View`]s are little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank16Views;
    ///
    /// let bank_16 = [66u8, 65, 78, 75, 1, 0, 6, 0, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0];
    /// let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    /// let banks = Bank16Views::from_le_bytes(&banks);
    ///
    /// assert_eq!(1, banks.count());
    /// ```
    pub fn from_le_bytes(buffer: &'a [u8]) -> Self {
        Bank16Views {
            curr: 0,
            slice: buffer,
            endianness: Endianness::LittleEndian,
        }
    }
    /// Create an iterator over a slice where the underlying [`Bank16View`]s are big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank16Views;
    ///
    /// let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0];
    /// let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    /// let banks = Bank16Views::from_be_bytes(&banks);
    ///
    /// assert_eq!(1, banks.count());
    /// ```
    pub fn from_be_bytes(buffer: &'a [u8]) -> Self {
        Bank16Views {
            curr: 0,
            slice: buffer,
            endianness: Endianness::BigEndian,
        }
    }
    /// Return, at any given step, the portion of the original slice which hasn't been iterated
    /// over.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank16Views;
    ///
    /// let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0, 1, 1];
    /// let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    /// let mut banks = Bank16Views::from_be_bytes(&banks);
    ///
    /// assert_eq!([66, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6, 0, 0, 1, 1], banks.remainder());
    ///
    /// banks.next();
    /// assert_eq!([1, 1], banks.remainder());
    /// ```
    pub fn remainder(&self) -> &[u8] {
        &self.slice[self.curr..]
    }
}
impl<'a> Iterator for Bank16Views<'a> {
    type Item = Bank16View<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.len() - self.curr < Bank16View::HEADER_LENGTH {
            return None;
        }

        let offset = Bank16View::NAME_LENGTH + Bank16View::TYPE_LENGTH;
        let size = self.slice[self.curr + offset..][..Bank16View::SIZE_LENGTH]
            .try_into()
            .unwrap();
        let size: usize = match self.endianness {
            Endianness::LittleEndian => u16::from_le_bytes(size).into(),
            Endianness::BigEndian => u16::from_be_bytes(size).into(),
        };
        if self.slice.len() - self.curr < size + Bank16View::HEADER_LENGTH {
            return None;
        }

        let data_bank = &self.slice[self.curr..][..Bank16View::HEADER_LENGTH + size];
        let data_bank = match self.endianness {
            Endianness::LittleEndian => Bank16View::try_from_le_bytes(data_bank),
            Endianness::BigEndian => Bank16View::try_from_be_bytes(data_bank),
        };
        match data_bank {
            Ok(data_bank) => {
                let advance = Bank16View::HEADER_LENGTH + size + data_bank.padding();
                if advance > self.slice.len() - self.curr {
                    return None;
                }
                self.curr += advance;
                Some(data_bank)
            }
            Err(_) => None,
        }
    }
}

/// An iterator over a slice in (non-overlapping) [`Bank32View`]s, starting at the beginning of the
/// slice.
///
/// When the slice len is not evenly divided by the data bank view (plus its padding bytes), the
/// last slice of the iteration can be obtained with the [`Bank32Views::remainder()`].
///
/// # Examples
///
/// ```
/// use midasio::read::events::Bank32Views;
///
/// let bank_32 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 1, 2, 3, 4, 5, 6];
/// let padding = [0u8, 0, 1, 1];
/// let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
/// let mut banks = Bank32Views::from_be_bytes(&banks);
///
/// banks.next();
/// assert_eq!([1, 1], banks.remainder());
/// ```
pub struct Bank32Views<'a> {
    curr: usize,
    slice: &'a [u8],
    endianness: Endianness,
}
impl<'a> Bank32Views<'a> {
    /// Create an iterator over a slice where the underlying [`Bank32View`]s are little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank32Views;
    ///
    /// let bank_32 = [66u8, 65, 78, 75, 1, 0, 0, 0, 6, 0, 0, 0, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0];
    /// let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
    /// let banks = Bank32Views::from_le_bytes(&banks);
    ///
    /// assert_eq!(1, banks.count());
    /// ```
    pub fn from_le_bytes(buffer: &'a [u8]) -> Self {
        Bank32Views {
            curr: 0,
            slice: buffer,
            endianness: Endianness::LittleEndian,
        }
    }
    /// Create an iterator over a slice where the underlying [`Bank32View`]s are big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank32Views;
    ///
    /// let bank_32 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0];
    /// let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
    /// let banks = Bank32Views::from_be_bytes(&banks);
    ///
    /// assert_eq!(1, banks.count());
    /// ```
    pub fn from_be_bytes(buffer: &'a [u8]) -> Self {
        Bank32Views {
            curr: 0,
            slice: buffer,
            endianness: Endianness::BigEndian,
        }
    }
    /// Return, at any given step, the portion of the original slice which hasn't been iterated
    /// over.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank32Views;
    ///
    /// let bank_32 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0, 1, 1];
    /// let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
    /// let mut banks = Bank32Views::from_be_bytes(&banks);
    ///
    /// assert_eq!([66, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 1, 2, 3, 4, 5, 6, 0, 0, 1, 1], banks.remainder());
    ///
    /// banks.next();
    /// assert_eq!([1, 1], banks.remainder());
    /// ```
    pub fn remainder(&self) -> &[u8] {
        &self.slice[self.curr..]
    }
}
impl<'a> Iterator for Bank32Views<'a> {
    type Item = Bank32View<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.len() - self.curr < Bank32View::HEADER_LENGTH {
            return None;
        }

        let offset = Bank32View::NAME_LENGTH + Bank32View::TYPE_LENGTH;
        let size = self.slice[self.curr + offset..][..Bank32View::SIZE_LENGTH]
            .try_into()
            .unwrap();
        let size: usize = match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(size).try_into().unwrap(),
            Endianness::BigEndian => u32::from_be_bytes(size).try_into().unwrap(),
        };
        if self.slice.len() - self.curr < size + Bank32View::HEADER_LENGTH {
            return None;
        }

        let data_bank = &self.slice[self.curr..][..Bank32View::HEADER_LENGTH + size];
        let data_bank = match self.endianness {
            Endianness::LittleEndian => Bank32View::try_from_le_bytes(data_bank),
            Endianness::BigEndian => Bank32View::try_from_be_bytes(data_bank),
        };
        match data_bank {
            Ok(data_bank) => {
                let advance = Bank32View::HEADER_LENGTH + size + data_bank.padding();
                if advance > self.slice.len() - self.curr {
                    return None;
                }
                self.curr += advance;
                Some(data_bank)
            }
            Err(_) => None,
        }
    }
}

/// An iterator over a slice in (non-overlapping) [`Bank32AView`]s, starting at the beginning of the
/// slice.
///
/// When the slice len is not evenly divided by the data bank view (plus its padding bytes), the
/// last slice of the iteration can be obtained with the [`Bank32AViews::remainder()`].
///
/// # Examples
///
/// ```
/// use midasio::read::events::Bank32AViews;
///
/// let bank_32a = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6];
/// let padding = [0u8, 0, 1, 1];
/// let banks: Vec<u8> = bank_32a.into_iter().chain(padding.into_iter()).collect();
/// let mut banks = Bank32AViews::from_be_bytes(&banks);
///
/// banks.next();
/// assert_eq!([1, 1], banks.remainder());
/// ```
pub struct Bank32AViews<'a> {
    curr: usize,
    slice: &'a [u8],
    endianness: Endianness,
}
impl<'a> Bank32AViews<'a> {
    /// Create an iterator over a slice where the underlying [`Bank32AView`]s are little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank32AViews;
    ///
    /// let bank_32a = [66u8, 65, 78, 75, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0];
    /// let banks: Vec<u8> = bank_32a.into_iter().chain(padding.into_iter()).collect();
    /// let banks = Bank32AViews::from_le_bytes(&banks);
    ///
    /// assert_eq!(1, banks.count());
    /// ```
    pub fn from_le_bytes(buffer: &'a [u8]) -> Self {
        Bank32AViews {
            curr: 0,
            slice: buffer,
            endianness: Endianness::LittleEndian,
        }
    }
    /// Create an iterator over a slice where the underlying [`Bank32AView`]s are big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank32AViews;
    ///
    /// let bank_32a = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0];
    /// let banks: Vec<u8> = bank_32a.into_iter().chain(padding.into_iter()).collect();
    /// let banks = Bank32AViews::from_be_bytes(&banks);
    ///
    /// assert_eq!(1, banks.count());
    /// ```
    pub fn from_be_bytes(buffer: &'a [u8]) -> Self {
        Bank32AViews {
            curr: 0,
            slice: buffer,
            endianness: Endianness::BigEndian,
        }
    }
    /// Return, at any given step, the portion of the original slice which hasn't been iterated
    /// over.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::Bank32AViews;
    ///
    /// let bank_32a = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0, 1, 1];
    /// let banks: Vec<u8> = bank_32a.into_iter().chain(padding.into_iter()).collect();
    /// let mut banks = Bank32AViews::from_be_bytes(&banks);
    ///
    /// assert_eq!([66, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 0, 0, 1, 1], banks.remainder());
    ///
    /// banks.next();
    /// assert_eq!([1, 1], banks.remainder());
    /// ```
    pub fn remainder(&self) -> &[u8] {
        &self.slice[self.curr..]
    }
}
impl<'a> Iterator for Bank32AViews<'a> {
    type Item = Bank32AView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.len() - self.curr < Bank32AView::HEADER_LENGTH {
            return None;
        }

        let offset = Bank32AView::NAME_LENGTH + Bank32AView::TYPE_LENGTH;
        let size = self.slice[self.curr + offset..][..Bank32AView::SIZE_LENGTH]
            .try_into()
            .unwrap();
        let size: usize = match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(size).try_into().unwrap(),
            Endianness::BigEndian => u32::from_be_bytes(size).try_into().unwrap(),
        };
        if self.slice.len() - self.curr < size + Bank32AView::HEADER_LENGTH {
            return None;
        }

        let data_bank = &self.slice[self.curr..][..Bank32AView::HEADER_LENGTH + size];
        let data_bank = match self.endianness {
            Endianness::LittleEndian => Bank32AView::try_from_le_bytes(data_bank),
            Endianness::BigEndian => Bank32AView::try_from_be_bytes(data_bank),
        };
        match data_bank {
            Ok(data_bank) => {
                let advance = Bank32AView::HEADER_LENGTH + size + data_bank.padding();
                if advance > self.slice.len() - self.curr {
                    return None;
                }
                self.curr += advance;
                Some(data_bank)
            }
            Err(_) => None,
        }
    }
}

/// An iterator over a slice in (non-overlapping) [`BankView`]s, starting at the beginning of the
/// slice.
///
/// When the slice len is not evenly divided by the data bank view (plus its padding bytes), the
/// last slice of the iteration can be obtained with the [`BankViews::remainder()`].
///
/// # Examples
///
/// ```
/// use midasio::read::events::{BankViews, Bank16Views};
///
/// let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6];
/// let padding = [0u8, 0, 1, 1];
/// let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
/// let mut banks = BankViews::B16(Bank16Views::from_be_bytes(&banks));
///
/// banks.next();
/// assert_eq!([1, 1], banks.remainder());
/// ```
pub enum BankViews<'a> {
    B16(Bank16Views<'a>),
    B32(Bank32Views<'a>),
    B32A(Bank32AViews<'a>),
}
impl<'a> BankViews<'a> {
    /// Return, at any given step, the portion of the original slice which hasn't been iterated
    /// over.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::{BankViews, Bank16Views};
    ///
    /// let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6];
    /// let padding = [0u8, 0, 1, 1];
    /// let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    /// let mut banks = BankViews::B16(Bank16Views::from_be_bytes(&banks));
    ///
    /// assert_eq!([66, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6, 0, 0, 1, 1], banks.remainder());
    ///
    /// banks.next();
    /// assert_eq!([1, 1], banks.remainder());
    /// ```
    pub fn remainder(&self) -> &[u8] {
        match self {
            BankViews::B16(iter) => &iter.slice[iter.curr..],
            BankViews::B32(iter) => &iter.slice[iter.curr..],
            BankViews::B32A(iter) => &iter.slice[iter.curr..],
        }
    }
}
impl<'a> Iterator for BankViews<'a> {
    type Item = BankView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BankViews::B16(iter) => iter.next().map(BankView::B16),
            BankViews::B32(iter) => iter.next().map(BankView::B32),
            BankViews::B32A(iter) => iter.next().map(BankView::B32A),
        }
    }
}

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to an [`EventView`] fails.
#[derive(Clone, Copy, Debug)]
pub enum TryEventViewFromSliceError {
    /// The event size and the size of all banks don't match.
    EventAndBanksMismatch,
    /// Integer representation of all banks size does not match the length of all banks slice.
    SizeMismatch,
    /// Error converting a sub-slice into a [`BankView`].
    BadBank,
    /// Unknown flag
    UnknownFlag,
}
impl fmt::Display for TryEventViewFromSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryEventViewFromSliceError::EventAndBanksMismatch => {
                write!(f, "event size and all banks size (header fields) mismatch")
            }
            TryEventViewFromSliceError::SizeMismatch => {
                write!(f, "size field and all banks slice mismatch")
            }
            TryEventViewFromSliceError::BadBank => {
                write!(f, "sub-slice incompatible with a bank view")
            }
            TryEventViewFromSliceError::UnknownFlag => write!(f, "unknown flag"),
        }
    }
}
impl Error for TryEventViewFromSliceError {}

/// An immutable view to a MIDAS event.
///
/// An event is defined as a 24 bytes header followed by an arbitrary number of [`BankView`]s. The
/// binary representation of a MIDAS event is:
/// - 2 bytes event id.
/// - 2 bytes trigger mask.
/// - 4 bytes serial number.
/// - 4 bytes time stamp.
/// - 4 bytes event size. It doesn't include the 12 bytes from the event id, trigger mask, serial
/// number, and time stamp.
/// - 4 bytes size of all banks. Redundant. Equal to event size minus 8 bytes.
/// - 4 bytes flags. Determines the variant of the following [`BankView`]s.
/// - Arbitrary number of [`BankView`]s.
///
/// # Examples
///
/// ```
/// use midasio::read::events::EventView;
///
/// let header = [
///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
/// ];
/// let bank1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
/// let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
/// let bank2 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
/// let padding2 = [0u8, 0, 0, 0, 0, 0, 0];
///
/// let event: Vec<u8> = header
///     .into_iter()
///     .chain(bank1.into_iter())
///     .chain(padding1.into_iter())
///     .chain(bank2.into_iter())
///     .chain(padding2.into_iter())
///     .collect();
/// let event = EventView::try_from_be_bytes(&event).unwrap();
///
/// for bank in &event {
///     assert_eq!("BANK", bank.name());
///     assert_eq!([255], bank.data_slice());
/// }
/// ```
pub struct EventView<'a> {
    slice: &'a [u8],
    // MIDAS documentation is very poor. Somewhere it says that the "flags" field of the event is
    // used for endianness detection. The first "flags" field is far into the file (after the ODB
    // dump) so you should know endianness before hand to get there.
    // At some other point it says that endianness of the file is fixed to that of the system that
    // wrote the file. In other words, guess the endianness. If LittleEndian doesn't work, then it
    // must be BigEndian; it is EXTREMELY unlikely both will yield valid sizes, ASCII, etc.
    endianness: Endianness,
}

impl<'a> EventView<'a> {
    unsafe fn from_le_bytes_unchecked(buffer: &'a [u8]) -> Self {
        EventView {
            slice: buffer,
            endianness: Endianness::LittleEndian,
        }
    }
    unsafe fn from_be_bytes_unchecked(buffer: &'a [u8]) -> Self {
        EventView {
            slice: buffer,
            endianness: Endianness::BigEndian,
        }
    }
    /// Create a native view to a MIDAS event from its representation as a byte slice in little
    /// endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::EventView;
    ///
    /// let header = [
    ///     1u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .collect();
    /// let event = EventView::try_from_le_bytes(&event).unwrap();
    /// ```
    pub fn try_from_le_bytes(buffer: &'a [u8]) -> Result<Self, TryEventViewFromSliceError> {
        let event = unsafe { Self::from_le_bytes_unchecked(buffer) };
        match error_in_event_view(&event) {
            Some(error) => Err(error),
            None => Ok(event),
        }
    }
    /// Create a native view to a MIDAS event from its representation as a byte slice in little
    /// endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::EventView;
    ///
    /// let header = [
    ///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .collect();
    /// let event = EventView::try_from_be_bytes(&event).unwrap();
    /// ```
    pub fn try_from_be_bytes(buffer: &'a [u8]) -> Result<Self, TryEventViewFromSliceError> {
        let event = unsafe { Self::from_be_bytes_unchecked(buffer) };
        match error_in_event_view(&event) {
            Some(error) => Err(error),
            None => Ok(event),
        }
    }
    /// Return the ID of a MIDAS event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::EventView;
    ///
    /// let header = [
    ///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .collect();
    /// let event = EventView::try_from_be_bytes(&event).unwrap();
    ///
    /// assert_eq!(1, event.id());
    /// ```
    pub fn id(&self) -> u16 {
        let id = self.slice[..EVENT_ID_LENGTH].try_into().unwrap();
        match self.endianness {
            Endianness::LittleEndian => u16::from_le_bytes(id),
            Endianness::BigEndian => u16::from_be_bytes(id),
        }
    }
    /// Return the trigger mask of a MIDAS event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::EventView;
    ///
    /// let header = [
    ///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .collect();
    /// let event = EventView::try_from_be_bytes(&event).unwrap();
    ///
    /// assert_eq!(2, event.trigger_mask());
    /// ```
    pub fn trigger_mask(&self) -> u16 {
        let offset = EVENT_ID_LENGTH;
        let trigger_mask = self.slice[offset..][..EVENT_TRIGGER_MASK_LENGTH]
            .try_into()
            .unwrap();
        match self.endianness {
            Endianness::LittleEndian => u16::from_le_bytes(trigger_mask),
            Endianness::BigEndian => u16::from_be_bytes(trigger_mask),
        }
    }
    /// Return the serial number of a MIDAS event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::EventView;
    ///
    /// let header = [
    ///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .collect();
    /// let event = EventView::try_from_be_bytes(&event).unwrap();
    ///
    /// assert_eq!(3, event.serial_number());
    /// ```
    pub fn serial_number(&self) -> u32 {
        let offset = EVENT_ID_LENGTH + EVENT_TRIGGER_MASK_LENGTH;
        let serial_number = self.slice[offset..][..EVENT_SERIAL_NUMBER_LENGTH]
            .try_into()
            .unwrap();
        match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(serial_number),
            Endianness::BigEndian => u32::from_be_bytes(serial_number),
        }
    }
    /// Return the time stamp of a MIDAS event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::EventView;
    ///
    /// let header = [
    ///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .collect();
    /// let event = EventView::try_from_be_bytes(&event).unwrap();
    ///
    /// assert_eq!(4, event.time_stamp());
    /// ```
    pub fn time_stamp(&self) -> u32 {
        let offset = EVENT_ID_LENGTH + EVENT_TRIGGER_MASK_LENGTH + EVENT_SERIAL_NUMBER_LENGTH;
        let time_stamp = self.slice[offset..][..EVENT_TIME_STAMP_LENGTH]
            .try_into()
            .unwrap();
        match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(time_stamp),
            Endianness::BigEndian => u32::from_be_bytes(time_stamp),
        }
    }
    /// Return the unsigned integer representation of the flags of a MIDAS event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::EventView;
    ///
    /// let header = [
    ///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .collect();
    /// let event = EventView::try_from_be_bytes(&event).unwrap();
    ///
    /// assert_eq!(1, event.flags());
    /// ```
    pub fn flags(&self) -> u32 {
        let offset = EVENT_ID_LENGTH
            + EVENT_TRIGGER_MASK_LENGTH
            + EVENT_SERIAL_NUMBER_LENGTH
            + EVENT_TIME_STAMP_LENGTH
            + EVENT_SIZE_LENGTH
            + EVENT_ALL_BANKS_SIZE_LENGTH;
        let flags = self.slice[offset..][..EVENT_FLAGS_LENGTH]
            .try_into()
            .unwrap();
        match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(flags),
            Endianness::BigEndian => u32::from_be_bytes(flags),
        }
    }
    /// Return all the data banks as a slice of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::events::EventView;
    ///
    /// let header = [
    ///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .collect();
    /// let event = EventView::try_from_be_bytes(&event).unwrap();
    ///
    /// assert_eq!([66, 65, 78, 75, 0, 1, 0, 1, 255, 0, 0, 0, 0, 0, 0, 0], event.all_banks_slice());
    /// ```
    pub fn all_banks_slice(&self) -> &[u8] {
        let offset = EVENT_ID_LENGTH
            + EVENT_TRIGGER_MASK_LENGTH
            + EVENT_SERIAL_NUMBER_LENGTH
            + EVENT_TIME_STAMP_LENGTH
            + EVENT_SIZE_LENGTH
            + EVENT_ALL_BANKS_SIZE_LENGTH
            + EVENT_FLAGS_LENGTH;
        &self.slice[offset..]
    }
}

impl<'a> IntoIterator for &'a EventView<'a> {
    type Item = BankView<'a>;
    type IntoIter = BankViews<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let flags = self.flags();
        let bank_type = BankType::try_from(flags).unwrap();
        match bank_type {
            BankType::B16 => match self.endianness {
                Endianness::LittleEndian => {
                    BankViews::B16(Bank16Views::from_le_bytes(self.all_banks_slice()))
                }
                Endianness::BigEndian => {
                    BankViews::B16(Bank16Views::from_be_bytes(self.all_banks_slice()))
                }
            },
            BankType::B32 => match self.endianness {
                Endianness::LittleEndian => {
                    BankViews::B32(Bank32Views::from_le_bytes(self.all_banks_slice()))
                }
                Endianness::BigEndian => {
                    BankViews::B32(Bank32Views::from_be_bytes(self.all_banks_slice()))
                }
            },
            BankType::B32A => match self.endianness {
                Endianness::LittleEndian => {
                    BankViews::B32A(Bank32AViews::from_le_bytes(self.all_banks_slice()))
                }
                Endianness::BigEndian => {
                    BankViews::B32A(Bank32AViews::from_be_bytes(self.all_banks_slice()))
                }
            },
        }
    }
}

// A slice of bytes cannot be a MIDAS event under the following errors:
// 1) Too short for even the header.
// 2) The event size and all banks size do not match.
// 3) The size field and the length of the slice do not match.
// 4) The flags field is an unknown integer.
// 5) The all_banks_slice is not exactly a number of data banks plus their padding. There shouldn't
//    be any remainder.
fn error_in_event_view(event: &EventView) -> Option<TryEventViewFromSliceError> {
    let header_length = EVENT_ID_LENGTH
        + EVENT_TRIGGER_MASK_LENGTH
        + EVENT_SERIAL_NUMBER_LENGTH
        + EVENT_TIME_STAMP_LENGTH
        + EVENT_SIZE_LENGTH
        + EVENT_ALL_BANKS_SIZE_LENGTH
        + EVENT_FLAGS_LENGTH;
    if event.slice.len() < header_length {
        return Some(TryEventViewFromSliceError::SizeMismatch);
    }

    let offset = EVENT_ID_LENGTH
        + EVENT_TRIGGER_MASK_LENGTH
        + EVENT_SERIAL_NUMBER_LENGTH
        + EVENT_TIME_STAMP_LENGTH;
    let event_size = event.slice[offset..][..EVENT_SIZE_LENGTH]
        .try_into()
        .unwrap();
    let event_size = match event.endianness {
        Endianness::LittleEndian => u32::from_le_bytes(event_size),
        Endianness::BigEndian => u32::from_be_bytes(event_size),
    };

    let offset = offset + EVENT_SIZE_LENGTH;
    let all_banks_size = event.slice[offset..][..EVENT_ALL_BANKS_SIZE_LENGTH]
        .try_into()
        .unwrap();
    let all_banks_size = match event.endianness {
        Endianness::LittleEndian => u32::from_le_bytes(all_banks_size),
        Endianness::BigEndian => u32::from_be_bytes(all_banks_size),
    };

    if usize::try_from(event_size).unwrap() - EVENT_ALL_BANKS_SIZE_LENGTH - EVENT_FLAGS_LENGTH
        != all_banks_size.try_into().unwrap()
    {
        return Some(TryEventViewFromSliceError::EventAndBanksMismatch);
    }

    if event.slice.len() - header_length != all_banks_size.try_into().unwrap() {
        return Some(TryEventViewFromSliceError::SizeMismatch);
    }

    let offset = offset + EVENT_ALL_BANKS_SIZE_LENGTH;
    let flags = event.slice[offset..][..EVENT_FLAGS_LENGTH]
        .try_into()
        .unwrap();
    let flags = match event.endianness {
        Endianness::LittleEndian => u32::from_le_bytes(flags),
        Endianness::BigEndian => u32::from_be_bytes(flags),
    };
    let bank_type = match BankType::try_from(flags) {
        Ok(bank_type) => bank_type,
        Err(_) => return Some(TryEventViewFromSliceError::UnknownFlag),
    };

    let mut all_banks = match bank_type {
        BankType::B16 => match event.endianness {
            Endianness::LittleEndian => {
                BankViews::B16(Bank16Views::from_le_bytes(&event.slice[header_length..]))
            }
            Endianness::BigEndian => {
                BankViews::B16(Bank16Views::from_be_bytes(&event.slice[header_length..]))
            }
        },
        BankType::B32 => match event.endianness {
            Endianness::LittleEndian => {
                BankViews::B32(Bank32Views::from_le_bytes(&event.slice[header_length..]))
            }
            Endianness::BigEndian => {
                BankViews::B32(Bank32Views::from_be_bytes(&event.slice[header_length..]))
            }
        },
        BankType::B32A => match event.endianness {
            Endianness::LittleEndian => {
                BankViews::B32A(Bank32AViews::from_le_bytes(&event.slice[header_length..]))
            }
            Endianness::BigEndian => {
                BankViews::B32A(Bank32AViews::from_be_bytes(&event.slice[header_length..]))
            }
        },
    };
    for _ in all_banks.by_ref() {}
    if !all_banks.remainder().is_empty() {
        return Some(TryEventViewFromSliceError::BadBank);
    }
    None
}

#[cfg(test)]
mod tests;
