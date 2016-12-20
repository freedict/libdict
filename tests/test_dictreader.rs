#[cfg(test)]
extern crate dict;

use std::io::{Cursor};

use dict::*;
use dict::dictreader::*;

type FakeFile = Cursor<String>;

fn mk_file(input: &str) -> FakeFile {
    let input = input.to_string();
    // Cursor<&[u8]> implements Read and Seek
    Cursor::new(input)
}

fn mk_dict(x: FakeFile) -> dictreader::DictReaderRaw<FakeFile> {
    dictreader::DictReaderRaw::new(x)
}

#[test]
fn test_that_dictreader_does_to_correct_position() {
    let text = mk_file("Ignore me: important");
    assert_eq!(mk_dict(text).fetch_definition(11, 9).unwrap(), "important");
}

#[test]
fn test_that_seeking_to_beginning_works() {
    let text = mk_file("abcdefg");
    assert_eq!(mk_dict(text).fetch_definition(0, 3).unwrap(), "abc");
}

#[test]
#[should_panic]
fn test_that_seeking_beyond_file_is_caught() {
    let text = mk_file("xyz is too short ;)");
    mk_dict(text).fetch_definition(66642, 18).unwrap();
}

#[test]
#[should_panic]
fn test_that_reading_beyond_file_boundary_is_caught() {
    let text = mk_file("blablablup");
    mk_dict(text).fetch_definition(0, 424242).unwrap();
}

#[test]
#[should_panic]
fn test_error_if_length_is_too_large() {
    let mut longfile = String::with_capacity(dictreader::MAX_BYTES_FOR_BUFFER as usize + 10);
    for _ in 0..(dictreader::MAX_BYTES_FOR_BUFFER+10) {
        longfile.push('u');
    }
    let text = mk_file(&longfile);
    mk_dict(text).fetch_definition(0, dictreader::MAX_BYTES_FOR_BUFFER+1).unwrap();
}
