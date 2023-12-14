use super::*;

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
        .chain(bank_1)
        .chain(padding1)
        .chain(bank_2)
        .chain(padding2)
        .collect();
    let event = EventView::try_from_le_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.timestamp());

    for bank in event {
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
        .chain(bank_1)
        .chain(padding1)
        .chain(bank_2)
        .chain(padding2)
        .collect();
    let event = EventView::try_from_be_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.timestamp());

    for bank in event {
        assert_eq!("BANK", bank.name());
        assert_eq!([255], bank.data_slice());
    }
}

#[test]
fn invalid_short_b16_be_event_view() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 1,
    ];

    let event: Vec<u8> = header.into_iter().collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(event.is_err());
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
        .chain(bank_1)
        .chain(padding1)
        .chain(bank_2)
        .chain(padding2)
        .collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(event.is_err());
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
        .chain(bank_1)
        .chain(padding1)
        .chain(bank_2)
        .chain(padding2)
        .collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(event.is_err());
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
        .chain(bank_1)
        .chain(padding1)
        .chain(bank_2)
        .chain(padding2)
        .collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(event.is_err());
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
        .chain(bank_1)
        .chain(padding1)
        .chain(bank_2)
        .chain(padding2)
        .collect();
    let event = EventView::try_from_be_bytes(&event);

    assert!(event.is_err());
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
        .chain(bank_1)
        .chain(padding1)
        .chain(bank_2)
        .chain(padding2)
        .collect();
    let event = EventView::try_from_le_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.timestamp());

    for bank in event {
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
        .chain(bank_1)
        .chain(padding1)
        .chain(bank_2)
        .chain(padding2)
        .collect();
    let event = EventView::try_from_le_bytes(&event).unwrap();

    assert_eq!(1, event.id());
    assert_eq!(2, event.trigger_mask());
    assert_eq!(3, event.serial_number());
    assert_eq!(4, event.timestamp());

    for bank in event {
        assert_eq!("BANK", bank.name());
        assert_eq!([255], bank.data_slice());
    }
}
