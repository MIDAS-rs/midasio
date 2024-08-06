#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

use winnow::binary::u32;
use winnow::combinator::{delimited, rest};
use winnow::error::{ContextError, PResult, StrContext};
use winnow::token::take;
use winnow::Parser;

#[cfg(feature = "rayon")]
use rayon::iter::IntoParallelRefIterator;

mod parse;

/// The error type returned when parsing a MIDAS file fails.
#[derive(Debug)]
pub struct ParseError {
    offset: usize,
    inner: ContextError,
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
#[non_exhaustive]
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
        parse::file_view.parse(bytes).map_err(|e| ParseError {
            offset: e.offset(),
            inner: e.into_inner(),
        })
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
    fn run_number(input: &mut &[u8]) -> PResult<u32> {
        let endianness = parse::endianness
            .context(StrContext::Label("begin-of-run id"))
            .parse_next(input)?;
        delimited(
            take(2usize).context(StrContext::Label("magic marker")),
            u32(endianness).context(StrContext::Label("run number")),
            rest,
        )
        .parse_next(input)
    }

    run_number.parse(bytes).map_err(|e| ParseError {
        offset: e.offset(),
        inner: e.into_inner(),
    })
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
    fn initial_timestamp(input: &mut &[u8]) -> PResult<u32> {
        let endianness = parse::endianness
            .context(StrContext::Label("begin-of-run id"))
            .parse_next(input)?;
        delimited(
            take(6usize).context(StrContext::Label("magic marker and run number")),
            u32(endianness).context(StrContext::Label("initial timestamp")),
            rest,
        )
        .parse_next(input)
    }

    initial_timestamp.parse(bytes).map_err(|e| ParseError {
        offset: e.offset(),
        inner: e.into_inner(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::repeat;

    const BOR_ID: u16 = 0x8000;
    const EOR_ID: u16 = 0x8001;
    const MAGIC: u16 = 0x494D;

    const INT_DATA_TYPES: [(u16, DataType); 18] = [
        (1, DataType::U8),
        (2, DataType::I8),
        (3, DataType::U8),
        (4, DataType::U16),
        (5, DataType::I16),
        (6, DataType::U32),
        (7, DataType::I32),
        (8, DataType::Bool),
        (9, DataType::F32),
        (10, DataType::F64),
        (11, DataType::U32),
        (12, DataType::Str),
        (13, DataType::Array),
        (14, DataType::Struct),
        (15, DataType::Str),
        (16, DataType::Str),
        (17, DataType::I64),
        (18, DataType::U64),
    ];

    fn bank_16_le(name: [u8; 4], data_type: u16, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 8 + data.len().next_multiple_of(8)];
        bytes[..4].copy_from_slice(&name);
        bytes[4..6].copy_from_slice(&data_type.to_le_bytes());
        bytes[6..8].copy_from_slice(&(data.len() as u16).to_le_bytes());
        bytes[8..][..data.len()].copy_from_slice(data);
        bytes
    }

    fn bank_16_be(name: [u8; 4], data_type: u16, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 8 + data.len().next_multiple_of(8)];
        bytes[..4].copy_from_slice(&name);
        bytes[4..6].copy_from_slice(&data_type.to_be_bytes());
        bytes[6..8].copy_from_slice(&(data.len() as u16).to_be_bytes());
        bytes[8..][..data.len()].copy_from_slice(data);
        bytes
    }

    fn bank_32_le(name: [u8; 4], data_type: u32, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 12 + data.len().next_multiple_of(8)];
        bytes[..4].copy_from_slice(&name);
        bytes[4..8].copy_from_slice(&data_type.to_le_bytes());
        bytes[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
        bytes[12..][..data.len()].copy_from_slice(data);
        bytes
    }

    fn bank_32_be(name: [u8; 4], data_type: u32, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 12 + data.len().next_multiple_of(8)];
        bytes[..4].copy_from_slice(&name);
        bytes[4..8].copy_from_slice(&data_type.to_be_bytes());
        bytes[8..12].copy_from_slice(&(data.len() as u32).to_be_bytes());
        bytes[12..][..data.len()].copy_from_slice(data);
        bytes
    }

    fn bank_32a_le(name: [u8; 4], data_type: u32, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 16 + data.len().next_multiple_of(8)];
        bytes[..4].copy_from_slice(&name);
        bytes[4..8].copy_from_slice(&data_type.to_le_bytes());
        bytes[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
        bytes[16..][..data.len()].copy_from_slice(data);
        bytes
    }

