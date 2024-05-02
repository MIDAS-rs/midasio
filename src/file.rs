use crate::event::{event_view, EventView};
use thiserror::Error;
use winnow::binary::{le_u16, length_take, u16, u32, Endianness};
use winnow::combinator::{delimited, dispatch, empty, fail, repeat, rest, seq};
use winnow::error::{ContextError, PResult, ParseError, StrContext};
use winnow::token::take;
use winnow::Parser;

#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};

const BOR_ID: u16 = 0x8000;
const BOR_ID_SWAPPED: u16 = BOR_ID.swap_bytes();
const EOR_ID: u16 = 0x8001;
const MAGIC: u16 = 0x494D;

fn endian(input: &mut &[u8]) -> PResult<Endianness> {
    dispatch! {le_u16;
        BOR_ID => empty.value(Endianness::Little),
        BOR_ID_SWAPPED => empty.value(Endianness::Big),
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
            rest,
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
            rest,
        )
        .parse_next(input)
    }

    Ok(initial_timestamp.parse(bytes)?)
}
