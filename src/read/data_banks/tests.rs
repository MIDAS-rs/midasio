use super::*;

#[test]
fn valid_le_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::Byte));
    assert_eq!(0, bank.data_slice().len());
    assert_eq!(0, bank.padding());

    let buffer = [66u8, 65, 78, 75, 3, 0, 1, 0, 100];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::U8));
    assert_eq!(1, bank.data_slice().len());
    assert_eq!(7, bank.padding());
    assert_eq!([100], bank.data_slice());
}

#[test]
fn invalid_le_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0];
    let bank = Bank16View::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 1];
    let bank = Bank16View::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 200, 75, 1, 0, 1, 0, 1];
    let bank = Bank16View::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::NonAsciiName)));

    let buffer = [66u8, 65, 78, 75, 4, 0, 1, 0, 100];
    let bank = Bank16View::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::IncompleteData)));

    let buffer = [66u8, 65, 78, 75, 20, 0, 1, 0, 1];
    let bank = Bank16View::try_from_le_bytes(&buffer);
    assert!(matches!(
        bank,
        Err(TryBankViewFromSliceError::UnknownDataType)
    ));
}

#[test]
fn valid_be_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 0, 1, 0, 0];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::Byte));
    assert_eq!(0, bank.data_slice().len());
    assert_eq!(0, bank.padding());

    let buffer = [66u8, 65, 78, 75, 0, 3, 0, 1, 100];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::U8));
    assert_eq!(1, bank.data_slice().len());
    assert_eq!(7, bank.padding());
    assert_eq!([100], bank.data_slice());
}

#[test]
fn invalid_be_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 0, 1, 0];
    let bank = Bank16View::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 78, 75, 0, 1, 0, 0, 1];
    let bank = Bank16View::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 200, 75, 0, 1, 0, 1, 1];
    let bank = Bank16View::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::NonAsciiName)));

    let buffer = [66u8, 65, 78, 75, 0, 4, 0, 1, 100];
    let bank = Bank16View::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::IncompleteData)));

    let buffer = [66u8, 65, 78, 75, 0, 20, 0, 1, 1];
    let bank = Bank16View::try_from_be_bytes(&buffer);
    assert!(matches!(
        bank,
        Err(TryBankViewFromSliceError::UnknownDataType)
    ));
}

#[test]
fn name_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 5, 0, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());

    let buffer = [48u8, 49, 110, 107, 1, 0, 5, 0, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("01nk", bank.name());

    let buffer = [66u8, 65, 78, 75, 0, 1, 0, 5, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());

    let buffer = [48u8, 49, 110, 107, 0, 1, 0, 5, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("01nk", bank.name());
}

#[test]
fn data_types_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 5, 0, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Byte));

    let buffer = [48u8, 49, 110, 107, 14, 0, 5, 0, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Struct));

    let buffer = [66u8, 65, 78, 75, 0, 10, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::F64));

    let buffer = [48u8, 49, 110, 107, 0, 8, 0, 5, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Bool));
}

#[test]
fn data_slices_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 5, 0, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!([1, 2, 3, 4, 5], bank.data_slice());

    let buffer = [48u8, 49, 110, 107, 14, 0, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!([1, 2, 3, 4, 5, 6, 7, 8], bank.data_slice());

    let buffer = [
        66u8, 65, 78, 75, 0, 10, 0, 16, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(
        [1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8],
        bank.data_slice()
    );

    let buffer = [48u8, 49, 110, 107, 0, 1, 0, 0];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.data_slice().len());
}

#[test]
fn padding_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 5, 0, 1, 2, 3, 4, 5];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(3, bank.padding());

    let buffer = [48u8, 49, 110, 107, 14, 0, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());

    let buffer = [
        66u8, 65, 78, 75, 0, 10, 0, 16, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());

    let buffer = [48u8, 49, 110, 107, 0, 1, 0, 0];
    let bank = Bank16View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());
}

