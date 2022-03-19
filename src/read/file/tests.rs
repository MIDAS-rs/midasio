use super::*;

#[test]
fn empty_le_event_views() {
    let header = [
        1u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1, 0, 0, 0,
    ];
    let bank_1 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();

    let event_views = EventViews::from_le_bytes(&event);

    assert_eq!(0, event_views.count());
}

#[test]
fn single_exact_le_event_views() {
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

    let event_views = EventViews::from_le_bytes(&event);
    assert_eq!(1, event_views.count());

    let mut event_views = EventViews::from_le_bytes(&event);
    assert_eq!(1, event_views.next().unwrap().id());
    assert_eq!(0, event_views.remainder().len());
}

#[test]
fn single_inexact_le_event_views() {
    let header = [
        1u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1, 0, 0, 0,
    ];
    let bank_1 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];
    let extra = [0u8, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .chain(extra.into_iter())
        .collect();

    let event_views = EventViews::from_le_bytes(&event);
    assert_eq!(1, event_views.count());

    let mut event_views = EventViews::from_le_bytes(&event);
    assert_eq!(1, event_views.next().unwrap().id());
    assert_eq!(6, event_views.remainder().len());
}

#[test]
fn multiple_exact_le_event_views() {
    let header_a = [
        1u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1, 0, 0, 0,
    ];
    let bank_1a = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding1a = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2a = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding2a = [0u8, 0, 0, 0, 0, 0, 0];

    let header_b = [
        5u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 48, 0, 0, 0, 40, 0, 0, 0, 17, 0, 0, 0,
    ];
    let bank_1b = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding1b = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2b = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding2b = [0u8, 0, 0, 0, 0, 0, 0];
    let event: Vec<u8> = header_a
        .into_iter()
        .chain(bank_1a.into_iter())
        .chain(padding1a.into_iter())
        .chain(bank_2a.into_iter())
        .chain(padding2a.into_iter())
        .chain(header_b.into_iter())
        .chain(bank_1b.into_iter())
        .chain(padding1b.into_iter())
        .chain(bank_2b.into_iter())
        .chain(padding2b.into_iter())
        .collect();

    let event_views = EventViews::from_le_bytes(&event);
    assert_eq!(2, event_views.count());

    let mut event_views = EventViews::from_le_bytes(&event);
    assert_eq!(1, event_views.next().unwrap().id());
    assert_eq!(5, event_views.next().unwrap().id());
    assert_eq!(0, event_views.remainder().len());
}

#[test]
fn multiple_inexact_le_event_views() {
    let header_a = [
        1u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1, 0, 0, 0,
    ];
    let bank_1a = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding1a = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2a = [66u8, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding2a = [0u8, 0, 0, 0, 0, 0, 0];

    let header_b = [
        5u8, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 48, 0, 0, 0, 40, 0, 0, 0, 17, 0, 0, 0,
    ];
    let bank_1b = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding1b = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2b = [66u8, 65, 78, 75, 1, 0, 0, 0, 1, 0, 0, 0, 255];
    let padding2b = [0u8, 0, 0, 0, 0, 0, 0];
    let extra = [0u8];
    let event: Vec<u8> = header_a
        .into_iter()
        .chain(bank_1a.into_iter())
        .chain(padding1a.into_iter())
        .chain(bank_2a.into_iter())
        .chain(padding2a.into_iter())
        .chain(header_b.into_iter())
        .chain(bank_1b.into_iter())
        .chain(padding1b.into_iter())
        .chain(bank_2b.into_iter())
        .chain(padding2b.into_iter())
        .chain(extra.into_iter())
        .collect();

    let event_views = EventViews::from_le_bytes(&event);
    assert_eq!(2, event_views.count());

    let mut event_views = EventViews::from_le_bytes(&event);
    assert_eq!(1, event_views.next().unwrap().id());
    assert_eq!(5, event_views.next().unwrap().id());
    assert_eq!(1, event_views.remainder().len());
}

#[test]
fn empty_be_event_views() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank_1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .collect();

    let event_views = EventViews::from_be_bytes(&event);

    assert_eq!(0, event_views.count());
}

