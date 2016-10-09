#[cfg(test)]

extern crate dict;
use dict::indexing::*;

////////////////////////////////////////////////////////////////////////////////
// Test single-character calculations
///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_that_uppercase_letters_get_correct_number() {
    assert_eq!(dict::indexing::get_offset("A"), 0);
    assert_eq!(dict::indexing::get_offset("M"), 12);
    assert_eq!(get_offset("Z"), 25);
}

#[test]
fn test_that_lowercase_letters_get_correct_number() {
     assert_eq!(dict::indexing::get_offset("a"), 26);
    assert_eq!(dict::indexing::get_offset("m"), 38);
    assert_eq!(get_offset("z"), 51);
}

#[test]
fn test_that_characters_get_correct_number() {
    assert_eq!(get_offset("0"), 52);
    assert_eq!(get_offset("9"), 61);
}

#[test]
fn test_that_slash_and_plus_get_correct_number() {
    assert_eq!(get_offset("+"), 62);
    assert_eq!(get_offset("/"), 63);
}

#[test]
fn test_that_unknown_characters_return_error() {
    assert_eq!(get_offset("*"), 99999);
}

////////////////////////////////////////////////////////////////////////////////
// Test multi-character-calculations calculations
///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_that_big_offsets_work() {
    assert_eq!(get_offset("3fW2"), 14546358);
}

#[test]
fn test_that_short_strings_work() {
    assert_eq!(get_offset("c"), 28);
}