#[test]
fn iterator_bank16_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 5, 0, 1, 1, 1, 1, 1];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(5, bank.into_iter().count());
    for num in &bank {
        let num = u8::from_le_bytes(num.try_into().unwrap());
        assert_eq!(1, num);
    }
    let buffer = [66u8, 65, 78, 75, 14, 0, 5, 0, 1, 1, 1, 1, 1];
    let bank = Bank16View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(5, bank.into_iter().count());
}

#[test]
fn valid_le_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 0, 0, 0, 0];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::Byte));
    assert_eq!(0, bank.data_slice().len());
    assert_eq!(0, bank.padding());

    let buffer = [66u8, 65, 78, 75, 3, 0, 0, 0, 1, 0, 0, 0, 100];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::U8));
    assert_eq!(1, bank.data_slice().len());
    assert_eq!(7, bank.padding());
    assert_eq!([100], bank.data_slice());
}

#[test]
fn invalid_le_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0];
    let bank = Bank32View::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 0, 0, 0, 0, 1];
    let bank = Bank32View::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 200, 75, 1, 0, 0, 0, 1, 0, 0, 0, 1];
    let bank = Bank32View::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::NonAsciiName)));

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 1, 0, 0, 0, 100];
    let bank = Bank32View::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::IncompleteData)));

    let buffer = [66u8, 65, 78, 75, 20, 0, 0, 0, 1, 0, 0, 0, 1];
    let bank = Bank32View::try_from_le_bytes(&buffer);
    assert!(matches!(
        bank,
        Err(TryBankViewFromSliceError::UnknownDataType)
    ));
}

#[test]
fn valid_be_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 0];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::Byte));
    assert_eq!(0, bank.data_slice().len());
    assert_eq!(0, bank.padding());

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 3, 0, 0, 0, 1, 100];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::U8));
    assert_eq!(1, bank.data_slice().len());
    assert_eq!(7, bank.padding());
    assert_eq!([100], bank.data_slice());
}

#[test]
fn invalid_be_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0];
    let bank = Bank32View::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    let bank = Bank32View::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 200, 75, 0, 0, 0, 1, 0, 0, 0, 1, 1];
    let bank = Bank32View::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::NonAsciiName)));

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 4, 0, 0, 0, 1, 100];
    let bank = Bank32View::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::IncompleteData)));

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 20, 0, 0, 0, 1, 1];
    let bank = Bank32View::try_from_be_bytes(&buffer);
    assert!(matches!(
        bank,
        Err(TryBankViewFromSliceError::UnknownDataType)
    ));
}

#[test]
fn name_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());

    let buffer = [48u8, 49, 110, 107, 1, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("01nk", bank.name());

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 5, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());

    let buffer = [48u8, 49, 110, 107, 0, 0, 0, 1, 0, 0, 0, 5, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("01nk", bank.name());
}

#[test]
fn data_types_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Byte));

    let buffer = [48u8, 49, 110, 107, 14, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Struct));

    let buffer = [
        66u8, 65, 78, 75, 0, 0, 0, 10, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::F64));

    let buffer = [48u8, 49, 110, 107, 0, 0, 0, 8, 0, 0, 0, 5, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Bool));
}

#[test]
fn data_slices_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!([1, 2, 3, 4, 5], bank.data_slice());

    let buffer = [
        48u8, 49, 110, 107, 14, 0, 0, 0, 8, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!([1, 2, 3, 4, 5, 6, 7, 8], bank.data_slice());

    let buffer = [
        66u8, 65, 78, 75, 0, 0, 0, 10, 0, 0, 0, 16, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(
        [1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8],
        bank.data_slice()
    );

    let buffer = [48u8, 49, 110, 107, 0, 0, 0, 1, 0, 0, 0, 0];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.data_slice().len());
}

#[test]
fn padding_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(3, bank.padding());

    let buffer = [
        48u8, 49, 110, 107, 14, 0, 0, 0, 8, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());

    let buffer = [
        66u8, 65, 78, 75, 0, 0, 0, 10, 0, 0, 0, 16, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());

    let buffer = [48u8, 49, 110, 107, 0, 0, 0, 1, 0, 0, 0, 0];
    let bank = Bank32View::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());
}

