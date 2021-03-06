use dict::indexing::*;

use std::io::Cursor;

////////////////////////////////////////////////////////////////////////////////
// Test single-character calculations
///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_that_uppercase_letters_get_correct_number() {
    assert_eq!(dict::indexing::decode_number("A").unwrap(), 0);
    assert_eq!(dict::indexing::decode_number("M").unwrap(), 12);
    assert_eq!(dict::indexing::decode_number("Z").unwrap(), 25);
}

#[test]
fn test_that_lowercase_letters_get_correct_number() {
     assert_eq!(dict::indexing::decode_number("a").unwrap(), 26);
    assert_eq!(dict::indexing::decode_number("m").unwrap(), 38);
    assert_eq!(dict::indexing::decode_number("z").unwrap(), 51);
}

#[test]
fn test_that_characters_get_correct_number() {
    assert_eq!(dict::indexing::decode_number("0").unwrap(), 52);
    assert_eq!(dict::indexing::decode_number("9").unwrap(), 61);
}

#[test]
fn test_that_slash_and_plus_get_correct_number() {
    assert_eq!(dict::indexing::decode_number("+").unwrap(), 62);
    assert_eq!(dict::indexing::decode_number("/").unwrap(), 63);
}

#[test]
fn test_that_unknown_characters_return_error() {
    assert!(dict::indexing::decode_number("*").is_err(), 99999);
}

////////////////////////////////////////////////////////////////////////////////
// Test multi-character-calculations calculations
///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_that_big_offsets_work() {
    assert_eq!(dict::indexing::decode_number("3fW2").unwrap(), 14546358);
}

#[test]
fn test_that_short_strings_work() {
    assert_eq!(dict::indexing::decode_number("c").unwrap(), 28);
}

////////////////////////////////////////////////////////////////////////////////
// Test parse_index
////////////////////////////////////////////////////////////////////////////////

fn mk_file(input: &str) -> Box<Cursor<String>> {
    let input = input.to_string();
    // Cursor<&[u8]> implements BufRead already
    Box::new(Cursor::new(input))
}

#[test]
#[should_panic]
fn test_that_invalid_line_causes_error() {
    parse_index(*mk_file("blabla\nblublbub yo")).unwrap();
}

#[test]
#[should_panic]
fn test_only_one_tab_causes_panic() {
    parse_index(*mk_file("only one\t(tab) character")).unwrap();
}

#[test]
fn test_that_normal_entry_works() {
    let index = parse_index(*mk_file("word\toffset\tlength")).unwrap();
    assert_eq!(*(index.get("word").unwrap()), (43478075309, 40242121569));
}

#[test]
fn test_that_two_entries_are_parsed() {
    let index = parse_index(*mk_file("word\toffset\tlength\nanother\ta0b\tc")).unwrap();
    assert_eq!(*(index.get("word").unwrap()), (43478075309, 40242121569));
    assert_eq!(*(index.get("another").unwrap()), (109851, 28));
}

#[test]
#[should_panic]
fn test_that_number_parsing_errors_are_propagated() {
    parse_index(*mk_file("valid word\tinvalid_offset\tDA")).unwrap();
}

