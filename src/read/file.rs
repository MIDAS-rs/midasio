use crate::read::event::EventView;
use crate::{Endianness, EVENT_HEADER_LENGTH};
use crate::{
    BOR_ID, EOR_ID, ODB_HEADER_LENGTH, ODB_ID_LENGTH, ODB_MI, ODB_MI_LENGTH, ODB_RUN_NUMBER_LENGTH,
    ODB_SIZE_LENGTH, ODB_TIME_STAMP_LENGTH,
};
use crate::{
    EVENT_ALL_BANKS_SIZE_LENGTH, EVENT_ID_LENGTH, EVENT_SERIAL_NUMBER_LENGTH, EVENT_SIZE_LENGTH,
    EVENT_TIME_STAMP_LENGTH, EVENT_TRIGGER_MASK_LENGTH,
};
use std::{error::Error, fmt};

/// An iterator over a slice in (non-overlapping) [`EventView`]s, starting at the beginning of the
/// slice.
///
/// When the slice len is not evenly divided by an event, the last slice can always be obtained
/// with the [`EventViews::remainder()`].
///
/// # Examples
///
/// ```
/// use midasio::read::file::EventViews;
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
///
/// let event_views = EventViews::from_le_bytes(&event);
/// assert_eq!(1, event_views.count());
/// ```
#[derive(Clone, Debug)]
pub struct EventViews<'a> {
    curr: usize,
    slice: &'a [u8],
    endianness: Endianness,
}
impl<'a> EventViews<'a> {
    /// Create an iterator over a slice where the underlying [`EventView`]s are little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::file::EventViews;
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
    ///
    /// let event_views = EventViews::from_le_bytes(&event);
    /// ```
    pub fn from_le_bytes(buffer: &'a [u8]) -> Self {
        EventViews {
            curr: 0,
            slice: buffer,
            endianness: Endianness::LittleEndian,
        }
    }
    /// Create an iterator over a slice where the underlying [`EventView`]s are big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::read::file::EventViews;
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
    ///
    /// let event_views = EventViews::from_be_bytes(&event);
    /// ```
    pub fn from_be_bytes(buffer: &'a [u8]) -> Self {
        EventViews {
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
    /// use midasio::read::file::EventViews;
    ///
    /// let header = [
    ///     0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    /// let padding = [0u8, 0, 0, 0, 0, 0, 0];
    /// let extra = [1u8, 2, 3, 4, 5, 6];
    ///
    /// let event: Vec<u8> = header
    ///     .into_iter()
    ///     .chain(bank.into_iter())
    ///     .chain(padding.into_iter())
    ///     .chain(extra.into_iter())
    ///     .collect();
    ///
    /// let mut event_views = EventViews::from_be_bytes(&event);
    /// assert_eq!(1, event_views.next().unwrap().id());
    /// assert_eq!([1, 2, 3, 4, 5, 6], event_views.remainder());
    /// ```
    pub fn remainder(&self) -> &'a [u8] {
        &self.slice[self.curr..]
    }
}
impl<'a> Iterator for EventViews<'a> {
    type Item = EventView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.len() - self.curr < EVENT_HEADER_LENGTH {
            return None;
        }

        let offset = EVENT_ID_LENGTH
            + EVENT_TRIGGER_MASK_LENGTH
            + EVENT_SERIAL_NUMBER_LENGTH
            + EVENT_TIME_STAMP_LENGTH
            + EVENT_SIZE_LENGTH;
        let size = self.slice[self.curr + offset..][..EVENT_ALL_BANKS_SIZE_LENGTH]
            .try_into()
            .unwrap();
        let size: usize = match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(size).try_into().unwrap(),
            Endianness::BigEndian => u32::from_be_bytes(size).try_into().unwrap(),
        };
        if self.slice.len() - self.curr < size + EVENT_HEADER_LENGTH {
            return None;
        }

