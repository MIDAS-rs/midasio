use super::*;

#[test]
fn type_size() {
    assert_eq!(DataType::Byte.size().unwrap(), 1);
    assert_eq!(DataType::I8.size().unwrap(), 1);
    assert_eq!(DataType::U8.size().unwrap(), 1);
    assert_eq!(DataType::U16.size().unwrap(), 2);
    assert_eq!(DataType::I16.size().unwrap(), 2);
    assert_eq!(DataType::U32.size().unwrap(), 4);
    assert_eq!(DataType::I32.size().unwrap(), 4);
    assert_eq!(DataType::Bool.size().unwrap(), 1);
    assert_eq!(DataType::F32.size().unwrap(), 4);
    assert_eq!(DataType::F64.size().unwrap(), 8);
    assert!(DataType::Struct.size().is_none());
}

#[test]
fn type_try_from_u8() {
    assert!(matches!(DataType::try_from(1u8).unwrap(), DataType::Byte));
    assert!(matches!(DataType::try_from(2u8).unwrap(), DataType::I8));
    assert!(matches!(DataType::try_from(3u8).unwrap(), DataType::U8));
    assert!(matches!(DataType::try_from(4u8).unwrap(), DataType::U16));
    assert!(matches!(DataType::try_from(5u8).unwrap(), DataType::I16));
    assert!(matches!(DataType::try_from(6u8).unwrap(), DataType::U32));
    assert!(matches!(DataType::try_from(7u8).unwrap(), DataType::I32));
    assert!(matches!(DataType::try_from(8u8).unwrap(), DataType::Bool));
    assert!(matches!(DataType::try_from(9u8).unwrap(), DataType::F32));
    assert!(matches!(DataType::try_from(10u8).unwrap(), DataType::F64));
    assert!(matches!(
        DataType::try_from(14u8).unwrap(),
        DataType::Struct
    ));
    assert!(matches!(
        DataType::try_from(100u8).unwrap_err(),
        TryDataTypeFromUnsignedError
    ));
}

#[test]
fn type_try_from_u16() {
    assert!(matches!(DataType::try_from(1u16).unwrap(), DataType::Byte));
    assert!(matches!(DataType::try_from(2u16).unwrap(), DataType::I8));
    assert!(matches!(DataType::try_from(3u16).unwrap(), DataType::U8));
    assert!(matches!(DataType::try_from(4u16).unwrap(), DataType::U16));
    assert!(matches!(DataType::try_from(5u16).unwrap(), DataType::I16));
    assert!(matches!(DataType::try_from(6u16).unwrap(), DataType::U32));
    assert!(matches!(DataType::try_from(7u16).unwrap(), DataType::I32));
    assert!(matches!(DataType::try_from(8u16).unwrap(), DataType::Bool));
    assert!(matches!(DataType::try_from(9u16).unwrap(), DataType::F32));
    assert!(matches!(DataType::try_from(10u16).unwrap(), DataType::F64));
    assert!(matches!(
        DataType::try_from(14u16).unwrap(),
        DataType::Struct
    ));
    assert!(matches!(
        DataType::try_from(100u16).unwrap_err(),
        TryDataTypeFromUnsignedError
    ));
}

#[test]
fn type_try_from_u32() {
    assert!(matches!(DataType::try_from(1u32).unwrap(), DataType::Byte));
    assert!(matches!(DataType::try_from(2u32).unwrap(), DataType::I8));
    assert!(matches!(DataType::try_from(3u32).unwrap(), DataType::U8));
    assert!(matches!(DataType::try_from(4u32).unwrap(), DataType::U16));
    assert!(matches!(DataType::try_from(5u32).unwrap(), DataType::I16));
    assert!(matches!(DataType::try_from(6u32).unwrap(), DataType::U32));
    assert!(matches!(DataType::try_from(7u32).unwrap(), DataType::I32));
    assert!(matches!(DataType::try_from(8u32).unwrap(), DataType::Bool));
    assert!(matches!(DataType::try_from(9u32).unwrap(), DataType::F32));
    assert!(matches!(DataType::try_from(10u32).unwrap(), DataType::F64));
    assert!(matches!(
        DataType::try_from(14u32).unwrap(),
        DataType::Struct
    ));
    assert!(matches!(
        DataType::try_from(100u32).unwrap_err(),
        TryDataTypeFromUnsignedError
    ));
}

#[test]
fn type_try_from_u64() {
    assert!(matches!(DataType::try_from(1u64).unwrap(), DataType::Byte));
    assert!(matches!(DataType::try_from(2u64).unwrap(), DataType::I8));
    assert!(matches!(DataType::try_from(3u64).unwrap(), DataType::U8));
    assert!(matches!(DataType::try_from(4u64).unwrap(), DataType::U16));
    assert!(matches!(DataType::try_from(5u64).unwrap(), DataType::I16));
    assert!(matches!(DataType::try_from(6u64).unwrap(), DataType::U32));
    assert!(matches!(DataType::try_from(7u64).unwrap(), DataType::I32));
    assert!(matches!(DataType::try_from(8u64).unwrap(), DataType::Bool));
    assert!(matches!(DataType::try_from(9u64).unwrap(), DataType::F32));
    assert!(matches!(DataType::try_from(10u64).unwrap(), DataType::F64));
    assert!(matches!(
        DataType::try_from(14u64).unwrap(),
        DataType::Struct
    ));
    assert!(matches!(
        DataType::try_from(100u64).unwrap_err(),
        TryDataTypeFromUnsignedError
    ));
}

#[test]
fn type_try_from_u128() {
    assert!(matches!(DataType::try_from(1u128).unwrap(), DataType::Byte));
    assert!(matches!(DataType::try_from(2u128).unwrap(), DataType::I8));
    assert!(matches!(DataType::try_from(3u128).unwrap(), DataType::U8));
    assert!(matches!(DataType::try_from(4u128).unwrap(), DataType::U16));
    assert!(matches!(DataType::try_from(5u128).unwrap(), DataType::I16));
    assert!(matches!(DataType::try_from(6u128).unwrap(), DataType::U32));
    assert!(matches!(DataType::try_from(7u128).unwrap(), DataType::I32));
    assert!(matches!(DataType::try_from(8u128).unwrap(), DataType::Bool));
    assert!(matches!(DataType::try_from(9u128).unwrap(), DataType::F32));
    assert!(matches!(DataType::try_from(10u128).unwrap(), DataType::F64));
    assert!(matches!(
        DataType::try_from(14u128).unwrap(),
        DataType::Struct
    ));
    assert!(matches!(
        DataType::try_from(100u128).unwrap_err(),
        TryDataTypeFromUnsignedError
    ));
}
