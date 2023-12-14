use super::*;

#[test]
fn short_file_view() {
    let ini_odb = [128, 0, 73, 77, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0];
    let file_view = FileView::try_from(&ini_odb[..]);

    assert!(file_view.is_err());
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(file_view.is_err());
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(file_view.is_err());
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(file_view.is_err());
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(file_view.is_err());
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(file_view.is_err());
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(file_view.is_err());
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
        .collect();
    let file_view = FileView::try_from(&file_view[..]);

    assert!(file_view.is_err());
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
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
        .chain(header)
        .chain(bank)
        .chain(padding)
        .chain(fin_odb)
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
    assert!(run_number_unchecked(&slice).is_err());
}

#[test]
fn run_number_unchecked_bad_bor_id() {
    let slice = [0, 0, 255, 255, 0, 0, 0, 1];
    assert!(run_number_unchecked(&slice).is_err());
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
    assert!(initial_timestamp_unchecked(&slice).is_err());
}

#[test]
fn initial_timestamp_unchecked_bad_bor_id() {
    let slice = [0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 1];
    assert!(initial_timestamp_unchecked(&slice).is_err());
}

#[test]
fn initial_timestamp_unchecked_ok() {
    let slice = [128, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 1];
    assert_eq!(1, initial_timestamp_unchecked(&slice).unwrap());

    let slice = [0, 128, 255, 255, 255, 255, 255, 255, 1, 0, 0, 0];
    assert_eq!(1, initial_timestamp_unchecked(&slice).unwrap());
}
