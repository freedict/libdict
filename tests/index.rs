use dict::*;
use std::io::Cursor;

// Index parsing

#[test]
#[should_panic]
fn invalid_line() {
    let reader = Cursor::new("blabla\nblublbub yo");
    index::parse(reader).unwrap();
}

#[test]
#[should_panic]
fn invalid_column() {
    let reader = Cursor::new("only one\t(tab) character");
    index::parse(reader).unwrap();
}

#[test]
fn good_line() {
    let reader = Cursor::new("word\toffset\tlength");
    let index = index::parse(reader).unwrap();

    assert_eq!(
        *index.words.get("word").unwrap(),
        (43478075309, 40242121569)
    );
}

#[test]
fn two_entries_parsed() {
    let reader = Cursor::new("word\toffset\tlength\nanother\ta0b\tc");
    let index = index::parse(reader).unwrap();

    assert_eq!(
        *index.words.get("word").unwrap(),
        (43478075309, 40242121569)
    );
    assert_eq!(*index.words.get("another").unwrap(), (109851, 28));
}

#[test]
#[should_panic]
fn number_parsing_fails() {
    let reader = Cursor::new("valid word\tinvalid_offset\tDA");
    index::parse(reader).unwrap();
}