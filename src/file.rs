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

#[cfg(test)]
mod tests {
    use super::*;

    fn le_file_bytes_example() -> Vec<u8> {
        let initial_dump =
            b"\x00\x80\x4D\x49\x01\x00\x00\x00\x02\x00\x00\x00\x0C\x00\x00\x00initial dump";
        let event_header = [
            1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
        ];
        let padded_bank = b"NAME\x01\x00\x01\x00\xFF\x00\x00\x00\x00\x00\x00\x00";
        let final_dump =
            b"\x01\x80\x4D\x49\x01\x00\x00\x00\x03\x00\x00\x00\x0A\x00\x00\x00final dump";

        [&initial_dump[..], &event_header, padded_bank, final_dump]
            .concat()
            .to_vec()
    }

    #[test]
    fn file_view_try_from_le_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = le_file_bytes_example();
        let file = FileView::try_from_bytes(&bytes[..])?;
        assert_eq!(file.run_number(), 1);
        assert_eq!(file.initial_timestamp(), 2);
        assert_eq!(file.initial_odb(), b"initial dump");
        assert_eq!(file.final_timestamp(), 3);
        assert_eq!(file.final_odb(), b"final dump");
        Ok(())
    }

    #[test]
    fn file_view_try_from_be_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let initial_dump =
            b"\x80\x00\x49\x4D\x00\x00\x00\x01\x00\x00\x00\x02\x00\x00\x00\x0Cinitial dump";
        let event_header = [
            0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
        ];
        let padded_bank = b"NAME\x00\x01\x00\x01\xFF\x00\x00\x00\x00\x00\x00\x00";
        let final_dump =
            b"\x80\x01\x49\x4D\x00\x00\x00\x01\x00\x00\x00\x03\x00\x00\x00\x0Afinal dump";

        let bytes = [&initial_dump[..], &event_header, padded_bank, final_dump].concat();
        let file = FileView::try_from_bytes(&bytes[..])?;
        assert_eq!(file.run_number(), 1);
        assert_eq!(file.initial_timestamp(), 2);
        assert_eq!(file.initial_odb(), b"initial dump");
        assert_eq!(file.final_timestamp(), 3);
        assert_eq!(file.final_odb(), b"final dump");
        Ok(())
    }

    #[test]
    fn file_view_invalid_bor_marker() {
        let mut bytes = le_file_bytes_example();
        bytes[0] = 0xFF;
        bytes[1] = 0xFF;
        let result = FileView::try_from_bytes(&bytes[..]);
        assert!(result.is_err());
    }

    #[test]
    fn file_view_invalid_initial_magic_midas_marker() {
        let mut bytes = le_file_bytes_example();
        bytes[2] = 0xFF;
        bytes[3] = 0xFF;
        let result = FileView::try_from_bytes(&bytes[..]);
        assert!(result.is_err());
    }

    #[test]
    fn file_view_invalid_eor_marker() {
        let mut bytes = le_file_bytes_example();
        bytes[68] = 0xFF;
        bytes[69] = 0xFF;
        let result = FileView::try_from_bytes(&bytes[..]);
        assert!(result.is_err());
    }

    #[test]
    fn file_view_invalid_final_magic_midas_marker() {
        let mut bytes = le_file_bytes_example();
        bytes[70] = 0xFF;
        bytes[71] = 0xFF;
        let result = FileView::try_from_bytes(&bytes[..]);
        assert!(result.is_err());
    }

    #[test]
    fn file_view_run_number_mismatch() {
        let mut bytes = le_file_bytes_example();
        bytes[4] = 0xFF;
        bytes[5] = 0xFF;
        let result = FileView::try_from_bytes(&bytes[..]);
        assert!(result.is_err());
    }

    #[test]
    fn file_view_into_iter() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = le_file_bytes_example();
        let file = FileView::try_from_bytes(&bytes[..])?;

        assert_eq!(file.into_iter().count(), 1);
        Ok(())
    }

    #[test]
    fn run_number_unchecked_le_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"\x00\x80\xFF\xFF\x01\x00\x00\x00\xFF";
        assert_eq!(run_number_unchecked(bytes)?, 1);
        Ok(())
    }

    #[test]
    fn run_number_unchecked_be_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"\x80\x00\xFF\xFF\x00\x00\x00\x01";
        assert_eq!(run_number_unchecked(bytes)?, 1);
        Ok(())
    }

    #[test]
    fn run_number_unchecked_invalid_bor_marker() {
        let bytes = b"\xFF\xFF\xFF\xFF\x01\x00\x00\x00";
        assert!(run_number_unchecked(bytes).is_err());
    }

    #[test]
    fn run_number_unchecked_invalid_run_number() {
        let bytes = b"\x80\x00\xFF\xFF\x12\x34\x56";
        assert!(run_number_unchecked(bytes).is_err());
    }

    #[test]
    fn initial_timestamp_unchecked_le_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"\x00\x80\xFF\xFF\xFF\xFF\xFF\xFF\x01\x00\x00\x00";
        assert_eq!(initial_timestamp_unchecked(bytes)?, 1);
        Ok(())
    }

    #[test]
    fn initial_timestamp_unchecked_be_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = b"\x80\x00\xFF\xFF\xFF\xFF\xFF\xFF\x00\x00\x00\x01\xFF";
        assert_eq!(initial_timestamp_unchecked(bytes)?, 1);
        Ok(())
    }

    #[test]
    fn initial_timestamp_unchecked_invalid_bor_marker() {
        let bytes = b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x01\x00\x00\x00";
        assert!(initial_timestamp_unchecked(bytes).is_err());
    }

    #[test]
    fn initial_timestamp_unchecked_invalid_timestamp() {
        let bytes = b"\x80\x00\xFF\xFF\xFF\xFF\xFF\xFF\x12\x34\x56";
        assert!(initial_timestamp_unchecked(bytes).is_err());
    }
}