#[test]
fn iterator_bank32_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 1, 1, 1, 1, 1];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(5, bank.into_iter().count());
    for num in &bank {
        let num = u8::from_le_bytes(num.try_into().unwrap());
        assert_eq!(1, num);
    }
    let buffer = [66u8, 65, 78, 75, 14, 0, 0, 0, 5, 0, 0, 0, 1, 1, 1, 1, 1];
    let bank = Bank32View::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(5, bank.into_iter().count());
}

#[test]
fn valid_le_bank32a_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::Byte));
    assert_eq!(0, bank.data_slice().len());
    assert_eq!(0, bank.padding());

    let buffer = [66u8, 65, 78, 75, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 100];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::U8));
    assert_eq!(1, bank.data_slice().len());
    assert_eq!(7, bank.padding());
    assert_eq!([100], bank.data_slice());
}

#[test]
fn invalid_le_bank32a_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    let bank = Bank32AView::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 78, 75, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    let bank = Bank32AView::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 200, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1];
    let bank = Bank32AView::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::NonAsciiName)));

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 100];
    let bank = Bank32AView::try_from_le_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::IncompleteData)));

    let buffer = [66u8, 65, 78, 75, 20, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1];
    let bank = Bank32AView::try_from_le_bytes(&buffer);
    assert!(matches!(
        bank,
        Err(TryBankViewFromSliceError::UnknownDataType)
    ));
}

#[test]
fn valid_be_bank32a_views() {
    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::Byte));
    assert_eq!(0, bank.data_slice().len());
    assert_eq!(0, bank.padding());

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 100];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());
    assert!(matches!(bank.data_type(), DataType::U8));
    assert_eq!(1, bank.data_slice().len());
    assert_eq!(7, bank.padding());
    assert_eq!([100], bank.data_slice());
}

#[test]
fn invalid_be_bank32a_views() {
    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0];
    let bank = Bank32AView::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    let bank = Bank32AView::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::SizeMismatch)));

    let buffer = [66u8, 65, 200, 75, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    let bank = Bank32AView::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::NonAsciiName)));

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0, 0, 100];
    let bank = Bank32AView::try_from_be_bytes(&buffer);
    assert!(matches!(bank, Err(TryBankViewFromSliceError::IncompleteData)));

    let buffer = [66u8, 65, 78, 75, 0, 0, 0, 20, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    let bank = Bank32AView::try_from_be_bytes(&buffer);
    assert!(matches!(
        bank,
        Err(TryBankViewFromSliceError::UnknownDataType)
    ));
}

