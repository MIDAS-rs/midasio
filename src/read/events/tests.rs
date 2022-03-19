use super::*;

#[test]
fn empty_le_bank16views() {
    let bank_16 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let bank16views = Bank16Views::from_le_bytes(&bank_16);

    assert_eq!(0, bank16views.count());
}

#[test]
fn single_exact_le_bank16views() {
    let bank_16 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    let bank16views = Bank16Views::from_le_bytes(&banks);

    assert_eq!(1, bank16views.count());

    let mut bank16views = Bank16Views::from_le_bytes(&banks);
    assert_eq!("BANK", bank16views.next().unwrap().name());
    assert_eq!(0, bank16views.remainder().len());
}

#[test]
fn single_inexact_le_bank16views() {
    let bank_16 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    let bank16views = Bank16Views::from_le_bytes(&banks);

    assert_eq!(1, bank16views.count());

    let mut bank16views = Bank16Views::from_le_bytes(&banks);
    assert_eq!("BANK", bank16views.next().unwrap().name());
    assert_eq!(8, bank16views.remainder().len());
}

#[test]
fn more_than_one_inexact_le_bank16views() {
    let bank1 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [49u8, 50, 51, 52, 1, 0, 3, 0, 255, 255, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank16views = Bank16Views::from_le_bytes(&banks);

    assert_eq!(2, bank16views.count());

    let mut bank16views = Bank16Views::from_le_bytes(&banks);
    assert_eq!("BANK", bank16views.next().unwrap().name());
    assert_eq!("1234", bank16views.next().unwrap().name());
    assert_eq!(10, bank16views.remainder().len());
}

#[test]
fn more_than_one_exact_le_bank16views() {
    let bank1 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [49u8, 50, 51, 52, 1, 0, 3, 0, 255, 255, 255];
    let padding2 = [0u8, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank16views = Bank16Views::from_le_bytes(&banks);

    assert_eq!(2, bank16views.count());

    let mut bank16views = Bank16Views::from_le_bytes(&banks);
    assert_eq!("BANK", bank16views.next().unwrap().name());
    assert_eq!("1234", bank16views.next().unwrap().name());
    assert_eq!(0, bank16views.remainder().len());
}

#[test]
fn empty_be_bank16views() {
    let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let bank16views = Bank16Views::from_be_bytes(&bank_16);

    assert_eq!(0, bank16views.count());
}

#[test]
fn single_exact_be_bank16views() {
    let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    let bank16views = Bank16Views::from_be_bytes(&banks);

    assert_eq!(1, bank16views.count());

    let mut bank16views = Bank16Views::from_be_bytes(&banks);
    assert_eq!("BANK", bank16views.next().unwrap().name());
    assert_eq!(0, bank16views.remainder().len());
}

#[test]
fn single_inexact_be_bank16views() {
    let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    let bank16views = Bank16Views::from_be_bytes(&banks);

    assert_eq!(1, bank16views.count());

    let mut bank16views = Bank16Views::from_be_bytes(&banks);
    assert_eq!("BANK", bank16views.next().unwrap().name());
    assert_eq!(8, bank16views.remainder().len());
}

#[test]
fn more_than_one_inexact_be_bank16views() {
    let bank1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [49u8, 50, 51, 52, 0, 1, 0, 3, 255, 255, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank16views = Bank16Views::from_be_bytes(&banks);

    assert_eq!(2, bank16views.count());

    let mut bank16views = Bank16Views::from_be_bytes(&banks);
    assert_eq!("BANK", bank16views.next().unwrap().name());
    assert_eq!("1234", bank16views.next().unwrap().name());
    assert_eq!(10, bank16views.remainder().len());
}

#[test]
fn more_than_one_exact_be_bank16views() {
    let bank1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [49u8, 50, 51, 52, 0, 1, 0, 3, 255, 255, 255];
    let padding2 = [0u8, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank16views = Bank16Views::from_be_bytes(&banks);

    assert_eq!(2, bank16views.count());

    let mut bank16views = Bank16Views::from_be_bytes(&banks);
    assert_eq!("BANK", bank16views.next().unwrap().name());
    assert_eq!("1234", bank16views.next().unwrap().name());
    assert_eq!(0, bank16views.remainder().len());
}

#[test]
fn empty_le_bank32views() {
    let bank_32 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let bank32views = Bank32Views::from_le_bytes(&bank_32);

    assert_eq!(0, bank32views.count());
}

#[test]
fn single_exact_le_bank32views() {
    let bank_32 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
    let bank32views = Bank32Views::from_le_bytes(&banks);

    assert_eq!(1, bank32views.count());

    let mut bank32views = Bank32Views::from_le_bytes(&banks);
    assert_eq!("BANK", bank32views.next().unwrap().name());
    assert_eq!(0, bank32views.remainder().len());
}

#[test]
fn single_inexact_le_bank32views() {
    let bank_32 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
    let bank32views = Bank32Views::from_le_bytes(&banks);

    assert_eq!(1, bank32views.count());

    let mut bank32views = Bank32Views::from_le_bytes(&banks);
    assert_eq!("BANK", bank32views.next().unwrap().name());
    assert_eq!(8, bank32views.remainder().len());
}

#[test]
fn more_than_one_inexact_le_bank32views() {
    let bank1 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [49u8, 50, 51, 52, 1, 0, 0, 0, 3, 0, 0, 0, 255, 255, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank32views = Bank32Views::from_le_bytes(&banks);

    assert_eq!(2, bank32views.count());

    let mut bank32views = Bank32Views::from_le_bytes(&banks);
    assert_eq!("BANK", bank32views.next().unwrap().name());
    assert_eq!("1234", bank32views.next().unwrap().name());
    assert_eq!(10, bank32views.remainder().len());
}

#[test]
fn more_than_one_exact_le_bank32views() {
    let bank1 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [49u8, 50, 51, 52, 1, 0, 0, 0, 3, 0, 0, 0, 255, 255, 255];
    let padding2 = [0u8, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank32views = Bank32Views::from_le_bytes(&banks);

    assert_eq!(2, bank32views.count());

    let mut bank32views = Bank32Views::from_le_bytes(&banks);
    assert_eq!("BANK", bank32views.next().unwrap().name());
    assert_eq!("1234", bank32views.next().unwrap().name());
    assert_eq!(0, bank32views.remainder().len());
}

#[test]
fn empty_be_bank32views() {
    let bank_32 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 255];
    let bank32views = Bank32Views::from_be_bytes(&bank_32);

    assert_eq!(0, bank32views.count());
}

#[test]
fn single_exact_be_bank32views() {
    let bank_32 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
    let bank32views = Bank32Views::from_be_bytes(&banks);

    assert_eq!(1, bank32views.count());

    let mut bank32views = Bank32Views::from_be_bytes(&banks);
    assert_eq!("BANK", bank32views.next().unwrap().name());
    assert_eq!(0, bank32views.remainder().len());
}

#[test]
fn single_inexact_be_bank32views() {
    let bank_32 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
    let bank32views = Bank32Views::from_be_bytes(&banks);

    assert_eq!(1, bank32views.count());

    let mut bank32views = Bank32Views::from_be_bytes(&banks);
    assert_eq!("BANK", bank32views.next().unwrap().name());
    assert_eq!(8, bank32views.remainder().len());
}

#[test]
fn more_than_one_inexact_be_bank32views() {
    let bank1 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [49u8, 50, 51, 52, 0, 0, 0, 1, 0, 0, 0, 3, 255, 255, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank32views = Bank32Views::from_be_bytes(&banks);

    assert_eq!(2, bank32views.count());

    let mut bank32views = Bank32Views::from_be_bytes(&banks);
    assert_eq!("BANK", bank32views.next().unwrap().name());
    assert_eq!("1234", bank32views.next().unwrap().name());
    assert_eq!(10, bank32views.remainder().len());
}

#[test]
fn more_than_one_exact_be_bank32views() {
    let bank1 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [49u8, 50, 51, 52, 0, 0, 0, 1, 0, 0, 0, 3, 255, 255, 255];
    let padding2 = [0u8, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank32views = Bank32Views::from_be_bytes(&banks);

    assert_eq!(2, bank32views.count());

    let mut bank32views = Bank32Views::from_be_bytes(&banks);
    assert_eq!("BANK", bank32views.next().unwrap().name());
    assert_eq!("1234", bank32views.next().unwrap().name());
    assert_eq!(0, bank32views.remainder().len());
}

#[test]
fn empty_le_bank32_aviews() {
    let bank_32_a = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 255];
    let bank32_aviews = Bank32AViews::from_le_bytes(&bank_32_a);

    assert_eq!(0, bank32_aviews.count());
}

#[test]
fn single_exact_le_bank32_aviews() {
    let bank_32_a = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_32_a.into_iter().chain(padding.into_iter()).collect();
    let bank32_aviews = Bank32AViews::from_le_bytes(&banks);

    assert_eq!(1, bank32_aviews.count());

    let mut bank32_aviews = Bank32AViews::from_le_bytes(&banks);
    assert_eq!("BANK", bank32_aviews.next().unwrap().name());
    assert_eq!(0, bank32_aviews.remainder().len());
}

#[test]
fn single_inexact_le_bank32_aviews() {
    let bank_32_a = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_32_a.into_iter().chain(padding.into_iter()).collect();
    let bank32_aviews = Bank32AViews::from_le_bytes(&banks);

    assert_eq!(1, bank32_aviews.count());

    let mut bank32_aviews = Bank32AViews::from_le_bytes(&banks);
    assert_eq!("BANK", bank32_aviews.next().unwrap().name());
    assert_eq!(8, bank32_aviews.remainder().len());
}

#[test]
fn more_than_one_inexact_le_bank32_aviews() {
    let bank1 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [
        49u8, 50, 51, 52, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255,
    ];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank32_aviews = Bank32AViews::from_le_bytes(&banks);

    assert_eq!(2, bank32_aviews.count());

    let mut bank32_aviews = Bank32AViews::from_le_bytes(&banks);
    assert_eq!("BANK", bank32_aviews.next().unwrap().name());
    assert_eq!("1234", bank32_aviews.next().unwrap().name());
    assert_eq!(10, bank32_aviews.remainder().len());
}

#[test]
fn more_than_one_exact_le_bank32_aviews() {
    let bank1 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [
        49u8, 50, 51, 52, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255,
    ];
    let padding2 = [0u8, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank32_aviews = Bank32AViews::from_le_bytes(&banks);

    assert_eq!(2, bank32_aviews.count());

    let mut bank32_aviews = Bank32AViews::from_le_bytes(&banks);
    assert_eq!("BANK", bank32_aviews.next().unwrap().name());
    assert_eq!("1234", bank32_aviews.next().unwrap().name());
    assert_eq!(0, bank32_aviews.remainder().len());
}

#[test]
fn empty_be_bank32_aviews() {
    let bank_32_a = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 255];
    let bank32_aviews = Bank32AViews::from_be_bytes(&bank_32_a);

    assert_eq!(0, bank32_aviews.count());
}

#[test]
fn single_exact_be_bank32_aviews() {
    let bank_32_a = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_32_a.into_iter().chain(padding.into_iter()).collect();
    let bank32_aviews = Bank32AViews::from_be_bytes(&banks);

    assert_eq!(1, bank32_aviews.count());

    let mut bank32_aviews = Bank32AViews::from_be_bytes(&banks);
    assert_eq!("BANK", bank32_aviews.next().unwrap().name());
    assert_eq!(0, bank32_aviews.remainder().len());
}

#[test]
fn single_inexact_be_bank32_aviews() {
    let bank_32_a = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 255];
    let padding = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank_32_a.into_iter().chain(padding.into_iter()).collect();
    let bank32_aviews = Bank32AViews::from_be_bytes(&banks);

    assert_eq!(1, bank32_aviews.count());

    let mut bank32_aviews = Bank32AViews::from_be_bytes(&banks);
    assert_eq!("BANK", bank32_aviews.next().unwrap().name());
    assert_eq!(8, bank32_aviews.remainder().len());
}

#[test]
fn more_than_one_inexact_be_bank32_aviews() {
    let bank1 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [
        49u8, 50, 51, 52, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 255, 255, 255,
    ];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank32_aviews = Bank32AViews::from_be_bytes(&banks);

    assert_eq!(2, bank32_aviews.count());

    let mut bank32_aviews = Bank32AViews::from_be_bytes(&banks);
    assert_eq!("BANK", bank32_aviews.next().unwrap().name());
    assert_eq!("1234", bank32_aviews.next().unwrap().name());
    assert_eq!(10, bank32_aviews.remainder().len());
}

#[test]
fn more_than_one_exact_be_bank32_aviews() {
    let bank1 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank2 = [
        49u8, 50, 51, 52, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 255, 255, 255,
    ];
    let padding2 = [0u8, 0, 0, 0, 0];
    let banks: Vec<u8> = bank1
        .into_iter()
        .chain(padding1.into_iter())
        .chain(bank2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let bank32_aviews = Bank32AViews::from_be_bytes(&banks);

    assert_eq!(2, bank32_aviews.count());

    let mut bank32_aviews = Bank32AViews::from_be_bytes(&banks);
    assert_eq!("BANK", bank32_aviews.next().unwrap().name());
    assert_eq!("1234", bank32_aviews.next().unwrap().name());
    assert_eq!(0, bank32_aviews.remainder().len());
}

#[test]
fn valid_bank_views_16() {
    let bank_16 = [66u8, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6];
    let padding = [0u8, 0, 1, 1];
    let banks: Vec<u8> = bank_16.into_iter().chain(padding.into_iter()).collect();
    let mut banks = BankViews::B16(Bank16Views::from_be_bytes(&banks));

    assert_eq!(
        [66, 65, 78, 75, 0, 1, 0, 6, 1, 2, 3, 4, 5, 6, 0, 0, 1, 1],
        banks.remainder()
    );

    banks.next();
    assert_eq!([1, 1], banks.remainder());
}

#[test]
fn valid_bank_views_32() {
    let bank_32 = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 1, 2, 3, 4, 5, 6];
    let padding = [0u8, 0, 1, 1];
    let banks: Vec<u8> = bank_32.into_iter().chain(padding.into_iter()).collect();
    let mut banks = BankViews::B32(Bank32Views::from_be_bytes(&banks));

    assert_eq!(
        [66, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 1, 2, 3, 4, 5, 6, 0, 0, 1, 1],
        banks.remainder()
    );

    banks.next();
    assert_eq!([1, 1], banks.remainder());
}

#[test]
fn valid_bank_views_32a() {
    let bank_32a = [
        66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6,
    ];
    let padding = [0u8, 0, 1, 1];
    let banks: Vec<u8> = bank_32a.into_iter().chain(padding.into_iter()).collect();
    let mut banks = BankViews::B32A(Bank32AViews::from_be_bytes(&banks));

    assert_eq!(
        [66, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 0, 0, 1, 1],
        banks.remainder()
    );

    banks.next();
    assert_eq!([1, 1], banks.remainder());
}

#[test]
fn valid_b16_le_event_view() {
    let header = [
        1u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1, 0, 0, 0,
    ];
    let bank_1 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let event = EventView::try_from_le_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.time_stamp());
    assert_eq!(1, event.flags());

    for bank in &event {
        assert_eq!("BANK", bank.name());
        assert_eq!([255], bank.data_slice());
    }
}

#[test]
fn valid_b16_be_event_view() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank_1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let event = EventView::try_from_be_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.time_stamp());
    assert_eq!(1, event.flags());

    for bank in &event {
        assert_eq!("BANK", bank.name());
        assert_eq!([255], bank.data_slice());
    }
}

#[test]
fn valid_empty_b16_be_event_view() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 1,
    ];

    let event: Vec<u8> = header.into_iter().collect();
    let event = EventView::try_from_be_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.time_stamp());
    assert_eq!(1, event.flags());
}

