use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::fs::File;

use errors::DictError;
use errors::DictError::*;


/// Get the assigned number for a character
/// If the character was unknown, an empty Err(()) is returned.
fn get_base(input: char) -> Result<u64, ()> {
    match input {
        'A' ... 'Z' => Ok((input as u64) - 65), // 'A' should become 0
        'a' ... 'z' => Ok((input as u64) - 71), // 'a' should become 26, ...
        '0' ... '9' => Ok((input as u64) + 4), // 0 should become 52
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(()),
    }
}

pub fn get_offset(word: &str) -> Result<u64, DictError> {
    let mut index = 0u64;
    for (i, character) in word.chars().rev().enumerate() {
        index += match get_base(character) {
            Ok(x) => x * 64u64.pow(i as u32),
            Err(_) => return Err(InvalidCharacter(character, None, Some(i))),
        };
    }
    Ok(index)
}

fn parse_line(line: &str, line_number: usize) -> Result<(String, u64, u64), DictError> {
    let mut split = line.split("\t");
    let word = try!(split.next().ok_or(MissingColumnInIndex(line_number)));

    // second column: offset into file
    let start_offset = try!(split.next().ok_or(MissingColumnInIndex(line_number)));
    let start_offset = try!(get_offset(start_offset));

    // get entry length
    let length = try!(split.next().ok_or(MissingColumnInIndex(line_number)));
    let length = try!(get_offset(length));

    Ok((word.to_string(), start_offset, length))
}

pub fn parse_index<B: BufRead>(br: B) -> Result<HashMap<String, (u64, u64)>, DictError> {
    let mut index = HashMap::new();

    for (line_number, line) in br.lines().enumerate() {
        let line = try!(line);
        let (word, start_offset, length) = try!(parse_line(&line, line_number));
        index.entry(word.clone()).or_insert((start_offset, length));
    }

    Ok(index)
}

pub fn parse_index_from_file(filename: String) -> Result<HashMap<String, (u64, u64)>, DictError> {
    let file = File::open(filename).unwrap();
    let file = BufReader::new(&file);
    parse_index(file)
}