        let event = &self.slice[self.curr..][..EVENT_HEADER_LENGTH + size];
        let event = match self.endianness {
            Endianness::LittleEndian => EventView::try_from_le_bytes(event),
            Endianness::BigEndian => EventView::try_from_be_bytes(event),
        };
        match event {
            Ok(event) => {
                self.curr += EVENT_HEADER_LENGTH + size;
                Some(event)
            }
            Err(_) => None,
        }
    }
}

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to a [`FileView`] fails.
#[derive(Clone, Copy, Debug)]
pub enum TryFileViewFromSliceError {
    /// The first two bytes of the slice don't match the expected begin-of-run ID.
    BadBorId,
    /// The 3rd and 4th bytes of the slice don't match the expected begin-of-run "MIDAS_MAGIC".
    BadBorMi,
    /// The slice is shorter than the ODB header length, or the slice is shorter than the header
    /// plus the expected size.
    IniOdbSizeMismatch,
    /// The first two bytes, after all the identified events, don't match the expected end-of-run ID. This error
    /// could indicate that there is an error parsing an event, hence the final ODB dump was
    /// searched before it should have been.
    BadEorId,
    /// The 3rd and 4th bytes, after all the identified events, don't match the expected end-of-run
    /// "MIDAS_MAGIC".
    BadEorMi,
    /// The slice finishes before the expected length by the final ODB dump.
    FinOdbSizeMismatch,
    /// The run number from the initial ODB dump doesn't match the run number from the final ODB
    /// dump.
    RunNumberMismatch,
}
impl fmt::Display for TryFileViewFromSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryFileViewFromSliceError::BadBorId => {
                write!(f, "begin of run id different to 0x8000")
            }
            TryFileViewFromSliceError::BadBorMi => {
                write!(f, "begin of run magic midas different to 0x494d")
            }
            TryFileViewFromSliceError::IniOdbSizeMismatch => {
                write!(f, "initial odb size mismatch")
            }
            TryFileViewFromSliceError::BadEorId => {
                write!(f, "end of run id different to 0x8001")
            }
            TryFileViewFromSliceError::BadEorMi => {
                write!(f, "end of run magic midas different to 0x494d")
            }
            TryFileViewFromSliceError::FinOdbSizeMismatch => {
                write!(f, "final odb size mismatch")
            }
            TryFileViewFromSliceError::RunNumberMismatch => {
                write!(f, "run number mismatch")
            }
        }
    }
}
impl Error for TryFileViewFromSliceError {}