    fn bank_32a_be(name: [u8; 4], data_type: u32, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 16 + data.len().next_multiple_of(8)];
        bytes[..4].copy_from_slice(&name);
        bytes[4..8].copy_from_slice(&data_type.to_be_bytes());
        bytes[8..12].copy_from_slice(&(data.len() as u32).to_be_bytes());
        bytes[16..][..data.len()].copy_from_slice(data);
        bytes
    }

    fn event_le(
        id: u16,
        trigger_mask: u16,
        serial_number: u32,
        timestamp: u32,
        flags: u32,
        banks: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_le_bytes());
        bytes.extend(trigger_mask.to_le_bytes());
        bytes.extend(serial_number.to_le_bytes());
        bytes.extend(timestamp.to_le_bytes());
        bytes.extend((banks.len() as u32).checked_add(8).unwrap().to_le_bytes());
        bytes.extend((banks.len() as u32).to_le_bytes());
        bytes.extend(flags.to_le_bytes());
        bytes.extend(banks);
        bytes
    }

    fn event_be(
        id: u16,
        trigger_mask: u16,
        serial_number: u32,
        timestamp: u32,
        flags: u32,
        banks: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_be_bytes());
        bytes.extend(trigger_mask.to_be_bytes());
        bytes.extend(serial_number.to_be_bytes());
        bytes.extend(timestamp.to_be_bytes());
        bytes.extend((banks.len() as u32).checked_add(8).unwrap().to_be_bytes());
        bytes.extend((banks.len() as u32).to_be_bytes());
        bytes.extend(flags.to_be_bytes());
        bytes.extend(banks);
        bytes
    }

    fn file_le(
        run_number: u32,
        initial_timestamp: u32,
        initial_odb: &[u8],
        events: &[u8],
        final_timestamp: u32,
        final_odb: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(BOR_ID.to_le_bytes());
        bytes.extend(MAGIC.to_le_bytes());
        bytes.extend(run_number.to_le_bytes());
        bytes.extend(initial_timestamp.to_le_bytes());
        bytes.extend((initial_odb.len() as u32).to_le_bytes());
        bytes.extend(initial_odb);
        bytes.extend(events);
        bytes.extend(EOR_ID.to_le_bytes());
        bytes.extend(MAGIC.to_le_bytes());
        bytes.extend(run_number.to_le_bytes());
        bytes.extend(final_timestamp.to_le_bytes());
        bytes.extend((final_odb.len() as u32).to_le_bytes());
        bytes.extend(final_odb);
        bytes
    }

    fn file_be(
        run_number: u32,
        initial_timestamp: u32,
        initial_odb: &[u8],
        events: &[u8],
        final_timestamp: u32,
        final_odb: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(BOR_ID.to_be_bytes());
        bytes.extend(MAGIC.to_be_bytes());
        bytes.extend(run_number.to_be_bytes());
        bytes.extend(initial_timestamp.to_be_bytes());
        bytes.extend((initial_odb.len() as u32).to_be_bytes());
        bytes.extend(initial_odb);
        bytes.extend(events);
        bytes.extend(EOR_ID.to_be_bytes());
        bytes.extend(MAGIC.to_be_bytes());
        bytes.extend(run_number.to_be_bytes());
        bytes.extend(final_timestamp.to_be_bytes());
        bytes.extend((final_odb.len() as u32).to_be_bytes());
        bytes.extend(final_odb);
        bytes
    }

    #[test]
    fn file_view_try_from_le_bytes() {
        let mut events = Vec::new();

        let banks = repeat(bank_16_le([65; 4], 1, &[2; 100]))
            .take(10)
            .flatten()
            .collect::<Vec<_>>();
        events.extend(event_le(3, 4, 5, 6, 1, &banks));

        let banks = repeat(bank_32_le([65; 4], 1, &[2; 100]))
            .take(10)
            .flatten()
            .collect::<Vec<_>>();
        events.extend(event_le(3, 4, 5, 6, 17, &banks));

        let banks = repeat(bank_32a_le([65; 4], 1, &[2; 100]))
            .take(10)
            .flatten()
            .collect::<Vec<_>>();
        events.extend(event_le(3, 4, 5, 6, 49, &banks));

        let file = file_le(7, 8, b"initial odb", &events, 9, b"final odb");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        let mut event_count = 0;
        let mut bank_count = 0;
        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial odb");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final odb");
        for event_view in file_view {
            event_count += 1;
            assert_eq!(event_view.id(), 3);
            assert_eq!(event_view.trigger_mask(), 4);
            assert_eq!(event_view.serial_number(), 5);
            assert_eq!(event_view.timestamp(), 6);
            for bank_view in event_view {
                bank_count += 1;
                assert_eq!(bank_view.name(), [65; 4]);
                assert_eq!(bank_view.data_type(), DataType::U8);
                assert_eq!(bank_view.data(), &[2; 100]);
            }
        }
        assert_eq!(event_count, 3);
        assert_eq!(bank_count, 30);
    }

    #[test]
    fn file_view_try_from_be_bytes() {
        let mut events = Vec::new();

        let banks = repeat(bank_16_be([65; 4], 1, &[2; 100]))
            .take(10)
            .flatten()
            .collect::<Vec<_>>();
        events.extend(event_be(3, 4, 5, 6, 1, &banks));

        let banks = repeat(bank_32_be([65; 4], 1, &[2; 100]))
            .take(10)
            .flatten()
            .collect::<Vec<_>>();
        events.extend(event_be(3, 4, 5, 6, 17, &banks));

        let banks = repeat(bank_32a_be([65; 4], 1, &[2; 100]))
            .take(10)
            .flatten()
            .collect::<Vec<_>>();
        events.extend(event_be(3, 4, 5, 6, 49, &banks));

        let file = file_be(7, 8, b"initial odb", &events, 9, b"final odb");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        let mut event_count = 0;
        let mut bank_count = 0;
        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial odb");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final odb");
        for event_view in file_view {
            event_count += 1;
            assert_eq!(event_view.id(), 3);
            assert_eq!(event_view.trigger_mask(), 4);
            assert_eq!(event_view.serial_number(), 5);
            assert_eq!(event_view.timestamp(), 6);
            for bank_view in event_view {
                bank_count += 1;
                assert_eq!(bank_view.name(), [65; 4]);
                assert_eq!(bank_view.data_type(), DataType::U8);
                assert_eq!(bank_view.data(), &[2; 100]);
            }
        }
        assert_eq!(event_count, 3);
        assert_eq!(bank_count, 30);
    }

    #[test]
    fn file_view_empty_bank_16_le() {
        let bank = bank_16_le([65; 4], 1, &[]);
        let events = event_le(4, 5, 6, 7, 1, &bank);
        let file = file_le(1, 2, b"initial", &events, 3, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 4);
        assert_eq!(event_view.trigger_mask(), 5);
        assert_eq!(event_view.serial_number(), 6);
        assert_eq!(event_view.timestamp(), 7);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert!(bank_view.data().is_empty());
    }

    #[test]
    fn file_view_empty_bank_16_be() {
        let bank = bank_16_be([65; 4], 1, &[]);
        let events = event_be(4, 5, 6, 7, 1, &bank);
        let file = file_be(1, 2, b"initial", &events, 3, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 4);
        assert_eq!(event_view.trigger_mask(), 5);
        assert_eq!(event_view.serial_number(), 6);
        assert_eq!(event_view.timestamp(), 7);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert!(bank_view.data().is_empty());
    }

    #[test]
    fn file_view_empty_bank_32_le() {
        let bank = bank_32_le([65; 4], 1, &[]);
        let events = event_le(4, 5, 6, 7, 17, &bank);
        let file = file_le(1, 2, b"initial", &events, 3, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 4);
        assert_eq!(event_view.trigger_mask(), 5);
        assert_eq!(event_view.serial_number(), 6);
        assert_eq!(event_view.timestamp(), 7);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert!(bank_view.data().is_empty());
    }

    #[test]
    fn file_view_empty_bank_32_be() {
        let bank = bank_32_be([65; 4], 1, &[]);
        let events = event_be(4, 5, 6, 7, 17, &bank);
        let file = file_be(1, 2, b"initial", &events, 3, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 4);
        assert_eq!(event_view.trigger_mask(), 5);
        assert_eq!(event_view.serial_number(), 6);
        assert_eq!(event_view.timestamp(), 7);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert!(bank_view.data().is_empty());
    }

    #[test]
    fn file_view_empty_bank_32a_le() {
        let bank = bank_32a_le([65; 4], 1, &[]);
        let events = event_le(4, 5, 6, 7, 49, &bank);
        let file = file_le(1, 2, b"initial", &events, 3, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 4);
        assert_eq!(event_view.trigger_mask(), 5);
        assert_eq!(event_view.serial_number(), 6);
        assert_eq!(event_view.timestamp(), 7);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert!(bank_view.data().is_empty());
    }

    #[test]
    fn file_view_empty_bank_32a_be() {
        let bank = bank_32a_be([65; 4], 1, &[]);
        let events = event_be(4, 5, 6, 7, 49, &bank);
        let file = file_be(1, 2, b"initial", &events, 3, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 4);
        assert_eq!(event_view.trigger_mask(), 5);
        assert_eq!(event_view.serial_number(), 6);
        assert_eq!(event_view.timestamp(), 7);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert!(bank_view.data().is_empty());
    }

    #[test]
    fn file_view_empty_event_le() {
        for flags in [1, 17, 49] {
            let event = event_le(4, 5, 6, 7, flags, &[]);
            let file = file_le(1, 2, b"initial", &event, 3, b"final");
            let file_view = FileView::try_from_bytes(&file).unwrap();

            assert_eq!(file_view.run_number(), 1);
            assert_eq!(file_view.initial_timestamp(), 2);
            assert_eq!(file_view.initial_odb(), b"initial");
            assert_eq!(file_view.final_timestamp(), 3);
            assert_eq!(file_view.final_odb(), b"final");
            let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(event_view.id(), 4);
            assert_eq!(event_view.trigger_mask(), 5);
            assert_eq!(event_view.serial_number(), 6);
            assert_eq!(event_view.timestamp(), 7);
            assert_eq!(event_view.into_iter().count(), 0);
        }
    }

    #[test]
    fn file_view_empty_event_be() {
        for flags in [1, 17, 49] {
            let event = event_be(4, 5, 6, 7, flags, &[]);
            let file = file_be(1, 2, b"initial", &event, 3, b"final");
            let file_view = FileView::try_from_bytes(&file).unwrap();

            assert_eq!(file_view.run_number(), 1);
            assert_eq!(file_view.initial_timestamp(), 2);
            assert_eq!(file_view.initial_odb(), b"initial");
            assert_eq!(file_view.final_timestamp(), 3);
            assert_eq!(file_view.final_odb(), b"final");
            let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(event_view.id(), 4);
            assert_eq!(event_view.trigger_mask(), 5);
            assert_eq!(event_view.serial_number(), 6);
            assert_eq!(event_view.timestamp(), 7);
            assert_eq!(event_view.into_iter().count(), 0);
        }
    }

    #[test]
    fn file_view_no_events_le() {
        let file = file_le(1, 2, b"initial", &[], 3, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"final");
        assert_eq!(file_view.into_iter().count(), 0);
    }

    #[test]
    fn file_view_no_events_be() {
        let file = file_be(1, 2, b"initial", &[], 3, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"final");
        assert_eq!(file_view.into_iter().count(), 0);
    }

    #[test]
    fn file_view_empty_odb_le() {
        let file = file_le(1, 2, b"", &[], 3, b"");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"");
        assert_eq!(file_view.into_iter().count(), 0);
    }

    #[test]
    fn file_view_empty_odb_be() {
        let file = file_be(1, 2, b"", &[], 3, b"");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 1);
        assert_eq!(file_view.initial_timestamp(), 2);
        assert_eq!(file_view.initial_odb(), b"");
        assert_eq!(file_view.final_timestamp(), 3);
        assert_eq!(file_view.final_odb(), b"");
        assert_eq!(file_view.into_iter().count(), 0);
    }

    #[test]
    fn file_view_data_type_bank_16_le() {
        for (n, data_type) in INT_DATA_TYPES {
            let bank = bank_16_le([65; 4], n, &[]);
            let events = event_le(4, 5, 6, 7, 1, &bank);
            let file = file_le(1, 2, b"initial", &events, 3, b"final");
            let file_view = FileView::try_from_bytes(&file).unwrap();

            assert_eq!(file_view.run_number(), 1);
            assert_eq!(file_view.initial_timestamp(), 2);
            assert_eq!(file_view.initial_odb(), b"initial");
            assert_eq!(file_view.final_timestamp(), 3);
            assert_eq!(file_view.final_odb(), b"final");
            let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(event_view.id(), 4);
            assert_eq!(event_view.trigger_mask(), 5);
            assert_eq!(event_view.serial_number(), 6);
            assert_eq!(event_view.timestamp(), 7);
            let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(bank_view.name(), [65; 4]);
            assert_eq!(bank_view.data_type(), data_type);
            assert!(bank_view.data().is_empty());
        }
    }

    #[test]
    fn file_view_data_type_bank_16_be() {
        for (n, data_type) in INT_DATA_TYPES {
            let bank = bank_16_be([65; 4], n, &[]);
            let events = event_be(4, 5, 6, 7, 1, &bank);
            let file = file_be(1, 2, b"initial", &events, 3, b"final");
            let file_view = FileView::try_from_bytes(&file).unwrap();

            assert_eq!(file_view.run_number(), 1);
            assert_eq!(file_view.initial_timestamp(), 2);
            assert_eq!(file_view.initial_odb(), b"initial");
            assert_eq!(file_view.final_timestamp(), 3);
            assert_eq!(file_view.final_odb(), b"final");
            let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(event_view.id(), 4);
            assert_eq!(event_view.trigger_mask(), 5);
            assert_eq!(event_view.serial_number(), 6);
            assert_eq!(event_view.timestamp(), 7);
            let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(bank_view.name(), [65; 4]);
            assert_eq!(bank_view.data_type(), data_type);
            assert!(bank_view.data().is_empty());
        }
    }

    #[test]
    fn file_view_data_type_bank_32_le() {
        for (n, data_type) in INT_DATA_TYPES {
            let bank = bank_32_le([65; 4], n.into(), &[]);
            let events = event_le(4, 5, 6, 7, 17, &bank);
            let file = file_le(1, 2, b"initial", &events, 3, b"final");
            let file_view = FileView::try_from_bytes(&file).unwrap();

            assert_eq!(file_view.run_number(), 1);
            assert_eq!(file_view.initial_timestamp(), 2);
            assert_eq!(file_view.initial_odb(), b"initial");
            assert_eq!(file_view.final_timestamp(), 3);
            assert_eq!(file_view.final_odb(), b"final");
            let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(event_view.id(), 4);
            assert_eq!(event_view.trigger_mask(), 5);
            assert_eq!(event_view.serial_number(), 6);
            assert_eq!(event_view.timestamp(), 7);
            let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(bank_view.name(), [65; 4]);
            assert_eq!(bank_view.data_type(), data_type);
            assert!(bank_view.data().is_empty());
        }
    }

    #[test]
    fn file_view_data_type_bank_32_be() {
        for (n, data_type) in INT_DATA_TYPES {
            let bank = bank_32_be([65; 4], n.into(), &[]);
            let events = event_be(4, 5, 6, 7, 17, &bank);
            let file = file_be(1, 2, b"initial", &events, 3, b"final");
            let file_view = FileView::try_from_bytes(&file).unwrap();

            assert_eq!(file_view.run_number(), 1);
            assert_eq!(file_view.initial_timestamp(), 2);
            assert_eq!(file_view.initial_odb(), b"initial");
            assert_eq!(file_view.final_timestamp(), 3);
            assert_eq!(file_view.final_odb(), b"final");
            let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(event_view.id(), 4);
            assert_eq!(event_view.trigger_mask(), 5);
            assert_eq!(event_view.serial_number(), 6);
            assert_eq!(event_view.timestamp(), 7);
            let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(bank_view.name(), [65; 4]);
            assert_eq!(bank_view.data_type(), data_type);
            assert!(bank_view.data().is_empty());
        }
    }

    #[test]
    fn file_view_data_type_bank_32a_le() {
        for (n, data_type) in INT_DATA_TYPES {
            let bank = bank_32a_le([65; 4], n.into(), &[]);
            let events = event_le(4, 5, 6, 7, 49, &bank);
            let file = file_le(1, 2, b"initial", &events, 3, b"final");
            let file_view = FileView::try_from_bytes(&file).unwrap();

            assert_eq!(file_view.run_number(), 1);
            assert_eq!(file_view.initial_timestamp(), 2);
            assert_eq!(file_view.initial_odb(), b"initial");
            assert_eq!(file_view.final_timestamp(), 3);
            assert_eq!(file_view.final_odb(), b"final");
            let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(event_view.id(), 4);
            assert_eq!(event_view.trigger_mask(), 5);
            assert_eq!(event_view.serial_number(), 6);
            assert_eq!(event_view.timestamp(), 7);
            let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(bank_view.name(), [65; 4]);
            assert_eq!(bank_view.data_type(), data_type);
            assert!(bank_view.data().is_empty());
        }
    }

    #[test]
    fn file_view_data_type_bank_32a_be() {
        for (n, data_type) in INT_DATA_TYPES {
            let bank = bank_32a_be([65; 4], n.into(), &[]);
            let events = event_be(4, 5, 6, 7, 49, &bank);
            let file = file_be(1, 2, b"initial", &events, 3, b"final");
            let file_view = FileView::try_from_bytes(&file).unwrap();

            assert_eq!(file_view.run_number(), 1);
            assert_eq!(file_view.initial_timestamp(), 2);
            assert_eq!(file_view.initial_odb(), b"initial");
            assert_eq!(file_view.final_timestamp(), 3);
            assert_eq!(file_view.final_odb(), b"final");
            let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(event_view.id(), 4);
            assert_eq!(event_view.trigger_mask(), 5);
            assert_eq!(event_view.serial_number(), 6);
            assert_eq!(event_view.timestamp(), 7);
            let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
                panic!()
            };
            assert_eq!(bank_view.name(), [65; 4]);
            assert_eq!(bank_view.data_type(), data_type);
            assert!(bank_view.data().is_empty());
        }
    }

    #[test]
    fn file_view_bank_32a_non_zero_reserved_le() {
        let mut bank = bank_32a_le([65; 4], 1, &[2; 100]);
        bank[12..16].copy_from_slice(&[0xFF; 4]);
        let events = event_le(3, 4, 5, 6, 49, &bank);
        let file = file_le(7, 8, b"initial", &events, 9, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 3);
        assert_eq!(event_view.trigger_mask(), 4);
        assert_eq!(event_view.serial_number(), 5);
        assert_eq!(event_view.timestamp(), 6);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert_eq!(bank_view.data(), &[2; 100]);
    }

    #[test]
    fn file_view_bank_32a_non_zero_reserved_be() {
        let mut bank = bank_32a_be([65; 4], 1, &[2; 100]);
        bank[12..16].copy_from_slice(&[0xFF; 4]);
        let events = event_be(3, 4, 5, 6, 49, &bank);
        let file = file_be(7, 8, b"initial", &events, 9, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 3);
        assert_eq!(event_view.trigger_mask(), 4);
        assert_eq!(event_view.serial_number(), 5);
        assert_eq!(event_view.timestamp(), 6);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert_eq!(bank_view.data(), &[2; 100]);
    }

    #[test]
    fn file_view_bank_16_non_zero_padding_le() {
        let mut bank = bank_16_le([65; 4], 1, &[2; 100]);
        bank[108..112].copy_from_slice(&[0xFF; 4]);
        let events = event_le(3, 4, 5, 6, 1, &bank);
        let file = file_le(7, 8, b"initial", &events, 9, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 3);
        assert_eq!(event_view.trigger_mask(), 4);
        assert_eq!(event_view.serial_number(), 5);
        assert_eq!(event_view.timestamp(), 6);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert_eq!(bank_view.data(), &[2; 100]);
    }

    #[test]
    fn file_view_bank_16_non_zero_padding_be() {
        let mut bank = bank_16_be([65; 4], 1, &[2; 100]);
        bank[108..112].copy_from_slice(&[0xFF; 4]);
        let events = event_be(3, 4, 5, 6, 1, &bank);
        let file = file_be(7, 8, b"initial", &events, 9, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 3);
        assert_eq!(event_view.trigger_mask(), 4);
        assert_eq!(event_view.serial_number(), 5);
        assert_eq!(event_view.timestamp(), 6);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert_eq!(bank_view.data(), &[2; 100]);
    }

    #[test]
    fn file_view_bank_32_non_zero_padding_le() {
        let mut bank = bank_32_le([65; 4], 1, &[2; 100]);
        bank[112..116].copy_from_slice(&[0xFF; 4]);
        let events = event_le(3, 4, 5, 6, 17, &bank);
        let file = file_le(7, 8, b"initial", &events, 9, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 3);
        assert_eq!(event_view.trigger_mask(), 4);
        assert_eq!(event_view.serial_number(), 5);
        assert_eq!(event_view.timestamp(), 6);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert_eq!(bank_view.data(), &[2; 100]);
    }

    #[test]
    fn file_view_bank_32_non_zero_padding_be() {
        let mut bank = bank_32_be([65; 4], 1, &[2; 100]);
        bank[112..116].copy_from_slice(&[0xFF; 4]);
        let events = event_be(3, 4, 5, 6, 17, &bank);
        let file = file_be(7, 8, b"initial", &events, 9, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 3);
        assert_eq!(event_view.trigger_mask(), 4);
        assert_eq!(event_view.serial_number(), 5);
        assert_eq!(event_view.timestamp(), 6);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert_eq!(bank_view.data(), &[2; 100]);
    }

    #[test]
    fn file_view_bank_32a_non_zero_padding_le() {
        let mut bank = bank_32a_le([65; 4], 1, &[2; 100]);
        bank[116..120].copy_from_slice(&[0xFF; 4]);
        let events = event_le(3, 4, 5, 6, 49, &bank);
        let file = file_le(7, 8, b"initial", &events, 9, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 3);
        assert_eq!(event_view.trigger_mask(), 4);
        assert_eq!(event_view.serial_number(), 5);
        assert_eq!(event_view.timestamp(), 6);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert_eq!(bank_view.data(), &[2; 100]);
    }

    #[test]
    fn file_view_bank_32a_non_zero_padding_be() {
        let mut bank = bank_32a_be([65; 4], 1, &[2; 100]);
        bank[116..120].copy_from_slice(&[0xFF; 4]);
        let events = event_be(3, 4, 5, 6, 49, &bank);
        let file = file_be(7, 8, b"initial", &events, 9, b"final");
        let file_view = FileView::try_from_bytes(&file).unwrap();

        assert_eq!(file_view.run_number(), 7);
        assert_eq!(file_view.initial_timestamp(), 8);
        assert_eq!(file_view.initial_odb(), b"initial");
        assert_eq!(file_view.final_timestamp(), 9);
        assert_eq!(file_view.final_odb(), b"final");
        let [ref event_view] = file_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(event_view.id(), 3);
        assert_eq!(event_view.trigger_mask(), 4);
        assert_eq!(event_view.serial_number(), 5);
        assert_eq!(event_view.timestamp(), 6);
        let [bank_view] = event_view.into_iter().collect::<Vec<_>>()[..] else {
            panic!()
        };
        assert_eq!(bank_view.name(), [65; 4]);
        assert_eq!(bank_view.data_type(), DataType::U8);
        assert_eq!(bank_view.data(), &[2; 100]);
    }

    #[test]
    fn file_view_bank_16_invalid_data_type_le() {
        let bank = bank_16_le([65; 4], 0, &[]);
        let events = event_le(0, 0, 0, 0, 1, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_16_invalid_data_type_be() {
        let bank = bank_16_be([65; 4], 0, &[]);
        let events = event_be(0, 0, 0, 0, 1, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_32_invalid_data_type_le() {
        let bank = bank_32_le([65; 4], 0, &[]);
        let events = event_le(0, 0, 0, 0, 17, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_32_invalid_data_type_be() {
        let bank = bank_32_be([65; 4], 0, &[]);
        let events = event_be(0, 0, 0, 0, 17, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_32a_invalid_data_type_le() {
        let bank = bank_32a_le([65; 4], 0, &[]);
        let events = event_le(0, 0, 0, 0, 49, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_32a_invalid_data_type_be() {
        let bank = bank_32a_be([65; 4], 0, &[]);
        let events = event_be(0, 0, 0, 0, 49, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_16_non_integer_data_elements_le() {
        let bank = bank_16_le([65; 4], 4, &[0; 99]);
        let events = event_le(0, 0, 0, 0, 1, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_16_non_integer_data_elements_be() {
        let bank = bank_16_be([65; 4], 4, &[0; 99]);
        let events = event_be(0, 0, 0, 0, 1, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_32_non_integer_data_elements_le() {
        let bank = bank_32_le([65; 4], 4, &[0; 99]);
        let events = event_le(0, 0, 0, 0, 17, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_32_non_integer_data_elements_be() {
        let bank = bank_32_be([65; 4], 4, &[0; 99]);
        let events = event_be(0, 0, 0, 0, 17, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_32a_non_integer_data_elements_le() {
        let bank = bank_32a_le([65; 4], 4, &[0; 99]);
        let events = event_le(0, 0, 0, 0, 49, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_bank_32a_non_integer_data_elements_be() {
        let bank = bank_32a_be([65; 4], 4, &[0; 99]);
        let events = event_be(0, 0, 0, 0, 49, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_event_16_bad_bank_le() {
        let mut bank = bank_16_le([65; 4], 1, &[0; 100]);
        bank[6..8].copy_from_slice(&96u16.to_le_bytes());
        let events = event_le(0, 0, 0, 0, 1, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_event_16_bad_bank_be() {
        let mut bank = bank_16_be([65; 4], 1, &[0; 100]);
        bank[6..8].copy_from_slice(&96u16.to_be_bytes());
        let events = event_be(0, 0, 0, 0, 1, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_event_32_bad_bank_le() {
        let mut bank = bank_32_le([65; 4], 1, &[0; 100]);
        bank[8..12].copy_from_slice(&96u32.to_le_bytes());
        let events = event_le(0, 0, 0, 0, 17, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_event_32_bad_bank_be() {
        let mut bank = bank_32_be([65; 4], 1, &[0; 100]);
        bank[8..12].copy_from_slice(&96u32.to_be_bytes());
        let events = event_be(0, 0, 0, 0, 17, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_event_32a_bad_bank_le() {
        let mut bank = bank_32a_le([65; 4], 1, &[0; 100]);
        bank[8..12].copy_from_slice(&96u32.to_le_bytes());
        let events = event_le(0, 0, 0, 0, 49, &bank);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_event_32a_bad_bank_be() {
        let mut bank = bank_32a_be([65; 4], 1, &[0; 100]);
        bank[8..12].copy_from_slice(&96u32.to_be_bytes());
        let events = event_be(0, 0, 0, 0, 49, &bank);
        let file = file_be(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_event_flags_le() {
        let events = event_le(0, 0, 0, 0, 0, &[]);
        let file = file_le(0, 0, b"", &events, 0, b"");
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_bor_le() {
        let mut file = file_le(0, 0, b"", &[], 0, b"");
        file[0..2].copy_from_slice(&[0, 0]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_bor_be() {
        let mut file = file_be(0, 0, b"", &[], 0, b"");
        file[0..2].copy_from_slice(&[0, 0]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_initial_magic_le() {
        let mut file = file_le(0, 0, b"", &[], 0, b"");
        file[2..4].copy_from_slice(&[0, 0]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_initial_magic_be() {
        let mut file = file_be(0, 0, b"", &[], 0, b"");
        file[2..4].copy_from_slice(&[0, 0]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_run_number_mismatch_le() {
        let mut file = file_le(0, 0, b"", &[], 0, b"");
        file[4..8].copy_from_slice(&[0xFF; 4]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_run_number_mismatch_be() {
        let mut file = file_be(0, 0, b"", &[], 0, b"");
        file[4..8].copy_from_slice(&[0xFF; 4]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_eor_le() {
        let mut file = file_le(0, 0, b"", &[], 0, b"");
        file[16..18].copy_from_slice(&[0, 0]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_eor_be() {
        let mut file = file_be(0, 0, b"", &[], 0, b"");
        file[16..18].copy_from_slice(&[0, 0]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_final_magic_le() {
        let mut file = file_le(0, 0, b"", &[], 0, b"");
        file[18..20].copy_from_slice(&[0, 0]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_invalid_final_magic_be() {
        let mut file = file_be(0, 0, b"", &[], 0, b"");
        file[18..20].copy_from_slice(&[0, 0]);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_extra_bytes_le() {
        let mut file = file_le(0, 0, b"", &[], 0, b"");
        file.push(0);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn file_view_extra_bytes_be() {
        let mut file = file_be(0, 0, b"", &[], 0, b"");
        file.push(0);
        assert!(FileView::try_from_bytes(&file).is_err());
    }

    #[test]
    fn run_number_unchecked_le() {
        let bytes = b"\x00\x80\xFF\xFF\x01\x00\x00\x00\xFF";
        assert_eq!(run_number_unchecked(bytes).unwrap(), 1);
    }

    #[test]
    fn run_number_unchecked_be() {
        let bytes = b"\x80\x00\xFF\xFF\x00\x00\x00\x01\xFF";
        assert_eq!(run_number_unchecked(bytes).unwrap(), 1);
    }

    #[test]
    fn run_number_unchecked_invalid_bor_marker() {
        let bytes = b"\xFF\xFF\xFF\xFF\x01\x00\x00\x00";
        assert!(run_number_unchecked(bytes).is_err());
    }

    #[test]
    fn run_number_unchecked_invalid_run_number_le() {
        let bytes = b"\x00\x80\xFF\xFF\x12\x34\x56";
        assert!(run_number_unchecked(bytes).is_err());
    }

    #[test]
    fn run_number_unchecked_invalid_run_number_be() {
        let bytes = b"\x80\x00\xFF\xFF\x12\x34\x56";
        assert!(run_number_unchecked(bytes).is_err());
    }

    #[test]
    fn initial_timestamp_unchecked_le() {
        let bytes = b"\x00\x80\xFF\xFF\xFF\xFF\xFF\xFF\x01\x00\x00\x00\xFF";
        assert_eq!(initial_timestamp_unchecked(bytes).unwrap(), 1);
    }

    #[test]
    fn initial_timestamp_unchecked_be() {
        let bytes = b"\x80\x00\xFF\xFF\xFF\xFF\xFF\xFF\x00\x00\x00\x01\xFF";
        assert_eq!(initial_timestamp_unchecked(bytes).unwrap(), 1);
    }

    #[test]
    fn initial_timestamp_unchecked_invalid_bor_marker() {
        let bytes = b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x01\x00\x00\x00";
        assert!(initial_timestamp_unchecked(bytes).is_err());
    }

    #[test]
    fn initial_timestamp_unchecked_invalid_timestamp_le() {
        let bytes = b"\x00\x80\xFF\xFF\xFF\xFF\xFF\xFF\x12\x34\x56";
        assert!(initial_timestamp_unchecked(bytes).is_err());
    }

    #[test]
    fn initial_timestamp_unchecked_invalid_timestamp_be() {
        let bytes = b"\x80\x00\xFF\xFF\xFF\xFF\xFF\xFF\x12\x34\x56";
        assert!(initial_timestamp_unchecked(bytes).is_err());
    }
}
