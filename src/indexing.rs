// ToDo: the whole file needs proper error handling, more precisely, the functions should return
// Options or results


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