#[test]
fn name_bank32a_views() {
    let buffer = [
        66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());

    let buffer = [
        48u8, 49, 110, 107, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!("01nk", bank.name());

    let buffer = [
        66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("BANK", bank.name());

    let buffer = [
        48u8, 49, 110, 107, 0, 0, 0, 1, 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert_eq!("01nk", bank.name());
}

#[test]
fn data_types_bank32a_views() {
    let buffer = [
        66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Byte));

    let buffer = [
        48u8, 49, 110, 107, 14, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Struct));

    let buffer = [
        66u8, 65, 78, 75, 0, 0, 0, 10, 0, 0, 0, 8, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::F64));

    let buffer = [
        48u8, 49, 110, 107, 0, 0, 0, 8, 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert!(matches!(bank.data_type(), DataType::Bool));
}

#[test]
fn data_slices_bank32a_views() {
    let buffer = [
        66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!([1, 2, 3, 4, 5], bank.data_slice());

    let buffer = [
        48u8, 49, 110, 107, 14, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!([1, 2, 3, 4, 5, 6, 7, 8], bank.data_slice());

    let buffer = [
        66u8, 65, 78, 75, 0, 0, 0, 10, 0, 0, 0, 16, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4,
        5, 6, 7, 8,
    ];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(
        [1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8],
        bank.data_slice()
    );

    let buffer = [48u8, 49, 110, 107, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.data_slice().len());
}

#[test]
fn padding_bank32a_views() {
    let buffer = [
        66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(3, bank.padding());

    let buffer = [
        48u8, 49, 110, 107, 14, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());

    let buffer = [
        66u8, 65, 78, 75, 0, 0, 0, 10, 0, 0, 0, 16, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 1, 2, 3, 4,
        5, 6, 7, 8,
    ];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());

    let buffer = [48u8, 49, 110, 107, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
    let bank = Bank32AView::try_from_be_bytes(&buffer).unwrap();
    assert_eq!(0, bank.padding());
}

#[test]
fn iterator_bank32a_views() {
    let buffer = [
        66u8, 65, 78, 75, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(5, bank.into_iter().count());
    for num in &bank {
        let num = u8::from_le_bytes(num.try_into().unwrap());
        assert_eq!(1, num);
    }
    let buffer = [
        66u8, 65, 78, 75, 14, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1,
    ];
    let bank = Bank32AView::try_from_le_bytes(&buffer).unwrap();
    assert_eq!(5, bank.into_iter().count());
}

#[test]
fn name_bank_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 1, 0, 100];
    let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [
        66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255,
    ];
    let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer).unwrap());

    assert_eq!("BANK", bank_16.name());
    assert_eq!("BANK", bank_32.name());
    assert_eq!("BANK", bank_32a.name());
}

#[test]
fn data_types_bank_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 1, 0, 100];
    let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [
        66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255,
    ];
    let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer).unwrap());

    assert!(matches!(bank_16.data_type(), DataType::Byte));
    assert!(matches!(bank_32.data_type(), DataType::U16));
    assert!(matches!(bank_32a.data_type(), DataType::U16));
}

#[test]
fn slices_bank_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 1, 0, 100];
    let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [
        66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255,
    ];
    let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer).unwrap());

    assert_eq!([100], bank_16.data_slice());
    assert_eq!([100, 155, 200, 255], bank_32.data_slice());
    assert_eq!([100, 155, 200, 255], bank_32a.data_slice());
}

#[test]
fn padding_bank_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 1, 0, 100];
    let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [
        66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255,
    ];
    let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer).unwrap());

    assert_eq!(7, bank_16.padding());
    assert_eq!(4, bank_32.padding());
    assert_eq!(4, bank_32a.padding());
}

#[test]
fn is_b16_bank_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 1, 0, 100];
    let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [
        66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255,
    ];
    let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer).unwrap());

    assert!(bank_16.is_b16());
    assert!(!bank_32.is_b16());
    assert!(!bank_32a.is_b16());
}

#[test]
fn is_b32_bank_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 1, 0, 100];
    let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [
        66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255,
    ];
    let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer).unwrap());

    assert!(!bank_16.is_b32());
    assert!(bank_32.is_b32());
    assert!(!bank_32a.is_b32());
}

#[test]
fn is_b32a_bank_views() {
    let buffer = [66u8, 65, 78, 75, 1, 0, 1, 0, 100];
    let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [
        66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255,
    ];
    let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer).unwrap());

    assert!(!bank_16.is_b32a());
    assert!(!bank_32.is_b32a());
    assert!(bank_32a.is_b32a());
}

#[test]
fn iterator_bank_views() {
    let buffer = [66u8, 65, 78, 75, 14, 0, 1, 0, 100];
    let bank_16 = BankView::B16(Bank16View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 100, 155, 200, 255];
    let bank_32 = BankView::B32(Bank32View::try_from_le_bytes(&buffer).unwrap());

    let buffer = [
        66u8, 65, 78, 75, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 100, 155, 200, 255,
    ];
    let bank_32a = BankView::B32A(Bank32AView::try_from_le_bytes(&buffer).unwrap());

    assert_eq!(1, bank_16.into_iter().count());
    assert_eq!(2, bank_32.into_iter().count());
    assert_eq!(2, bank_32a.into_iter().count());
}
