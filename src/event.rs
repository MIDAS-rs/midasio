use crate::data_bank::{bank_16_view, bank_32_view, bank_32a_view, BankView};
use thiserror::Error;
use winnow::binary::{length_and_then, u16, u32, Endianness};
use winnow::combinator::{dispatch, empty, eof, fail, repeat_till, seq};
use winnow::error::{ContextError, ParseError};
use winnow::token::take;
use winnow::Parser;

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
    seq! {EventView {
        event_id: u16(endian),
        trigger_mask: u16(endian),
        serial_number: u32(endian),
        timestamp: u32(endian),
        bank_views: u32(endian)
            .verify(|&event_size| event_size >= 8)
            .flat_map(|event_size| {
                u32(endian)
                    .verify(move |&banks_size| banks_size == event_size - 8)
            })
            .flat_map(|banks_size| {
                dispatch! {u32(endian);
                    1 => length_and_then(empty.value(banks_size), repeat_till(0.., padded_bank(bank_16_view(endian)), eof)),
                    17 => length_and_then(empty.value(banks_size), repeat_till(0.., padded_bank(bank_32_view(endian)), eof)),
                    49 => length_and_then(empty.value(banks_size), repeat_till(0.., padded_bank(bank_32a_view(endian)), eof)),
                    _ => fail,
                }
            })
            .map(|(bank_views, _)| bank_views),
    }}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_view_invalid_flags() {
        let header = [
            1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let bank = b"NAME\x01\x00\x01\x00\xFF";
        let padding = [0; 7];

        let bytes = [&header[..], bank, &padding].concat();
        let result = EventView::try_from_le_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn event_view_invalid_all_data_banks_due_to_bad_padding() {
        let header = [
            1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 17, 0, 0, 0, 9, 0, 0, 0, 1, 0, 0, 0,
        ];
        let bank = b"NAME\x01\x00\x01\x00\xFF";

        let bytes = [&header[..], bank].concat();
        let result = EventView::try_from_le_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn event_view_invalid_all_data_banks_due_to_bad_bank() {
        let header = [
            1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
        ];
        let invalid_bank = b"NAME\x04\x00\x01\x00\xFF";
        let padding = [0; 7];

        let bytes = [&header[..], invalid_bank, &padding].concat();
        let result = EventView::try_from_le_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn event_view_invalid_all_data_banks_due_to_remaining_bytes() {
        let header = [
            1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 25, 0, 0, 0, 17, 0, 0, 0, 1, 0, 0, 0,
        ];
        let bank = b"NAME\x01\x00\x01\x00\xFF";
        let padding = [0; 7];
        let extra_byte = [0xFF; 1];

        let bytes = [&header[..], bank, &padding, &extra_byte].concat();
        let result = EventView::try_from_le_bytes(&bytes);
        assert!(result.is_err());
    }
}
