#[cfg(test)]
extern crate dict;

use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use dict::*;
use dict::dictreader::*;

type StringFile = Cursor<String>;

// load test resource from tests/assets
fn load_resource(name: &str) -> File {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("assets");
    path.push(name);
    File::open(path).unwrap()
}


fn str2file(input: &str) -> StringFile {
    let input = input.to_string();
    // Cursor<&[u8]> implements Read and Seek
    Cursor::new(input)
}


fn mk_dict(x: StringFile) -> dictreader::DictReaderRaw<StringFile> {
    dictreader::DictReaderRaw::new(x)
}

#[test]
fn test_that_dictreader_does_to_correct_position() {
    let text = str2file("Ignore me: important");
    assert_eq!(mk_dict(text).fetch_definition(11, 9).unwrap(), "important");
}

#[test]
fn test_that_seeking_to_beginning_works() {
    let text = str2file("abcdefg");
    assert_eq!(mk_dict(text).fetch_definition(0, 3).unwrap(), "abc");
}

#[test]
#[should_panic]
fn test_that_seeking_beyond_file_is_caught() {
    let text = str2file("xyz is too short ;)");
    mk_dict(text).fetch_definition(66642, 18).unwrap();
}

#[test]
#[should_panic]
fn test_that_reading_beyond_file_boundary_is_caught() {
    let text = str2file("blablablup");
    mk_dict(text).fetch_definition(0, 424242).unwrap();
}

#[test]
#[should_panic]
fn test_error_if_length_is_too_large() {
    let mut longfile = String::with_capacity(dictreader::MAX_BYTES_FOR_BUFFER as usize + 10);
    for _ in 0..(dictreader::MAX_BYTES_FOR_BUFFER+10) {
        longfile.push('u');
    }
    let text = str2file(&longfile);
    mk_dict(text).fetch_definition(0, dictreader::MAX_BYTES_FOR_BUFFER+1).unwrap();
}

////////////////////////////////////////////////////////////////////////////////
// test dict.dz reader


#[test]
#[should_panic]
fn test_files_with_incorrect_file_id_are_detected() {
    let data = Cursor::new(vec![0x1F, 0x8C]);
    DictReaderDz::new(data).unwrap();
}

#[test]
fn test_files_with_correct_file_id_work() {
    let file = load_resource("foo-bar.dict.dz");
    DictReaderDz::new(file).unwrap();
}

#[test]
#[should_panic]
fn test_gzip_files_without_fextra_panic() {
    let mut rsrc = load_resource("foo-bar.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // reset flags field to 0
    data[3] = 0;
    let data = Cursor::new(data);
    DictReaderDz::new(data).unwrap();
}
// gz files without fextra are reported
// files with too short fextra, what happens
// file with invalid si1si2 report error
// invalid fextra extension version is detected
// chunk count and number length of XLEN field to accomodate chunk offsets must match (line 135)
// file name is ignored, if specified
// ignore comment if comment given
    
