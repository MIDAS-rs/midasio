#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(feature = "rayon")]
use rayon::iter::IntoParallelRefIterator;

/// The error type returned when parsing a MIDAS file fails.
#[derive(Debug)]
pub struct ParseError {
    offset: usize,
    inner: winnow::error::ContextError,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsing stopped at byte offset `{}`", self.offset)?;
        if self.inner.context().next().is_some() {
            write!(f, " ({})", self.inner)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner
            .cause()
            .map(|v| v as &(dyn std::error::Error + 'static))
    }
}

/// Possible data types stored inside a data bank.
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
    /// String.
    Str,
    /// Array with unknown contents.
    Array,
    /// User-defined structure.
    Struct,
    /// Signed 64-bits integer.
    I64,
    /// Unsigned 64-bits integer.
    U64,
}

/// An immutable view to a data bank in a MIDAS file.
#[derive(Clone, Copy, Debug)]
pub struct BankView<'a> {
    name: [u8; 4],
    data_type: DataType,
    data: &'a [u8],
}

impl<'a> BankView<'a> {
    /// Returns the name of the data bank.
    pub fn name(&self) -> [u8; 4] {
        self.name
    }
    /// Returns the data type of the data bank.
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    /// Returns the data stored in the data bank.
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

/// An immutable view to an event in a MIDAS file.
///
/// An event is a collection of [`BankView`]s.
#[derive(Clone, Debug)]
pub struct EventView<'a> {
    id: u16,
    trigger_mask: u16,
    serial_number: u32,
    timestamp: u32,
    bank_views: Box<[BankView<'a>]>,
}

impl<'a> EventView<'a> {
    /// Returns the event ID.
    pub fn id(&self) -> u16 {
        self.id
    }
    /// Returns the trigger mask of the event.
    pub fn trigger_mask(&self) -> u16 {
        self.trigger_mask
    }
    /// Returns the serial number of the event.
    pub fn serial_number(&self) -> u32 {
        self.serial_number
    }
    /// Returns the unix timestamp of the event.
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
    /// Returns an iterator over the data banks of the event.
    pub fn iter(&self) -> std::slice::Iter<'_, BankView<'a>> {
        self.into_iter()
    }
}

impl<'a, 'b> IntoIterator for &'b EventView<'a> {
    type Item = &'b BankView<'a>;
    type IntoIter = std::slice::Iter<'b, BankView<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.bank_views.iter()
    }
}

impl<'a> IntoIterator for EventView<'a> {
    type Item = BankView<'a>;
    type IntoIter = std::vec::IntoIter<BankView<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.bank_views.into_vec().into_iter()
    }
}

/// An immutable view to a MIDAS file.
///
/// A file is a collection of [`EventView`]s wrapped by two dumps of the Online
/// DataBase (ODB) at the beginning and end of the sub-run.
#[derive(Clone, Debug)]
pub struct FileView<'a> {
    run_number: u32,
    initial_timestamp: u32,
    initial_odb: &'a [u8],
    event_views: Box<[EventView<'a>]>,
    final_timestamp: u32,
    final_odb: &'a [u8],
}

impl<'a> FileView<'a> {
    /// Create a native view to the underlying file from its representation as a
    /// byte slice.
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, ParseError> {
        todo!()
    }
    /// Returns the run number of the file.
    pub fn run_number(&self) -> u32 {
        self.run_number
    }
    /// Returns the unix timestamp of the initial ODB dump.
    pub fn initial_timestamp(&self) -> u32 {
        self.initial_timestamp
    }
    /// Returns the initial ODB dump.
    pub fn initial_odb(&self) -> &'a [u8] {
        self.initial_odb
    }
    /// Returns the unix timestamp of the final ODB dump.
    pub fn final_timestamp(&self) -> u32 {
        self.final_timestamp
    }
    /// Returns the final ODB dump.
    pub fn final_odb(&self) -> &'a [u8] {
        self.final_odb
    }
    /// Returns an iterator over the events of the file.
    pub fn iter(&self) -> std::slice::Iter<'_, EventView<'a>> {
        self.into_iter()
    }
}