#[test]
fn invalid_short_b16_be_event_view() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0,
    ];

    let event: Vec<u8> = header.into_iter().collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(matches!(
        event,
        Err(TryEventViewFromSliceError::SizeMismatch)
    ));
}

#[test]
fn invalid_internal_mismatch_b16_be_event_view() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 39, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank_1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(matches!(
        event,
        Err(TryEventViewFromSliceError::EventAndBanksMismatch)
    ));
}

#[test]
fn invalid_size_mismatch_b16_be_event_view() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank_1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(matches!(
        event,
        Err(TryEventViewFromSliceError::SizeMismatch)
    ));
}

#[test]
fn invalid_flag_b16_be_event_view() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 2,
    ];
    let bank_1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(matches!(
        event,
        Err(TryEventViewFromSliceError::UnknownFlag)
    ));
}

#[test]
fn invalid_bad_bank_b16_be_event_view() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 17,
    ];
    let bank_1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(matches!(event, Err(TryEventViewFromSliceError::BadBank)));
}

#[test]
fn valid_b32_le_event_view() {
    let header = [
        1u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 48, 0, 0, 0, 40, 0, 0, 0, 17, 0, 0, 0,
    ];
    let bank_1 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let event = EventView::try_from_le_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.time_stamp());
    assert_eq!(17, event.flags());

    for bank in &event {
        assert_eq!("BANK", bank.name());
        assert_eq!([255], bank.data_slice());
    }
}

#[test]
fn valid_b32a_le_event_view() {
    let header = [
        1u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 56, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0,
    ];
    let bank_1 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();
    let event = EventView::try_from_le_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.time_stamp());
    assert_eq!(49, event.flags());

    for bank in &event {
        assert_eq!("BANK", bank.name());
        assert_eq!([255], bank.data_slice());
    }
}
