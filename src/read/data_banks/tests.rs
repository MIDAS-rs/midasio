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

    let buffer = [66u8, 65, 78, 75, 0, 20, 0, 1, 1];
    let bank = Bank16View::try_from_be_bytes(&buffer);
    assert!(matches!(
        bank,
        Err(TryBankViewFromSliceError::UnknownDataType)
    ));
}
