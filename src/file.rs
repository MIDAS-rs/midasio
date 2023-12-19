use crate::event::{event_view, EventView};
use thiserror::Error;
use winnow::binary::{le_u16, length_take, u16, u32, Endianness};
use winnow::combinator::{delimited, dispatch, fail, repeat, seq, success};
use winnow::error::{ContextError, PResult, ParseError, StrContext};
use winnow::token::{take, take_while};
use winnow::Parser;

#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};

/// The error type returned when conversion from
/// [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) to a
/// [`FileView`] fails.
#[derive(Debug, Error)]
// I am still experimenting with the error type. This allows me to hide the
// implementation details of the error type without breaking the public API.
#[error(transparent)]
pub struct TryFileViewFromBytesError(#[from] InnerFileParseError);

#[derive(Debug)]
struct InnerFileParseError {
    offset: usize,
    inner: ContextError,
}

impl std::fmt::Display for InnerFileParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsing failed at byte offset `{}`", self.offset)?;
        if self.inner.context().next().is_some() {
            write!(f, " ({})", self.inner)?;
        }
        Ok(())
    }
}

impl std::error::Error for InnerFileParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner
            .cause()
            .map(|v| v as &(dyn std::error::Error + 'static))
    }
}

#[doc(hidden)]
impl<I> From<ParseError<I, ContextError>> for TryFileViewFromBytesError {
    fn from(e: ParseError<I, ContextError>) -> Self {
        Self(InnerFileParseError {
            offset: e.offset(),
            inner: e.into_inner(),
        })
    }
}

/// An immutable view to a MIDAS file.
///
/// A files is a collection of [`EventView`]s wrapped by two dumps of the Online
/// Data Base (ODB) at the beginning and end of the sub-run. The binary
/// representation of a file is:
///
/// <center>
///
/// |Byte Offset|Size (in bytes)|Description|
/// |:-:|:-:|:-:|
/// |0|2|Begin-of-run marker (`0x8000`)|
/// |2|2|Magic-midas marker (`0x494D`)|
/// |4|4|Run number|
/// |8|4|Initial unix timestamp|
/// |12|4|Initial ODB dump size (`n`)|
/// |16|`n`|Initial ODB dump|
/// |16 + `n`|`m`|Events|
/// |16 + `n` + `m`|2|End-of-run marker (`0x8001`)|
/// |18 + `n` + `m`|2|Magic-midas marker (`0x494D`)|
/// |20 + `n` + `m`|4|Run number|
/// |24 + `n` + `m`|4|Final unix timestamp|
/// |28 + `n` + `m`|4|Final ODB dump size (`k`)|
/// |32 + `n` + `m`|`k`|Final ODB dump|
/// </center>
#[derive(Clone, Debug)]
pub struct FileView<'a> {
    run_number: u32,
    initial_timestamp: u32,
    initial_odb: &'a [u8],
    event_views: Vec<EventView<'a>>,
    final_timestamp: u32,
    final_odb: &'a [u8],
}

const BOR_ID: u16 = 0x8000;
const BOR_ID_SWAPPED: u16 = BOR_ID.swap_bytes();
const EOR_ID: u16 = 0x8001;
const MAGIC: u16 = 0x494D;

fn endian(input: &mut &[u8]) -> PResult<Endianness> {
    dispatch! {le_u16;
        BOR_ID => success(Endianness::Little),
        BOR_ID_SWAPPED => success(Endianness::Big),
        _ => fail,
    }
    .parse_next(input)
}

fn file_view<'a>(input: &mut &'a [u8]) -> PResult<FileView<'a>> {
    let endianness = endian
        .context(StrContext::Label("begin-of-run id"))
        .parse_next(input)?;

    seq! {FileView{
        _: u16(endianness).verify(|&magic| magic == MAGIC)
            .context(StrContext::Label("initial magic midas marker")),
        run_number: u32(endianness)
            .context(StrContext::Label("initial run number")),
        initial_timestamp: u32(endianness)
            .context(StrContext::Label("initial unix timestamp")),
        initial_odb: length_take(u32(endianness))
            .context(StrContext::Label("initial odb dump")),
        event_views: repeat(0.., event_view(endianness)),
        _: u16(endianness).verify(|&eor_id| eor_id == EOR_ID)
            .context(StrContext::Label("end-of-run id")),
        _: u16(endianness).verify(|&magic| magic == MAGIC)
            .context(StrContext::Label("final magic midas marker")),
        _: u32(endianness).verify(move |&n| n == run_number)
            .context(StrContext::Label("final run number")),
        final_timestamp: u32(endianness)
            .context(StrContext::Label("final unix timestamp")),
        final_odb: length_take(u32(endianness))
            .context(StrContext::Label("final odb dump")),
    }}
    .parse_next(input)
}

