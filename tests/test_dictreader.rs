#[cfg(test)]
extern crate dict;

use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use dict::*;
use dict::dictreader::*;

type StringFile = Cursor<String>;

fn get_asset_path(fname: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("assets");
    path.push(fname);
    path
}

// load test resource from tests/assets
fn load_resource(name: &str) -> File {
    let path = get_asset_path(name);
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
    let file = load_resource("lat-deu.dict.dz");
    DictReaderDz::new(file).unwrap();
}

#[test]
#[should_panic]
fn test_gzip_files_without_fextra_panic() {
    let mut rsrc = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // reset flags field to 0
    data[3] = 0;
    let data = Cursor::new(data);
    DictReaderDz::new(data).unwrap();
}

#[test]
#[should_panic]
fn test_that_file_with_invalid_si_bytes_reporged() {
    // si1si2 are the identificationfor the dictzip extension
    let mut rsrc = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // reset flags field to 0
    data[12] = 0;
    data[13] = 0;
    let data = Cursor::new(data);
    DictReaderDz::new(data).unwrap();
}

#[test]
#[should_panic]
fn test_gzip_with_invalid_version_num_are_reported() {
    // the dictzip format specifies a field called "VER"
    let mut rsrc = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // reset version field to 0
    data[16] = 0;
    data[17] = 0;
    let data = Cursor::new(data);
    DictReaderDz::new(data).unwrap();
}

#[test]
#[should_panic]
fn test_mismatching_subfield_length_and_fextra_length_is_reported() {
    // the "FEXTRA" length (also called XLEN in the specification) contains the additional header
    // information  for the dictzip format. This field has a header on its own and hence it is
    // necessary to check whether both match and whether non-matching field lengths are detected
    // the dictzip format specifies a field called "VER"
    let mut rsrc = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // reset flags field to 0
    data[14] = 0;
    data[15] = 0;
    let data = Cursor::new(data);
    DictReaderDz::new(data).unwrap();
}

#[test]
#[should_panic]
fn test_chunk_count_may_not_be_0() {
    // the "FEXTRA" length (also called XLEN in the specification) contains the additional header
    // information  for the dictzip format. This field has a header on its own and hence it is
    // necessary to check whether both match and whether non-matching field lengths are detected
    // the dictzip format specifies a field called "VER"
    let mut rsrc = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // reset chunk count to 0
    data[20] = 0;
    data[21] = 0;
    let data = Cursor::new(data);
    DictReaderDz::new(data).unwrap();
}


#[test]
#[should_panic]
fn test_chunk_count_and_xlen_must_match() {
    // doc see above
    let mut rsrc = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // reset chunk count to 0
    data[20] = 8;
    data[21] = 9;
    let data = Cursor::new(data);
    DictReaderDz::new(data).unwrap();
}

#[test]
fn test_retrieval_of_a_word_which_doesnt_exist_yields_error() {
    let dictdz = get_asset_path("lat-deu.dict.dz");
    let index = get_asset_path("lat-deu.index");
    let mut dict = load_dictionary(&dictdz.to_str().unwrap(), &index.to_str().unwrap()).unwrap();
    assert!(dict.lookup("testtesttest").is_err());
}

#[test]
fn test_retrieval_of_a_word_which_exists_works() {
    let dictdz = get_asset_path("lat-deu.dict.dz");
    let index = get_asset_path("lat-deu.index");
    let mut dict = load_dictionary(&dictdz.to_str().unwrap(), &index.to_str().unwrap()).unwrap();
    let word = dict.lookup("mater");
    let word = word.unwrap();
    assert!(word.starts_with("mater"));
}

