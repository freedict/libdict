//! Parse and decode `*.index` files.
//!
//! Each dictionary file (`*.dict.?)`) is accompanied by a `*.index` file containing a list of
//! words, together with its (byte) position in the dict file and its (byte) length. This module
//! provides functions to parse this index file.
//!
//! The position and the length of a definition is given in a semi-base64 encoding. It uses all
//! Latin letters (upper and lower case), all digits and additionally, `+` and `/`:
//!
//! `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/`
//!
//! The calculation works as follows: `sum += x * 64.pow(i`
//!
//! -   `i` is the position within the string to calculate the number from and counts from right to
//!     left, starting at 0.
//! -   `x` is the index within the array given above, i.e. `'a' == 26`.
//!
//! The sum makes up the index.
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::fs::File;

use crate::errors::DictError;
use crate::errors::DictError::*;

/// Datastructure to hold the word &rarr; (position, length) information.
pub type Index = HashMap<String, (u64, u64)>;

/// Get the assigned number for a character
/// If the character was unknown, an empty Err(()) is returned.
#[inline]
fn get_base(input: char) -> Result<u64, ()> {
    match input {
        'A' ..= 'Z' => Ok((input as u64) - 65), // 'A' should become 0
        'a' ..= 'z' => Ok((input as u64) - 71), // 'a' should become 26, ...
        '0' ..= '9' => Ok((input as u64) + 4), // 0 should become 52
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(()),
    }
}

/// Decode a number from a given String.
///
/// This function decodes a number from the format described in the module documentation. If
/// unknown characters/bytes are encountered, a `DictError` is returned.
///
/// # Example
///
/// ```
/// use dict::indexing::decode_number;
///
/// fn main() {
///     let myoffset = "3W/";
///     let myoffset = decode_number(myoffset).unwrap();
///     assert_eq!(myoffset, 226751);
/// }
/// ```
pub fn decode_number(word: &str) -> Result<u64, DictError> {
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
    let mut split = line.split('\t');
    let word = split.next().ok_or(MissingColumnInIndex(line_number))?;

    // second column: offset into file
    let start_offset = split.next().ok_or(MissingColumnInIndex(line_number))?;
    let start_offset = decode_number(start_offset)?;

    // get entry length
    let length = split.next().ok_or(MissingColumnInIndex(line_number))?;
    let length = decode_number(length)?;

    Ok((word.to_string(), start_offset, length))
}

/// Parse the index for a dictionary from a given BufRead compatible object.
pub fn parse_index<B: BufRead>(br: B) -> Result<Index, DictError> {
    let mut index = HashMap::new();

    for (line_number, line) in br.lines().enumerate() {
        let line = line?;
        let (word, start_offset, length) = parse_line(&line, line_number)?;
        index.entry(word.clone()).or_insert((start_offset, length));
    }

    Ok(index)
}

/// Parse the index for a dictionary from a given file name.
pub fn parse_index_from_file(filename: &str) -> Result<Index, DictError> {
    let file = File::open(filename)?;
    let file = BufReader::new(&file);
    parse_index(file)
}

