use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::PathBuf;

use dict::index::{Index, Location};
use dict::*;

fn get_asset_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
}

fn get_resource(name: &str) -> PathBuf {
    get_asset_path().join(name)
}

fn load_resource(name: &str) -> File {
    let res = get_resource(name);
    File::open(res).unwrap()
}

fn loc(offset: u64, size: u64) -> Location {
    Location { offset, size }
}

// Uncompressed dict reader

#[test]
fn correct_position() {
    let reader = Cursor::new("Ignore me: important");
    let mut dict = Uncompressed::new(reader).unwrap();
    let def = dict.fetch_definition(loc(11, 9)).unwrap();

    assert_eq!(def, "important");
}

#[test]
fn seeking_to_start() {
    let reader = Cursor::new("abcdefg");
    let mut dict = Uncompressed::new(reader).unwrap();
    let def = dict.fetch_definition(loc(0, 3)).unwrap();

    assert_eq!(def, "abc");
}

#[test]
#[should_panic]
fn seeking_beyond_file() {
    let reader = Cursor::new("xyz is too short ;)");
    let mut dict = Uncompressed::new(reader).unwrap();
    dict.fetch_definition(loc(66642, 18)).unwrap();
}

#[test]
#[should_panic]
fn reading_beyond_file_boundary() {
    let reader = Cursor::new("blablablup");
    let mut dict = Uncompressed::new(reader).unwrap();
    dict.fetch_definition(loc(0, 424242)).unwrap();
}

#[test]
#[should_panic]
fn length_too_large() {
    let reader = Cursor::new("blablablup");
    let mut dict = Uncompressed::new(reader).unwrap();
    dict.fetch_definition(loc(0, 424242)).unwrap();
}

// Compressed dict reader

#[test]
#[should_panic]
fn wrong_file_id() {
    let data = Cursor::new(vec![0x1F, 0x8C]);
    Compressed::new(data).unwrap();
}

#[test]
fn right_file_id() {
    let file = load_resource("lat-deu.dict.dz");
    Compressed::new(file).unwrap();
}

#[test]
#[should_panic]
fn no_fextra_field() {
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Reset flags field to 0
    data[3] = 0;

    let data = Cursor::new(data);
    Compressed::new(data).unwrap();
}

#[test]
#[should_panic]
fn invalid_si_bytes() {
    // silsi2 are the identification for the dictzip extension
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Reset flags field to 0
    data[12] = 0;
    data[13] = 0;

    let data = Cursor::new(data);
    Compressed::new(data).unwrap();
}

#[test]
#[should_panic]
fn invalid_version_number() {
    // dictzip format specifies a field called "VER"
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Reset version field to 0
    data[16] = 0;
    data[17] = 0;

    let data = Cursor::new(data);
    Compressed::new(data).unwrap();
}

#[test]
#[should_panic]
fn mismatched_subfield_and_fextra_length() {
    // the "FEXTRA" length (also called XLEN in the specification) contains the additional header
    // information  for the dictzip format. This field has a header on its own and hence it is
    // necessary to check whether both match and whether non-matching field lengths are detected
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Reset flags field to 0
    data[14] = 8;
    data[15] = 9;

    let data = Cursor::new(data);
    Compressed::new(data).unwrap();
}

#[test]
#[should_panic]
fn chunk_count_is_zero() {
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Reset chunk count to 0
    data[20] = 0;
    data[21] = 0;

    let data = Cursor::new(data);
    Compressed::new(data).unwrap();
}

#[test]
#[should_panic]
fn mismatched_chunk_count_and_xlen() {
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Reset chunk count to 0
    data[20] = 8;
    data[21] = 9;

    let data = Cursor::new(data);
    Compressed::new(data).unwrap();
}

#[test]
fn word_doesnt_exist() {
    let dict_path = get_resource("lat-deu.dict.dz");
    let index_path = get_resource("lat-deu.index");
    let mut dict = Dict::from_file(dict_path, index_path).unwrap();

    assert!(dict.lookup("testtesttest", false).is_err());
}

