use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::fs::File;


fn get_base(input: char) -> Result<i64, String> {
    match input {
        'A' ... 'Z' => Ok((input as i64) - 65), // 'A' should become 0
        'a' ... 'z' => Ok((input as i64) - 71), // 'a' should become 26, ...
        '0' ... '9' => Ok((input as i64) + 4), // 0 should become 52
        '+' => Ok(62),
        '/' => Ok(63),
        x @ _ => Err(format!("Unknown character {}", x)),
    }
}

pub fn get_offset(word: &str) -> Result<i64, String> {
    let mut index = 0i64;
    for (i, character) in word.chars().rev().enumerate() {
        index += match get_base(character) {
            Ok(x) => x * 64i64.pow(i as u32),
            Err(_) => return Err(format!("Invalid character {} at position {}", character, i)),
        };
    }
    Ok(index)
}

fn parse_line(line: &str) -> Result<(String, i64, i64), String> {
    let mut split = line.split("\t");
    let word = try!(match split.next() {
        Some(x) => Ok(x.to_string()),
        None => Err("Unable to find a \\t delimiter in this line")
    });
    // second column: offset into file
    let start_offset = try!(match split.next(){
        Some(x) => Ok(x),
        None => Err("Unable to find a \\t delimiter in this line")
    });
    let start_offset = try!(get_offset(start_offset));

    // get entry length
    let length = try!(match split.next() {
        Some(x) => Ok(x),
        None => Err("Unable to find a second \\t delimiter in this line")
    });
    let length = try!(get_offset(length));

    Ok((word.to_string(), start_offset, length))
}

pub fn parse_index<B: BufRead>(br: B) -> Result<HashMap<String, (i64, i64)>, String> {
    let mut index = HashMap::new();

    for line in br.lines() {
        //let line = try!(line);
        let line = line.unwrap();
        let (word, start_offset, length) = try!(parse_line(&line));
        index.entry(word.clone()).or_insert((start_offset, length));
    }

    Ok(index)
}

pub fn parse_index_from_file(filename: String) -> Result<HashMap<String, (i64, i64)>, String> {
    let file = File::open(filename).unwrap();
    let file = BufReader::new(&file);
    parse_index(file)
}

