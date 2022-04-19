use dict::{index::{Entry, Index, IndexReader, Location}, IndexError, LookupResult};
use std::{io::{Cursor, BufReader}, fs::File, path::PathBuf};

fn get_asset_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
}

fn get_resource(name: &str) -> PathBuf {
    get_asset_path().join(name)
}

fn load_resource(name: &str) -> BufReader<File> {
    let res = get_resource(name);
    BufReader::new(File::open(res).unwrap())
}

fn example_index() -> Box<dyn IndexReader> {
    let index = load_resource("lat-deu.index");
    let index = Index::new(index).unwrap();

    Box::new(index)
}

fn custom_index(index_path: &str) -> Box<dyn IndexReader> {
    let index = load_resource(index_path);
    let index = Index::new(index).unwrap();

    Box::new(index)
}

fn loc(offset: u64, size: u64) -> Location {
    Location { offset, size }
}

// Index parsing

#[test]
#[should_panic]
fn invalid_line() {
    let reader = Cursor::new("blabla\nblublbub yo");
    Index::new(reader).unwrap();
}

#[test]
#[should_panic]
fn invalid_column() {
    let reader = Cursor::new("only one\t(tab) character");
    Index::new(reader).unwrap();
}

#[test]
fn good_line() {
    let reader = Cursor::new("word\toffset\tlength");
    let mut index = Index::new(reader).unwrap();

    assert_eq!(
        index.find("word", false, false).unwrap(),
        vec![Entry {
            headword: "word".into(),
            location: loc(43478075309, 40242121569),
            original: None
        }]
    );
}

#[test]
fn two_entries_parsed() {
    let reader = Cursor::new("another\ta0b\tc\nword\toffset\tlength");
    let mut index = Index::new(reader).unwrap();

    assert_eq!(
        index.find("word", false, false).unwrap(),
        vec![Entry {
            headword: "word".into(),
            location: loc(43478075309, 40242121569),
            original: None
        }]
    );
    assert_eq!(
        index.find("another", false, false).unwrap(),
        vec![Entry {
            headword: "another".into(),
            location: loc(109851, 28),
            original: None
        }]
    );
}

#[test]
#[should_panic]
fn number_parsing_fails() {
    let reader = Cursor::new("valid word\tinvalid_offset\tDA");
    Index::new(reader).unwrap();
}

// Test indexes

#[test]
fn test_index_find() {
    //let mut index = custom_index("case_insensitive_dict.index");
    let index = load_resource("case_insensitive_dict.index");
    let mut index = Index::new(index).unwrap();

    // Nonexistant word
    assert!(index.find("apples", false, false).is_err());

    dbg!(&index.entries);

    // Without fuzzy
    let results = index.find("bar", false, false).unwrap();
    let expected = vec![
        Entry{
            headword: "bar".into(),
            location: loc(443, 30),
            original: None,
        }
    ];
    assert_eq!(results, expected);

    // With fuzzy
    let results = index.find("bas", true, false).unwrap();
    let expected = vec![
        Entry{
            headword: "bar".into(),
            location: loc(443, 30),
            original: None,
        }
    ];
    assert_eq!(results, expected);
}