impl<'a, 'b> IntoIterator for &'b FileView<'a> {
    type Item = &'b EventView<'a>;
    type IntoIter = std::slice::Iter<'b, EventView<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.event_views.iter()
    }
}

impl<'a> IntoIterator for FileView<'a> {
    type Item = EventView<'a>;
    type IntoIter = std::vec::IntoIter<EventView<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.event_views.into_vec().into_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a> rayon::iter::IntoParallelIterator for FileView<'a> {
    type Item = EventView<'a>;
    type Iter = rayon::vec::IntoIter<EventView<'a>>;

    fn into_par_iter(self) -> Self::Iter {
        self.event_views.into_vec().into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, 'b> rayon::iter::IntoParallelIterator for &'b FileView<'a> {
    type Item = &'b EventView<'a>;
    type Iter = rayon::slice::Iter<'b, EventView<'a>>;

    fn into_par_iter(self) -> Self::Iter {
        self.event_views.par_iter()
    }
}

/// Returns the run number assuming that the input slice has the correct MIDAS
/// file format.
///
/// This is useful for checking the run number of a file without having to parse
/// its entire contents. Returns an error if the run number cannot be
/// determined.
///
/// # Examples
///
/// ```
/// // Note that the following is an invalid MIDAS file:
/// // - The magic midas marker is 0xFFFF instead of 0x494D.
/// // - Too short to even contain the rest of the header.
/// let bytes = b"\x00\x80\xFF\xFF\x01\x00\x00\x00";
///
/// // Nonetheless, a "run number" can still be extracted with this function.
/// let run_number = midasio::run_number_unchecked(bytes)?;
/// assert_eq!(run_number, 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn run_number_unchecked(bytes: &[u8]) -> Result<u32, ParseError> {
    todo!()
}

