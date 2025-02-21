use crate::{BankView, DataType, EventView, FileView};
use std::mem::size_of;
use winnow::binary::{le_u16, length_and_then, length_take, u16, u32, Endianness};
use winnow::combinator::{dispatch, empty, eof, fail, repeat, repeat_till, seq, terminated};
use winnow::error::{ContextError, ModalResult, StrContext};
use winnow::token::take;
use winnow::Parser;

macro_rules! impl_data_type_from_unsigned {
    ($num_type:ty) => {
        #[doc(hidden)]
        impl TryFrom<$num_type> for DataType {
            type Error = ();

            fn try_from(num: $num_type) -> Result<Self, Self::Error> {
                match num {
                    1 => Ok(DataType::U8),
                    2 => Ok(DataType::I8),
                    3 => Ok(DataType::U8),
                    4 => Ok(DataType::U16),
                    5 => Ok(DataType::I16),
                    6 => Ok(DataType::U32),
                    7 => Ok(DataType::I32),
                    8 => Ok(DataType::Bool),
                    9 => Ok(DataType::F32),
                    10 => Ok(DataType::F64),
                    11 => Ok(DataType::U32),
                    12 => Ok(DataType::Str),
                    13 => Ok(DataType::Array),
                    14 => Ok(DataType::Struct),
                    15 => Ok(DataType::Str),
                    16 => Ok(DataType::Str),
                    17 => Ok(DataType::I64),
                    18 => Ok(DataType::U64),
                    _ => Err(()),
                }
            }
        }
    };

    ($first:ty, $($rest:ty),+) => {
        impl_data_type_from_unsigned!($first);
        impl_data_type_from_unsigned!($($rest),+);
    };
}
impl_data_type_from_unsigned!(u16, u32);

impl DataType {
    fn size(&self) -> Option<usize> {
        match self {
            DataType::U8 => Some(size_of::<u8>()),
            DataType::I8 => Some(size_of::<i8>()),
            DataType::U16 => Some(size_of::<u16>()),
            DataType::I16 => Some(size_of::<i16>()),
            DataType::U32 => Some(size_of::<u32>()),
            DataType::I32 => Some(size_of::<i32>()),
            DataType::Bool => Some(4),
            DataType::F32 => Some(size_of::<f32>()),
            DataType::F64 => Some(size_of::<f64>()),
            DataType::Str => None,
            DataType::Array => None,
            DataType::Struct => None,
            DataType::I64 => Some(size_of::<i64>()),
            DataType::U64 => Some(size_of::<u64>()),
        }
    }
}

fn bank_16_view<'a>(endianness: Endianness) -> impl Parser<&'a [u8], BankView<'a>, ContextError> {
    seq! {BankView {
        name: take(4usize).map(|b: &[u8]| b.try_into().unwrap()),
        data_type: u16(endianness).verify_map(|n| DataType::try_from(n).ok()),
        data : length_take::<&[u8], _, _, _>(u16(endianness))
            .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0),
        _: take(data.len().next_multiple_of(8) - data.len()),
    }}
}

fn bank_32_view<'a>(endianness: Endianness) -> impl Parser<&'a [u8], BankView<'a>, ContextError> {
    seq! {BankView {
        name: take(4usize).map(|b: &[u8]| b.try_into().unwrap()),
        data_type: u32(endianness).verify_map(|n| DataType::try_from(n).ok()),
        data : length_take::<&[u8], _, _, _>(u32(endianness))
            .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0),
        _: take(data.len().next_multiple_of(8) - data.len()),
    }}
}

fn bank_32a_view<'a>(endianness: Endianness) -> impl Parser<&'a [u8], BankView<'a>, ContextError> {
    seq! {BankView{
        name: take(4usize).map(|b: &[u8]| b.try_into().unwrap()),
        data_type: u32(endianness).verify_map(|n| DataType::try_from(n).ok()),
        data: length_take::<&[u8], _, _, _>(terminated(u32(endianness), take(4usize)))
            .verify(|b: &[u8]| b.len() % data_type.size().unwrap_or(1) == 0),
        _: take(data.len().next_multiple_of(8) - data.len()),
    }}
}

fn event_view<'a>(endianness: Endianness) -> impl Parser<&'a [u8], EventView<'a>, ContextError> {
    seq! {EventView {
        id: u16(endianness),
        trigger_mask: u16(endianness),
        serial_number: u32(endianness),
        timestamp: u32(endianness),
        bank_views: u32(endianness)
            .verify(|&event_size| event_size >= 8)
            .flat_map(|event_size| {
                u32(endianness).verify(move |&banks_size| banks_size == event_size - 8)
            })
            .flat_map(|banks_size| {dispatch! {u32(endianness);
                1 => length_and_then(empty.value(banks_size), repeat_till(0.., bank_16_view(endianness), eof)),
                17 => length_and_then(empty.value(banks_size), repeat_till(0.., bank_32_view(endianness), eof)),
                49 => length_and_then(empty.value(banks_size), repeat_till(0.., bank_32a_view(endianness), eof)),
                _ => fail,
            }}).map(|(bank_views, _): (Vec<_>, _)| bank_views.into_boxed_slice()),
    }}
}

const BOR_ID: u16 = 0x8000;
const BOR_ID_SWAPPED: u16 = BOR_ID.swap_bytes();
const EOR_ID: u16 = 0x8001;
const MAGIC: u16 = 0x494D;

pub(crate) fn endianness(input: &mut &[u8]) -> ModalResult<Endianness> {
    dispatch! {le_u16;
        BOR_ID => empty.value(Endianness::Little),
        BOR_ID_SWAPPED => empty.value(Endianness::Big),
        _ => fail,
    }
    .parse_next(input)
}

pub(crate) fn file_view<'a>(input: &mut &'a [u8]) -> ModalResult<FileView<'a>> {
    let endianness = endianness
        .context(StrContext::Label("begin-of-run id"))
        .parse_next(input)?;

    seq! {FileView{
        _: u16(endianness).verify(|&magic| magic == MAGIC)
            .context(StrContext::Label("initial magic marker")),
        run_number: u32(endianness)
            .context(StrContext::Label("initial run number")),
        initial_timestamp: u32(endianness)
            .context(StrContext::Label("initial unix timestamp")),
        initial_odb: length_take(u32(endianness))
            .context(StrContext::Label("initial odb dump")),
        event_views: repeat(0.., event_view(endianness))
            .map(|event_views: Vec<_>| event_views.into_boxed_slice()),
        _: u16(endianness).verify(|&eor_id| eor_id == EOR_ID)
            .context(StrContext::Label("end-of-run id")),
        _: u16(endianness).verify(|&magic| magic == MAGIC)
            .context(StrContext::Label("final magic marker")),
        _: u32(endianness).verify(|&n| n == run_number)
            .context(StrContext::Label("final run number")),
        final_timestamp: u32(endianness)
            .context(StrContext::Label("final unix timestamp")),
        final_odb: length_take(u32(endianness))
            .context(StrContext::Label("final odb dump")),
    }}
    .parse_next(input)
}