#[test]
fn word_does_exist() {
    let dict_path = get_resource("lat-deu.dict.dz");
    let index_path = get_resource("lat-deu.index");
    let mut dict = Dict::from_file(dict_path, index_path).unwrap();
    let res = dict.lookup("mater", false).unwrap();

    assert!(res[0].headword.starts_with("mater"));
}

#[test]
fn get_word_from_first_chunk() {
    let dict_path = get_resource("lat-deu.dict.dz");
    let index_path = get_resource("lat-deu.index");
    let mut dict = Dict::from_file(dict_path, index_path).unwrap();
    let res = dict.lookup("amo", false).unwrap();

    assert!(res[0].headword.starts_with("amo"));
}

#[test]
fn get_word_from_last_chunk() {
    let dict_path = get_resource("lat-deu.dict.dz");
    let index_path = get_resource("lat-deu.index");
    let mut dict = Dict::from_file(dict_path, index_path).unwrap();
    let res = dict.lookup("vultus", false).unwrap();

    assert!(res[0].headword.starts_with("vultus"));
}

#[test]
fn get_word_split_at_chunk_border() {
    let dict_path = get_resource("lat-deu.dict.dz");
    let index_path = get_resource("lat-deu.index");
    let mut dict = Dict::from_file(dict_path, index_path).unwrap();
    let res = dict.lookup("circumfero", false).unwrap();

    // For the above dictionary, the chunk (or block) length of each uncompressed chunk is 58315;
    // Exactly there, the definition circumfero is split into two pieces:
    assert!(res[0].headword.starts_with("circumfero"));
    assert!(res[0].definition.ends_with("herumtreiben\n"));
}

#[test]
fn comment_parsing_correct() {
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Set comment bit to 1
    data[3] |= dict::compressed::GZ_COMMENT;

    // Add comment after file name; the header itself is for this particular file 36 bytes + 13
    // bytes file name (byte 13 is 0-byte)
    let mut newdata: Vec<u8> = Vec::with_capacity(data.len() - 13);
    newdata.extend(&data[0..49]);
    newdata.extend(b"hi there\0"); // Insert comment
    newdata.extend(&data[49..]);

    let index_reader = BufReader::new(File::open(get_resource("lat-deu.index")).unwrap());
    let index = Box::new(Index::new(index_reader).unwrap());
    let data = Cursor::new(newdata);
    let reader = Box::new(Compressed::new(data).unwrap());
    let mut dict = Dict::from_existing(reader, index).unwrap();
    let res = dict.lookup("mater", false).unwrap();

    assert!(res[0].headword.starts_with("mater"));
}

#[test]
fn no_filename_correct() {
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Reset fname bit to 0
    data[3] &= !dict::compressed::GZ_FNAME;

    // flags byte of gz header
    // remove file name from file; there are various fields in the gz header, which I won't repeat;
    // together with the bytes in fextra (listing 7 compressed chunks), the file name starts at
    // position 36. If you want to check the maths, have a look at src/dictreader.rs. The file name
    // is 13 bytes long, so these need to be extracted:
    let mut newdata: Vec<u8> = Vec::with_capacity(data.len() - 13);
    newdata.extend(&data[0..36]);
    newdata.extend(&data[49..]);

    let index_reader = BufReader::new(File::open(get_resource("lat-deu.index")).unwrap());
    let index = Box::new(Index::new(index_reader).unwrap());
    let data = Cursor::new(newdata);
    let reader = Box::new(Compressed::new(data).unwrap());
    let mut dict = Dict::from_existing(reader, index).unwrap();
    let res = dict.lookup("mater", false).unwrap();

    assert!(res[0].headword.starts_with("mater"));
}

#[test]
#[should_panic]
fn seek_beyond_end_of_file() {
    let mut file = load_resource("lat-deu.dict.dz");
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    let data = Cursor::new(data);
    let mut dict = Compressed::new(data).unwrap();
    dict.fetch_definition(loc(9999999999u64, 888u64)).unwrap();
}
