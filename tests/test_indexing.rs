#[cfg(test)]

extern crate dict;
use dict::indexing::*;

use std::io::{BufRead, Cursor};

////////////////////////////////////////////////////////////////////////////////
// Test single-character calculations
///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_that_uppercase_letters_get_correct_number() {
    assert_eq!(dict::indexing::get_offset("A").unwrap(), 0);
    assert_eq!(dict::indexing::get_offset("M").unwrap(), 12);
    assert_eq!(dict::indexing::get_offset("Z").unwrap(), 25);
}

#[test]
fn test_that_lowercase_letters_get_correct_number() {
     assert_eq!(dict::indexing::get_offset("a").unwrap(), 26);
    assert_eq!(dict::indexing::get_offset("m").unwrap(), 38);
    assert_eq!(dict::indexing::get_offset("z").unwrap(), 51);
}

#[test]
fn test_that_characters_get_correct_number() {
    assert_eq!(dict::indexing::get_offset("0").unwrap(), 52);
    assert_eq!(dict::indexing::get_offset("9").unwrap(), 61);
}

#[test]
fn test_that_slash_and_plus_get_correct_number() {
    assert_eq!(dict::indexing::get_offset("+").unwrap(), 62);
    assert_eq!(dict::indexing::get_offset("/").unwrap(), 63);
}

#[test]
fn test_that_unknown_characters_return_error() {
    assert!(dict::indexing::get_offset("*").is_err(), 99999);
}

////////////////////////////////////////////////////////////////////////////////
// Test multi-character-calculations calculations
///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_that_big_offsets_work() {
    assert_eq!(dict::indexing::get_offset("3fW2").unwrap(), 14546358);
}

#[test]
fn test_that_short_strings_work() {
    assert_eq!(dict::indexing::get_offset("c").unwrap(), 28);
}

////////////////////////////////////////////////////////////////////////////////
// Test parse_index
////////////////////////////////////////////////////////////////////////////////

fn mk_file(input: &str) -> Box<Cursor> {
    let input = input.to_string().as_bytes();
    // Cursor<&[u8]> implements BufRead already
    Box::new(Cursor::new(input));
}

#[test]
#[should_panic]
fn test_that_invalid_line_causes_error() {
    parse_index(*mk_file("blabla\nblublbub yo"));
}

