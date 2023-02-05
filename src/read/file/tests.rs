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

#[test]
fn short_file_view() {
    let ini_odb = [128, 0, 73, 77, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0];
    let file_view = FileView::try_from(&ini_odb[..]);

    assert!(matches!(
        file_view,
        Err(TryFileViewFromSliceError::IniOdbSizeMismatch)
    ));
}

#[test]
fn invalid_badborid_file_view() {
    let ini_odb = [127, 0, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let header = [
        0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank = [66, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [128, 1, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(matches!(
        file_view,
        Err(TryFileViewFromSliceError::BadBorId)
    ));
}

#[test]
fn invalid_badbormi_file_view() {
    let ini_odb = [128, 0, 74, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let header = [
        0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank = [66, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [128, 1, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(matches!(
        file_view,
        Err(TryFileViewFromSliceError::BadBorMi)
    ));
}

#[test]
fn invalid_iniodbsizemismatch_file_view() {
    let ini_odb = [128, 0, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 1, 0, 0, 0, 255];
    let header = [
        0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 40, 0, 0, 0, 32, 0, 0, 0, 1,
    ];
    let bank = [66, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [128, 1, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(matches!(
        file_view,
        Err(TryFileViewFromSliceError::IniOdbSizeMismatch)
    ));
}

#[test]
fn invalid_finodbsizemismatch_file_view() {
    let ini_odb = [128, 0, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let header = [
        0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    ];
    let bank = [66, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [128, 1, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 2, 255];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(matches!(
        file_view,
        Err(TryFileViewFromSliceError::FinOdbSizeMismatch)
    ));
}

#[test]
fn invalid_badeorid_file_view() {
    let ini_odb = [128, 0, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let header = [
        0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    ];
    let bank = [66, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [127, 1, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(matches!(
        file_view,
        Err(TryFileViewFromSliceError::BadEorId)
    ));
}

#[test]
fn invalid_badeormi_file_view() {
    let ini_odb = [128, 0, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let header = [
        0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    ];
    let bank = [66, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [128, 1, 74, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(matches!(
        file_view,
        Err(TryFileViewFromSliceError::BadEorMi)
    ));
}

#[test]
fn invalid_run_number_mismatch_file_view() {
    let ini_odb = [128, 0, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let header = [
        0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    ];
    let bank = [66, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [128, 1, 73, 77, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(matches!(
        file_view,
        Err(TryFileViewFromSliceError::RunNumberMismatch)
    ));
}

#[test]
fn valid_be_file_view() {
    let ini_odb = [128, 0, 73, 77, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 255];
    let header = [
        0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1,
    ];
    let bank = [66, 65, 78, 75, 0, 1, 0, 1, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [128, 1, 73, 77, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 1, 254];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]).unwrap();

    assert_eq!(1, file_view.run_number());
    assert_eq!(2, file_view.initial_timestamp());
    assert_eq!(3, file_view.final_timestamp());
    assert_eq!([255], file_view.initial_odb());
    assert_eq!([254], file_view.final_odb());
}

#[test]
fn valid_le_file_view() {
    let ini_odb = [0, 128, 77, 73, 1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 255];
    let header = [
        1, 0, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 16, 0, 0, 0, 1, 0, 0, 0,
    ];
    let bank = [66, 65, 78, 75, 1, 0, 1, 0, 255];
    let padding = [0, 0, 0, 0, 0, 0, 0];
    let fin_odb = [1, 128, 77, 73, 1, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 254];
    let file_view: Vec<u8> = ini_odb
        .into_iter()
        .chain(header.into_iter())
        .chain(bank.into_iter())
        .chain(padding.into_iter())
        .chain(fin_odb.into_iter())
        .collect();
    let file_view = FileView::try_from(&file_view[..]).unwrap();

    assert_eq!(1, file_view.run_number());
    assert_eq!(2, file_view.initial_timestamp());
    assert_eq!(3, file_view.final_timestamp());
    assert_eq!([255], file_view.initial_odb());
    assert_eq!([254], file_view.final_odb());
}

#[test]
fn run_number_unchecked_short() {
    let slice = [128, 0, 255, 255, 0, 0, 0];
    assert!(matches!(
        run_number_unchecked(&slice),
        Err(TryFileViewFromSliceError::IniOdbSizeMismatch)
    ));
}

#[test]
fn run_number_unchecked_bad_bor_id() {
    let slice = [0, 0, 255, 255, 0, 0, 0, 1];
    assert!(matches!(
        run_number_unchecked(&slice),
        Err(TryFileViewFromSliceError::BadBorId)
    ));
}

#[test]
fn run_number_unchecked_ok() {
    let slice = [128, 0, 255, 255, 0, 0, 0, 1];
    assert_eq!(1, run_number_unchecked(&slice).unwrap());

    let slice = [0, 128, 255, 255, 1, 0, 0, 0];
    assert_eq!(1, run_number_unchecked(&slice).unwrap());
}

#[test]
fn initial_timestamp_unchecked_short() {
    let slice = [128, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0];
    assert!(matches!(
        initial_timestamp_unchecked(&slice),
        Err(TryFileViewFromSliceError::IniOdbSizeMismatch)
    ));
}

#[test]
fn initial_timestamp_unchecked_bad_bor_id() {
    let slice = [0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 1];
    assert!(matches!(
        initial_timestamp_unchecked(&slice),
        Err(TryFileViewFromSliceError::BadBorId)
    ));
}

#[test]
fn initial_timestamp_unchecked_ok() {
    let slice = [128, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 1];
    assert_eq!(1, initial_timestamp_unchecked(&slice).unwrap());

    let slice = [0, 128, 255, 255, 255, 255, 255, 255, 1, 0, 0, 0];
    assert_eq!(1, initial_timestamp_unchecked(&slice).unwrap());
}
