use super::*;

#[test]
fn type_size() {
    assert_eq!(Type::Byte.size().unwrap(), 1);
    assert_eq!(Type::I8.size().unwrap(), 1);
    assert_eq!(Type::U8.size().unwrap(), 1);
    assert_eq!(Type::U16.size().unwrap(), 2);
    assert_eq!(Type::I16.size().unwrap(), 2);
    assert_eq!(Type::U32.size().unwrap(), 4);
    assert_eq!(Type::I32.size().unwrap(), 4);
    assert_eq!(Type::Bool.size().unwrap(), 1);
    assert_eq!(Type::F32.size().unwrap(), 4);
    assert_eq!(Type::F64.size().unwrap(), 8);
    assert!(Type::Struct.size().is_none());
}

#[test]
fn type_try_from_u16() {
    assert!(matches!(Type::try_from(1u16).unwrap(), Type::Byte));
    assert!(matches!(Type::try_from(2u16).unwrap(), Type::I8));
    assert!(matches!(Type::try_from(3u16).unwrap(), Type::U8));
    assert!(matches!(Type::try_from(4u16).unwrap(), Type::U16));
    assert!(matches!(Type::try_from(5u16).unwrap(), Type::I16));
    assert!(matches!(Type::try_from(6u16).unwrap(), Type::U32));
    assert!(matches!(Type::try_from(7u16).unwrap(), Type::I32));
    assert!(matches!(Type::try_from(8u16).unwrap(), Type::Bool));
    assert!(matches!(Type::try_from(9u16).unwrap(), Type::F32));
    assert!(matches!(Type::try_from(10u16).unwrap(), Type::F64));
    assert!(matches!(Type::try_from(14u16).unwrap(), Type::Struct));
    assert!(matches!(
        Type::try_from(100u16).unwrap_err(),
        TryTypeFromUnsignedError
    ));
}

#[test]
fn type_try_from_u32() {
    assert!(matches!(Type::try_from(1u32).unwrap(), Type::Byte));
    assert!(matches!(Type::try_from(2u32).unwrap(), Type::I8));
    assert!(matches!(Type::try_from(3u32).unwrap(), Type::U8));
    assert!(matches!(Type::try_from(4u32).unwrap(), Type::U16));
    assert!(matches!(Type::try_from(5u32).unwrap(), Type::I16));
    assert!(matches!(Type::try_from(6u32).unwrap(), Type::U32));
    assert!(matches!(Type::try_from(7u32).unwrap(), Type::I32));
    assert!(matches!(Type::try_from(8u32).unwrap(), Type::Bool));
    assert!(matches!(Type::try_from(9u32).unwrap(), Type::F32));
    assert!(matches!(Type::try_from(10u32).unwrap(), Type::F64));
    assert!(matches!(Type::try_from(14u32).unwrap(), Type::Struct));
    assert!(matches!(
        Type::try_from(100u32).unwrap_err(),
        TryTypeFromUnsignedError
    ));
}
