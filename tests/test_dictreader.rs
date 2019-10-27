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
    dictreader::DictReaderRaw::new(x).unwrap()
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
fn test_that_file_with_invalid_si_bytes_is_reported() {
    // si1si2 are the identification for the dictzip extension
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
    let mut dict = load_dictionary_from_file(dictdz, index).unwrap();
    assert!(dict.lookup("testtesttest").is_err());
}

#[test]
fn test_retrieval_of_a_word_which_exists_works() {
    let dictdz = get_asset_path("lat-deu.dict.dz");
    let index = get_asset_path("lat-deu.index");
    let mut dict = load_dictionary_from_file(dictdz, index).unwrap();
    let word = dict.lookup("mater");
    let word = word.unwrap();
    assert!(word.starts_with("mater"));
}

#[test]
fn test_that_word_from_first_chunk_works() {
    let dictdz = get_asset_path("lat-deu.dict.dz");
    let index = get_asset_path("lat-deu.index");
    let mut dict = load_dictionary_from_file(dictdz, index).unwrap();
    let word = dict.lookup("amo").unwrap();
    assert!(word.starts_with("amo"));
}

#[test]
fn test_lookup_into_last_chunk_works() {
    let dictdz = get_asset_path("lat-deu.dict.dz");
    let index = get_asset_path("lat-deu.index");
    let mut dict = load_dictionary_from_file(dictdz, index).unwrap();
    let word = dict.lookup("vultus").unwrap();
    assert!(word.starts_with("vultus"));
}

#[test]
fn test_that_definitions_wrapping_around_chunk_border_are_extracted_correctly() {
    let dictdz = get_asset_path("lat-deu.dict.dz");
    let index = get_asset_path("lat-deu.index");
    let mut dict = load_dictionary_from_file(dictdz, index).unwrap();
    // for the above dictionary, the chunk (or block) length of each uncompressed chunk is 58315;
    // exactly there, the definition circumfero is split into two pieces:
    let word = dict.lookup("circumfero").unwrap();
    assert!(word.starts_with("circumfero"));
    // last word from definition must be present, too
    assert!(word.ends_with("herumtreiben\n"));
}

#[test]
fn test_files_with_comment_is_parsed_correctly() {
    // file in assets has no comment, so add one
    let mut rsrc = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // set comment bit to 1
    data[3] |= dictreader::GZ_COMMENT;
    // add comment _after_file name; the header itself is for this particular file 36 bytes + 13
    // bytes file name (byte 13 is 0-byte)
    let mut newdata: Vec<u8> = Vec::with_capacity(data.len() - 13);
    newdata.extend(&data[0..49]);
    // "h", "i", " ", "t", "h", "e", "r", "e"
    newdata.extend(vec![104u8, 105u8, 32u8, 116u8, 104u8, 101u8, 114u8, 101u8, 0u8]);
    newdata.extend(&data[49..]);

    let data = dict::dictreader::DictReaderDz::new(Cursor::new(newdata)).unwrap();
    let index = dict::indexing::parse_index_from_file(get_asset_path("lat-deu.index")).unwrap();
    let mut dict = dict::load_dictionary(Box::new(data), index);
    let word = dict.lookup("mater");
    let word = word.unwrap();
    assert!(word.starts_with("mater"));
}

#[test]
fn test_file_without_file_name_is_parsed_correctly() {
    let mut rsrc = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    rsrc.read_to_end(&mut data).unwrap();
    // reset fname bit to 0
    data[3] &= !dictreader::GZ_FNAME; // flags byte of gz header
    // remove file name from file; there are various fields in the gz header, which I won't repeat;
    // together with the bytes in fextra (listing 7 compressed chunks), the file name starts at
    // position 36. If you want to check the maths, have a look at src/dictreader.rs. The file name
    // is 13 bytes long, so these need to be extracted:
    let mut newdata: Vec<u8> = Vec::with_capacity(data.len() - 13);
    newdata.extend(&data[0..36]);
    newdata.extend(&data[49..]);

    let data = dict::dictreader::DictReaderDz::new(Cursor::new(newdata)).unwrap();
    let index = dict::indexing::parse_index_from_file(get_asset_path("lat-deu.index")).unwrap();
    let mut dict = dict::load_dictionary(Box::new(data), index);
    let word = dict.lookup("mater");
    let word = word.unwrap();
    assert!(word.starts_with("mater"));
}

#[test]
#[should_panic]
fn test_that_seek_beyond_end_of_file_is_detected() {
    let dictdz = get_asset_path("lat-deu.dict.dz");
    let mut dict = dictreader::load_dict(dictdz).unwrap();
    dict.fetch_definition(9999999999u64, 888u64).unwrap();
}