impl<'a> TryFrom<&'a [u8]> for FileView<'a> {
    type Error = TryFileViewFromBytesError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(file_view.parse(bytes)?)
    }
}

impl<'a> FileView<'a> {
    /// Create a native view to the underlying file from its representation as a
    /// byte slice. Endianness is determined from the begin-of-run marker.
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, TryFileViewFromBytesError> {
        Self::try_from(bytes)
    }
    /// Return the run number associated with the file.
    pub fn run_number(&self) -> u32 {
        self.run_number
    }
    /// Return the unix timestamp of the initial ODB dump.
    pub fn initial_timestamp(&self) -> u32 {
        self.initial_timestamp
    }
    /// Return the initial ODB dump. This is not guaranteed to be valid ASCII
    /// nor UTF-8.
    pub fn initial_odb(&self) -> &'a [u8] {
        self.initial_odb
    }
    /// Return the unix timestamp of the final ODB dump.
    pub fn final_timestamp(&self) -> u32 {
        self.final_timestamp
    }
    /// Return the final ODB dump. This is not guaranteed to be valid ASCII
    /// nor UTF-8.
    pub fn final_odb(&self) -> &'a [u8] {
        self.final_odb
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
        self.event_views.into_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a> IntoParallelIterator for FileView<'a> {
    type Item = EventView<'a>;
    type Iter = rayon::vec::IntoIter<EventView<'a>>;

    fn into_par_iter(self) -> Self::Iter {
        self.event_views.into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, 'b> IntoParallelIterator for &'b FileView<'a> {
    type Item = &'b EventView<'a>;
    type Iter = rayon::slice::Iter<'b, EventView<'a>>;

    fn into_par_iter(self) -> Self::Iter {
        self.event_views.par_iter()
    }
}

/// Return the run number assuming that the input slice has the correct MIDAS
/// file format.
///
/// This is useful for checking the run number of a file without having to parse
/// its entire contents. Returns an error if the run number cannot be
/// determined.
///
/// # Examples
///
/// ```
/// use midasio::file::run_number_unchecked;
///
/// // Note that the following is an invalid MIDAS file:
/// // - The magic midas marker is 0xFFFF instead of 0x494D.
/// // - Too short to even contain the rest of the header.
/// let bytes = b"\x00\x80\xFF\xFF\x01\x00\x00\x00";
/// // Nonetheless, a "run number" can still be extracted with this function.
/// let run_number = run_number_unchecked(bytes)?;
/// assert_eq!(run_number, 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn run_number_unchecked(bytes: &[u8]) -> Result<u32, TryFileViewFromBytesError> {
    fn run_number(input: &mut &[u8]) -> PResult<u32> {
        let endianness = endian
            .context(StrContext::Label("begin-of-run id"))
            .parse_next(input)?;
        delimited(
            take(2usize).context(StrContext::Label("magic midas marker")),
            u32(endianness).context(StrContext::Label("run number")),
            take_while(0.., |_| true),
        )
        .parse_next(input)
    }

    Ok(run_number.parse(bytes)?)
}

/// Return the unix timestamp of the initial ODB dump assuming that the input
/// slice has the correct MIDAS file format.
///
/// This is useful for checking the initial timestamp of a file without having
/// to parse its entire contents. Returns an error if the timestamp cannot be
/// determined.
///
/// # Examples
///
/// ```
/// use midasio::file::initial_timestamp_unchecked;
///
/// // Note that the following is an invalid MIDAS file:
/// // - The magic midas marker is 0xFFFF instead of 0x494D.
/// // - Too short to even contain the rest of the header.
/// let bytes = b"\x00\x80\xFF\xFF\xFF\xFF\xFF\xFF\x01\x00\x00\x00";
/// // Nonetheless, an "initial timestamp" can still be extracted with this function.
/// let timestamp = initial_timestamp_unchecked(bytes)?;
/// assert_eq!(timestamp, 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn initial_timestamp_unchecked(bytes: &[u8]) -> Result<u32, TryFileViewFromBytesError> {
    fn initial_timestamp(input: &mut &[u8]) -> PResult<u32> {
        let endianness = endian
            .context(StrContext::Label("begin-of-run id"))
            .parse_next(input)?;
        delimited(
            take(6usize).context(StrContext::Label("magic midas marker or run number")),
            u32(endianness).context(StrContext::Label("initial timestamp")),
            take_while(0.., |_| true),
        )
        .parse_next(input)
    }

    Ok(initial_timestamp.parse(bytes)?)
}