#[test]
fn single_exact_be_event_views() {
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

    let event_views = EventViews::from_be_bytes(&event);
    assert_eq!(1, event_views.count());

    let mut event_views = EventViews::from_be_bytes(&event);
    assert_eq!(1, event_views.next().unwrap().id());
    assert_eq!(0, event_views.remainder().len());
}

#[test]
fn single_inexact_be_event_views() {
    let header = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank_1 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1 = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2 = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2 = [0u8, 0, 0, 0, 0, 0, 0];
    let extra = [0u8, 0, 0, 0, 0, 0];

    let event: Vec<u8> = header
        .into_iter()
        .chain(bank_1.into_iter())
        .chain(padding1.into_iter())
        .chain(bank_2.into_iter())
        .chain(padding2.into_iter())
        .chain(extra.into_iter())
        .collect();

    let event_views = EventViews::from_be_bytes(&event);
    assert_eq!(1, event_views.count());

    let mut event_views = EventViews::from_be_bytes(&event);
    assert_eq!(1, event_views.next().unwrap().id());
    assert_eq!(6, event_views.remainder().len());
}

#[test]
fn multiple_exact_be_event_views() {
    let header_a = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank_1a = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1a = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2a = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2a = [0u8, 0, 0, 0, 0, 0, 0];

    let header_b = [
        0u8, 5, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank_1b = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1b = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2b = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2b = [0u8, 0, 0, 0, 0, 0, 0];
    let event: Vec<u8> = header_a
        .into_iter()
        .chain(bank_1a.into_iter())
        .chain(padding1a.into_iter())
        .chain(bank_2a.into_iter())
        .chain(padding2a.into_iter())
        .chain(header_b.into_iter())
        .chain(bank_1b.into_iter())
        .chain(padding1b.into_iter())
        .chain(bank_2b.into_iter())
        .chain(padding2b.into_iter())
        .collect();

    let event_views = EventViews::from_be_bytes(&event);
    assert_eq!(2, event_views.count());

    let mut event_views = EventViews::from_be_bytes(&event);
    assert_eq!(1, event_views.next().unwrap().id());
    assert_eq!(5, event_views.next().unwrap().id());
    assert_eq!(0, event_views.remainder().len());
}

#[test]
fn multiple_inexact_be_event_views() {
    let header_a = [
        0u8, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank_1a = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding1a = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2a = [66u8, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding2a = [0u8, 0, 0, 0, 0, 0, 0];

    let header_b = [
        0u8, 5, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 56, 0, 0, 0, 48, 0, 0, 0, 49,
    ];
    let bank_1b = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 255];
    let padding1b = [0u8, 0, 0, 0, 0, 0, 0];
    let bank_2b = [66u8, 65, 78, 75, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 255];
    let padding2b = [0u8, 0, 0, 0, 0, 0, 0];
    let extra = [0u8];
    let event: Vec<u8> = header_a
        .into_iter()
        .chain(bank_1a.into_iter())
        .chain(padding1a.into_iter())
        .chain(bank_2a.into_iter())
        .chain(padding2a.into_iter())
        .chain(header_b.into_iter())
        .chain(bank_1b.into_iter())
        .chain(padding1b.into_iter())
        .chain(bank_2b.into_iter())
        .chain(padding2b.into_iter())
        .chain(extra.into_iter())
        .collect();

    let event_views = EventViews::from_be_bytes(&event);
    assert_eq!(2, event_views.count());

    let mut event_views = EventViews::from_be_bytes(&event);
    assert_eq!(1, event_views.next().unwrap().id());
    assert_eq!(5, event_views.next().unwrap().id());
    assert_eq!(1, event_views.remainder().len());
}
