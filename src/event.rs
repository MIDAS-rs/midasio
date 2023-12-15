use crate::data_bank::{bank_16_view, bank_32_view, bank_32a_view, BankView};
use thiserror::Error;
use winnow::binary::{length_and_then, u16, u32, Endianness};
use winnow::combinator::{dispatch, eof, fail, repeat_till0, success};
use winnow::error::{ContextError, ParseError};
use winnow::token::take;
use winnow::Parser;

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to an
/// [`EventView`] fails.
#[derive(Debug, Error)]
// I am still experimenting with the error type. This allows me to hide the
// implementation details of the error type without breaking the public API.
#[error(transparent)]
pub struct TryEventViewFromBytesError(#[from] InnerEventParseError);

#[derive(Debug)]
struct InnerEventParseError {
    offset: usize,
    inner: ContextError,
}

impl std::fmt::Display for InnerEventParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsing failed at byte offset `{}`", self.offset)?;
        if self.inner.context().next().is_some() {
            write!(f, " ({})", self.inner)?;
        }
        Ok(())
    }
}

impl std::error::Error for InnerEventParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner
            .cause()
            .map(|v| v as &(dyn std::error::Error + 'static))
    }
}

#[doc(hidden)]
impl<I> From<ParseError<I, ContextError>> for TryEventViewFromBytesError {
    fn from(e: ParseError<I, ContextError>) -> Self {
        Self(InnerEventParseError {
            offset: e.offset(),
            inner: e.into_inner(),
        })
    }
}

/// An immutable view to a MIDAS event.
///
/// An event is defined as a 24 bytes header followed by an arbitrary number of
/// [`BankView`]s. The binary representation of a MIDAS event is:
///
/// <center>
///
/// |Byte Offset|Size (in bytes)|Description|
/// |:-:|:-:|:-:|
/// |0|2|Event ID|
/// |2|2|Trigger mask|
/// |4|4|Serial number|
/// |8|4|Unix timestamp|
/// |12|4|Event size (`n + 8`)|
/// |16|4|All banks size (`n`)|
/// |20|4|Flags|
/// |24|`n`|Data banks|
///
/// </center>
///
/// # Examples
///
/// ```
/// use midasio::event::EventView;
///
/// let header = [
///     1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
/// ];
/// let bank = b"BANK\x01\x00\x01\x00\xFF";
/// let padding = [0; 7];
///
/// let bytes = [&header[..], bank, &padding].concat();
/// let event = EventView::try_from_le_bytes(&bytes)?;
///
/// assert_eq!(event.id(), 1);
/// assert_eq!(event.trigger_mask(), 2);
/// assert_eq!(event.serial_number(), 3);
/// assert_eq!(event.timestamp(), 4);
/// assert_eq!(event.into_iter().count(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct EventView<'a> {
    event_id: u16,
    trigger_mask: u16,
    serial_number: u32,
    timestamp: u32,
    bank_views: Vec<BankView<'a>>,
}

fn padded_bank<'a, F, B>(mut f: F) -> impl Parser<&'a [u8], BankView<'a>, ContextError>
where
    F: Parser<&'a [u8], B, ContextError>,
    B: Into<BankView<'a>>,
{
    move |input: &mut &'a [u8]| {
        let bank_view = f.parse_next(input)?.into();
        let _ = take(bank_view.required_padding()).parse_next(input)?;

        Ok(bank_view)
    }
}

pub(crate) fn event_view<'a>(
    endian: Endianness,
) -> impl Parser<&'a [u8], EventView<'a>, ContextError> {
    move |input: &mut &'a [u8]| {
        let (event_id, trigger_mask, serial_number, timestamp, (bank_views, _)) = (
            u16(endian),
            u16(endian),
            u32(endian),
            u32(endian),
            u32(endian)
                .verify(|&event_size| event_size >= 8)
                .flat_map(|event_size| {
                    u32(endian)
                        .verify(move |&banks_size| banks_size == event_size - 8)
                })
                .flat_map(|banks_size| {
                    dispatch! {u32(endian);
                        1 => length_and_then(success(banks_size), repeat_till0(padded_bank(bank_16_view(endian)), eof)),
                        17 => length_and_then(success(banks_size), repeat_till0(padded_bank(bank_32_view(endian)), eof)),
                        49 => length_and_then(success(banks_size), repeat_till0(padded_bank(bank_32a_view(endian)), eof)),
                        _ => fail,
                    }
                })
            )
                .parse_next(input)?;

        Ok(EventView {
            event_id,
            trigger_mask,
            serial_number,
            timestamp,
            bank_views,
        })
    }
}

impl<'a> EventView<'a> {
    /// Create a native view to the underlying event from its representation as
    /// a byte slice in little endian.
    pub fn try_from_le_bytes(bytes: &'a [u8]) -> Result<Self, TryEventViewFromBytesError> {
        Ok(event_view(Endianness::Little).parse(bytes)?)
    }
    /// Create a native view to the underlying event from its representation as
    /// a byte slice in big endian.
    pub fn try_from_be_bytes(bytes: &'a [u8]) -> Result<Self, TryEventViewFromBytesError> {
        Ok(event_view(Endianness::Big).parse(bytes)?)
    }
    /// Return the ID of the event.
    pub fn id(&self) -> u16 {
        self.event_id
    }
    /// Return the trigger mask of the event.
    pub fn trigger_mask(&self) -> u16 {
        self.trigger_mask
    }
    /// Return the serial number of the event.
    pub fn serial_number(&self) -> u32 {
        self.serial_number
    }
    /// Return the unix timestamp of the event.
    pub fn timestamp(&self) -> u32 {
        self.timestamp
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
        self.bank_views.into_iter()
    }
}
