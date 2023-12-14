use crate::data_bank::{bank_16_view, bank_32_view, bank_32a_view, BankView};
use thiserror::Error;
use winnow::binary::{u16, u32, Endianness};
use winnow::combinator::{dispatch, fail, repeat};
use winnow::error::{ContextError, ParseError, StrContext};
use winnow::token::{take, take_while};
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
/// An event is defined as a 24 bytes header followed by an arbitrary (non-zero)
/// number of [`BankView`]s. The binary representation of a MIDAS event is:
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
        let bank_view = f
            .by_ref()
            .context(StrContext::Label("bank view"))
            .parse_next(input)?
            .into();
        let _ = take_while(bank_view.required_padding(), 0)
            .context(StrContext::Label("bank padding"))
            .parse_next(input)?;

        Ok(bank_view)
    }
}

pub(crate) fn event_view<'a>(
    endian: Endianness,
) -> impl Parser<&'a [u8], EventView<'a>, ContextError> {
    move |input: &mut &'a [u8]| {
        let (event_id, trigger_mask, serial_number, timestamp, bank_views) = (
            u16(endian).context(StrContext::Label("event id")),
            u16(endian).context(StrContext::Label("trigger mask")),
            u32(endian).context(StrContext::Label("serial number")),
            u32(endian).context(StrContext::Label("timestamp")),
            // The MIDAS wiki:
            // https://daq00.triumf.ca/MidasWiki/index.php/Event_Structure#Data_Area
            // says that the data area contains at least 1 data bank.
            u32(endian)
                .context(StrContext::Label("event size"))
                .verify(|&event_size| event_size >= 8)
                .flat_map(|event_size| {
                    u32(endian)
                        .verify(move |&banks_size| banks_size == event_size - 8)
                        .context(StrContext::Label("all banks size"))
                })
                .flat_map(|banks_size| {
                    dispatch! {u32(endian).context(StrContext::Label("flags"));
                        1 => take(banks_size).and_then(repeat(1.., padded_bank(bank_16_view(endian)))),
                        17 => take(banks_size).and_then(repeat(1.., padded_bank(bank_32_view(endian)))),
                        49 => take(banks_size).and_then(repeat(1.., padded_bank(bank_32a_view(endian)))),
                        _ => fail.context(StrContext::Label("unknown flags")),
                    }
                })
                .context(StrContext::Label("all banks slice"))
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
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::event::EventView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let header = [
    ///     1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
    /// ];
    /// let bank = b"BANK\x01\x00\x01\x00\xFF";
    /// let padding = [0; 7];
    ///
    /// let bytes = [&header[..], bank, &padding].concat();
    /// let event = EventView::try_from_le_bytes(&bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_le_bytes(bytes: &'a [u8]) -> Result<Self, TryEventViewFromBytesError> {
        Ok(event_view(Endianness::Little).parse(bytes)?)
    }
    /// Create a native view to the underlying event from its representation as
    /// a byte slice in big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::event::EventView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let header = [
    ///    0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    /// ];
    /// let bank = b"BANK\x00\x01\x00\x01\xFF";
    /// let padding = [0; 7];
    ///
    /// let bytes = [&header[..], bank, &padding].concat();
    /// let event = EventView::try_from_be_bytes(&bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_be_bytes(bytes: &'a [u8]) -> Result<Self, TryEventViewFromBytesError> {
        Ok(event_view(Endianness::Big).parse(bytes)?)
    }
    /// Return the ID of the event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::event::EventView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// # Ok(())
    /// # }
    /// ```
    pub fn id(&self) -> u16 {
        self.event_id
    }
    /// Return the trigger mask of the event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::event::EventView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let header = [
    ///     1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
    /// ];
    /// let bank = b"BANK\x01\x00\x01\x00\xFF";
    /// let padding = [0; 7];
    ///
    /// let bytes = [&header[..], bank, &padding].concat();
    /// let event = EventView::try_from_le_bytes(&bytes)?;
    ///
    /// assert_eq!(event.trigger_mask(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn trigger_mask(&self) -> u16 {
        self.trigger_mask
    }
    /// Return the serial number of the event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::event::EventView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let header = [
    ///     1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
    /// ];
    /// let bank = b"BANK\x01\x00\x01\x00\xFF";
    /// let padding = [0; 7];
    ///
    /// let bytes = [&header[..], bank, &padding].concat();
    /// let event = EventView::try_from_le_bytes(&bytes)?;
    ///
    /// assert_eq!(event.serial_number(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn serial_number(&self) -> u32 {
        self.serial_number
    }
    /// Return the timestamp of the event.
    ///
    /// # Examples
    ///
    /// ```
    /// use midasio::event::EventView;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let header = [
    ///     1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
    /// ];
    /// let bank = b"BANK\x01\x00\x01\x00\xFF";
    /// let padding = [0; 7];
    ///
    /// let bytes = [&header[..], bank, &padding].concat();
    /// let event = EventView::try_from_le_bytes(&bytes)?;
    ///
    /// assert_eq!(event.timestamp(), 4);
    /// # Ok(())
    /// # }
    /// ```
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

#[cfg(test)]
mod tests;