/// Returns the timestamp of the initial ODB dump assuming the correct MIDAS
/// file format.
///
/// This is useful for checking the initial timestamp of a file without having
/// to parse its entire contents. Returns an error if the timestamp cannot be
/// determined.
///
/// # Examples
///
/// ```
/// // Note that the following is an invalid MIDAS file:
/// // - The magic midas marker is 0xFFFF instead of 0x494D.
/// // - Too short to even contain the rest of the header.
/// let bytes = b"\x00\x80\xFF\xFF\xFF\xFF\xFF\xFF\x01\x00\x00\x00";
///
/// // Nonetheless, an "initial timestamp" can still be extracted with this function.
/// let timestamp = midasio::initial_timestamp_unchecked(bytes)?;
/// assert_eq!(timestamp, 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn initial_timestamp_unchecked(bytes: &[u8]) -> Result<u32, ParseError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::repeat;

    fn bank_16(name: [u8; 4], data_type: u16, data: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut bytes_le = Vec::new();
        bytes_le.extend(&name);
        bytes_le.extend(data_type.to_le_bytes());
        bytes_le.extend((data.len() as u16).to_le_bytes());
        bytes_le.extend(data);
        bytes_le.extend(repeat(0).take(data.len().next_multiple_of(8) - data.len()));

        let mut bytes_be = bytes_le.clone();
        bytes_be[4..6].copy_from_slice(&data_type.to_be_bytes());
        bytes_be[6..8].copy_from_slice(&(data.len() as u16).to_be_bytes());

        (bytes_le, bytes_be)
    }

    fn bank_32(name: [u8; 4], data_type: u32, data: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut bytes_le = Vec::new();
        bytes_le.extend(&name);
        bytes_le.extend(data_type.to_le_bytes());
        bytes_le.extend((data.len() as u32).to_le_bytes());
        bytes_le.extend(data);
        bytes_le.extend(repeat(0).take(data.len().next_multiple_of(8) - data.len()));

        let mut bytes_be = bytes_le.clone();
        bytes_be[4..8].copy_from_slice(&data_type.to_be_bytes());
        bytes_be[8..12].copy_from_slice(&(data.len() as u32).to_be_bytes());

        (bytes_le, bytes_be)
    }

    fn bank_32a(name: [u8; 4], data_type: u32, data: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut bytes_le = Vec::new();
        bytes_le.extend(&name);
        bytes_le.extend(data_type.to_le_bytes());
        bytes_le.extend((data.len() as u32).to_le_bytes());
        bytes_le.extend(repeat(0).take(4));
        bytes_le.extend(data);
        bytes_le.extend(repeat(0).take(data.len().next_multiple_of(8) - data.len()));

        let mut bytes_be = bytes_le.clone();
        bytes_be[4..8].copy_from_slice(&data_type.to_be_bytes());
        bytes_be[8..12].copy_from_slice(&(data.len() as u32).to_be_bytes());

        (bytes_le, bytes_be)
    }

    fn event(
        id: u16,
        trigger_mask: u16,
        serial_number: u32,
        timestamp: u32,
        flags: u32,
        banks: &[u8],
    ) -> (Vec<u8>, Vec<u8>) {
        let mut bytes_le = Vec::new();
        bytes_le.extend(id.to_le_bytes());
        bytes_le.extend(trigger_mask.to_le_bytes());
        bytes_le.extend(serial_number.to_le_bytes());
        bytes_le.extend(timestamp.to_le_bytes());
        bytes_le.extend(((banks.len() as u32).checked_add(8).unwrap()).to_le_bytes());
        bytes_le.extend((banks.len() as u32).to_le_bytes());
        bytes_le.extend(flags.to_le_bytes());
        bytes_le.extend(banks);

        let mut bytes_be = Vec::new();
        bytes_be.extend(id.to_be_bytes());
        bytes_be.extend(trigger_mask.to_be_bytes());
        bytes_be.extend(serial_number.to_be_bytes());
        bytes_be.extend(timestamp.to_be_bytes());
        bytes_be.extend(((banks.len() as u32).checked_add(8).unwrap()).to_be_bytes());
        bytes_be.extend((banks.len() as u32).to_be_bytes());
        bytes_be.extend(flags.to_be_bytes());
        bytes_be.extend(banks);

        (bytes_le, bytes_be)
    }

    fn file(
        run_number: u32,
        initial_timestamp: u32,
        initial_odb: &[u8],
        events: &[u8],
        final_timestamp: u32,
        final_odb: &[u8],
    ) -> (Vec<u8>, Vec<u8>) {
        let bor_id: u16 = 0x8000;
        let magic: u16 = 0x494D;
        let eor_id: u16 = 0x8001;

        let mut bytes_le = Vec::new();
        bytes_le.extend(bor_id.to_le_bytes());
        bytes_le.extend(magic.to_le_bytes());
        bytes_le.extend(run_number.to_le_bytes());
        bytes_le.extend(initial_timestamp.to_le_bytes());
        bytes_le.extend((initial_odb.len() as u32).to_le_bytes());
        bytes_le.extend(initial_odb);
        bytes_le.extend(events);
        bytes_le.extend(eor_id.to_le_bytes());
        bytes_le.extend(magic.to_le_bytes());
        bytes_le.extend(run_number.to_le_bytes());
        bytes_le.extend(final_timestamp.to_le_bytes());
        bytes_le.extend((final_odb.len() as u32).to_le_bytes());
        bytes_le.extend(final_odb);

        let mut bytes_be = Vec::new();
        bytes_be.extend(bor_id.to_be_bytes());
        bytes_be.extend(magic.to_be_bytes());
        bytes_be.extend(run_number.to_be_bytes());
        bytes_be.extend(initial_timestamp.to_be_bytes());
        bytes_be.extend((initial_odb.len() as u32).to_be_bytes());
        bytes_be.extend(initial_odb);
        bytes_be.extend(events);
        bytes_be.extend(eor_id.to_be_bytes());
        bytes_be.extend(magic.to_be_bytes());
        bytes_be.extend(run_number.to_be_bytes());
        bytes_be.extend(final_timestamp.to_be_bytes());
        bytes_be.extend((final_odb.len() as u32).to_be_bytes());
        bytes_be.extend(final_odb);

        (bytes_le, bytes_be)
    }
}
