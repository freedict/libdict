use std::collections::HashMap;
use std::io::BufRead;


fn get_base(input: char) -> Result<i64, String> {
    return match input {
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
    return Ok(index);
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

    return Ok((word.to_string(), start_offset, length));
}

pub fn parse_index<B: BufRead>(br: B) -> Box<HashMap<String, (i64, i64)>> {
    let mut index = Box::new(HashMap::new());

    for line in br.lines() {
        let l = &line.unwrap(); // ToDo: how to handle properly
        let (word, start_offset, length) = parse_line(l).unwrap(); // toDo: handle properly
        *index.entry(word.clone()).or_insert((start_offset, length));
    }

    return index;
}