/// An immutable view to a MIDAS file.
///
/// A MIDAS file is a list of [`EventView`]s wrapped by a dump of the Online Data Base (ODB) at the
/// beginning and the end of a run:
/// - Initial ODB dump.
/// - Aritrary number of [`EventView`]s.
/// - Final ODB dump.
///
/// # Examples
///
/// ```no_run
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use std::fs;
/// use midasio::read::file::FileView;
///
/// let contents = fs::read("example.mid")?;
///
/// let file_view = FileView::try_from(&contents[..])?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct FileView<'a> {
    slice: &'a [u8],
    // Endianness is detected from the BOR id i.e. the first 2 bytes.
    endianness: Endianness,
}
impl<'a> FileView<'a> {
    /// Return the run number associated with a MIDAS file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use std::fs;
    /// use midasio::read::file::FileView;
    ///
    /// let contents = fs::read("example.mid")?;
    /// let file_view = FileView::try_from(&contents[..])?;
    ///
    /// let run_number: u32 = file_view.run_number();
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_number(&self) -> u32 {
        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH;
        let run_number = self.slice[offset..][..ODB_RUN_NUMBER_LENGTH]
            .try_into()
            .unwrap();
        match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(run_number),
            Endianness::BigEndian => u32::from_be_bytes(run_number),
        }
    }
    /// Return the timestamp of the initial ODB dump.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use std::fs;
    /// use midasio::read::file::FileView;
    ///
    /// let contents = fs::read("example.mid")?;
    /// let file_view = FileView::try_from(&contents[..])?;
    ///
    /// let timestamp: u32 = file_view.initial_timestamp();
    /// # Ok(())
    /// # }
    /// ```
    pub fn initial_timestamp(&self) -> u32 {
        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH + ODB_RUN_NUMBER_LENGTH;
        let timestamp = self.slice[offset..][..ODB_TIME_STAMP_LENGTH]
            .try_into()
            .unwrap();
        match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(timestamp),
            Endianness::BigEndian => u32::from_be_bytes(timestamp),
        }
    }
    /// Return the initial ODB dump. This is not guaranteed to be valid ASCII alphanumeric nor
    /// valid UTF8.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use std::fs;
    /// use midasio::read::file::FileView;
    ///
    /// let contents = fs::read("example.mid")?;
    /// let file_view = FileView::try_from(&contents[..])?;
    ///
    /// let odb_dump = file_view.initial_odb();
    /// # Ok(())
    /// # }
    /// ```
    pub fn initial_odb(&self) -> &[u8] {
        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH + ODB_RUN_NUMBER_LENGTH + ODB_TIME_STAMP_LENGTH;
        let size = self.slice[offset..][..ODB_SIZE_LENGTH].try_into().unwrap();
        let size: usize = match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(size).try_into().unwrap(),
            Endianness::BigEndian => u32::from_be_bytes(size).try_into().unwrap(),
        };
        &self.slice[ODB_HEADER_LENGTH..][..size]
    }
    /// Return the timestamp of the final ODB dump.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use std::fs;
    /// use midasio::read::file::FileView;
    ///
    /// let contents = fs::read("example.mid")?;
    /// let file_view = FileView::try_from(&contents[..])?;
    ///
    /// let timestamp: u32 = file_view.final_timestamp();
    /// # Ok(())
    /// # }
    /// ```
    pub fn final_timestamp(&self) -> u32 {
        let mut events = self.into_iter();
        for _ in events.by_ref() {}

        let footer = events.remainder();
        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH + ODB_RUN_NUMBER_LENGTH;
        let timestamp = footer[offset..][..ODB_TIME_STAMP_LENGTH]
            .try_into()
            .unwrap();
        match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(timestamp),
            Endianness::BigEndian => u32::from_be_bytes(timestamp),
        }
    }
    /// Return the final ODB dump. This is not guaranteed to be valid ASCII alphanumeric nor
    /// valid UTF8.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use std::fs;
    /// use midasio::read::file::FileView;
    ///
    /// let contents = fs::read("example.mid")?;
    /// let file_view = FileView::try_from(&contents[..])?;
    ///
    /// let odb_dump = file_view.final_odb();
    /// # Ok(())
    /// # }
    /// ```
    pub fn final_odb(&self) -> &[u8] {
        let mut events = self.into_iter();
        for _ in events.by_ref() {}

        let footer = events.remainder();
        &footer[ODB_HEADER_LENGTH..]
    }
}
impl<'a> TryFrom<&'a [u8]> for FileView<'a> {
    type Error = TryFileViewFromSliceError;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < ODB_HEADER_LENGTH {
            return Err(TryFileViewFromSliceError::IniOdbSizeMismatch);
        }

        let endianness: Endianness;
        let odb_id = buffer[..ODB_ID_LENGTH].try_into().unwrap();
        let odb_id = u16::from_le_bytes(odb_id);
        if odb_id == BOR_ID {
            endianness = Endianness::LittleEndian;
        } else if odb_id.swap_bytes() == BOR_ID {
            endianness = Endianness::BigEndian;
        } else {
            return Err(TryFileViewFromSliceError::BadBorId);
        }

        let odb_mi = buffer[ODB_ID_LENGTH..][..ODB_MI_LENGTH].try_into().unwrap();
        let odb_mi = match endianness {
            Endianness::LittleEndian => u16::from_le_bytes(odb_mi),
            Endianness::BigEndian => u16::from_be_bytes(odb_mi),
        };
        if odb_mi != ODB_MI {
            return Err(TryFileViewFromSliceError::BadBorMi);
        }

        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH;
        let init_run_num = buffer[offset..][..ODB_RUN_NUMBER_LENGTH]
            .try_into()
            .unwrap();
        let init_run_num = match endianness {
            Endianness::LittleEndian => u32::from_le_bytes(init_run_num),
            Endianness::BigEndian => u32::from_be_bytes(init_run_num),
        };

        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH + ODB_RUN_NUMBER_LENGTH + ODB_TIME_STAMP_LENGTH;
        let size = buffer[offset..][..ODB_SIZE_LENGTH].try_into().unwrap();
        let size: usize = match endianness {
            Endianness::LittleEndian => u32::from_le_bytes(size).try_into().unwrap(),
            Endianness::BigEndian => u32::from_be_bytes(size).try_into().unwrap(),
        };
        if buffer.len() < ODB_HEADER_LENGTH + size {
            return Err(TryFileViewFromSliceError::IniOdbSizeMismatch);
        }

        let events = &buffer[ODB_HEADER_LENGTH + size..];
        let mut events = match endianness {
            Endianness::LittleEndian => EventViews::from_le_bytes(events),
            Endianness::BigEndian => EventViews::from_be_bytes(events),
        };
        for _ in events.by_ref() {}

        let footer = events.remainder();
        if footer.len() < ODB_HEADER_LENGTH {
            return Err(TryFileViewFromSliceError::FinOdbSizeMismatch);
        }

        let odb_id = footer[..ODB_ID_LENGTH].try_into().unwrap();
        let odb_id = match endianness {
            Endianness::LittleEndian => u16::from_le_bytes(odb_id),
            Endianness::BigEndian => u16::from_be_bytes(odb_id),
        };
        if odb_id != EOR_ID {
            return Err(TryFileViewFromSliceError::BadEorId);
        }

        let odb_mi = footer[ODB_ID_LENGTH..][..ODB_MI_LENGTH].try_into().unwrap();
        let odb_mi = match endianness {
            Endianness::LittleEndian => u16::from_le_bytes(odb_mi),
            Endianness::BigEndian => u16::from_be_bytes(odb_mi),
        };
        if odb_mi != ODB_MI {
            return Err(TryFileViewFromSliceError::BadEorMi);
        }

        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH;
        let final_run_num = footer[offset..][..ODB_RUN_NUMBER_LENGTH]
            .try_into()
            .unwrap();
        let final_run_num = match endianness {
            Endianness::LittleEndian => u32::from_le_bytes(final_run_num),
            Endianness::BigEndian => u32::from_be_bytes(final_run_num),
        };
        if final_run_num != init_run_num {
            return Err(TryFileViewFromSliceError::RunNumberMismatch);
        }

        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH + ODB_RUN_NUMBER_LENGTH + ODB_TIME_STAMP_LENGTH;
        let size = footer[offset..][..ODB_SIZE_LENGTH].try_into().unwrap();
        let size: usize = match endianness {
            Endianness::LittleEndian => u32::from_le_bytes(size).try_into().unwrap(),
            Endianness::BigEndian => u32::from_be_bytes(size).try_into().unwrap(),
        };
        if footer.len() != ODB_HEADER_LENGTH + size {
            return Err(TryFileViewFromSliceError::FinOdbSizeMismatch);
        }

        Ok(FileView {
            slice: buffer,
            endianness,
        })
    }
}
impl<'a> IntoIterator for &'_ FileView<'a> {
    type Item = EventView<'a>;
    type IntoIter = EventViews<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let offset = ODB_ID_LENGTH + ODB_MI_LENGTH + ODB_RUN_NUMBER_LENGTH + ODB_TIME_STAMP_LENGTH;
        let size = self.slice[offset..][..ODB_SIZE_LENGTH].try_into().unwrap();
        let size: usize = match self.endianness {
            Endianness::LittleEndian => u32::from_le_bytes(size).try_into().unwrap(),
            Endianness::BigEndian => u32::from_be_bytes(size).try_into().unwrap(),
        };

        let events = &self.slice[ODB_HEADER_LENGTH + size..];
        match self.endianness {
            Endianness::LittleEndian => EventViews::from_le_bytes(events),
            Endianness::BigEndian => EventViews::from_be_bytes(events),
        }
    }
}

#[cfg(test)]
mod tests;
