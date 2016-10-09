// ToDo: the whole file needs proper error handling, more precisely, the functions should return
// Options or results


fn get_base(input: char) -> i64 {
    return match input {
        'A' ... 'Z' => (input as i64) - 65, // 'A' should become 0
        'a' ... 'z' => (input as i64) - 71, // 'a' should become 26, ...
        '0' ... '9' => (input as i64) + 4, // 0 should become 52
        '+' => 62,
        '/' => 63,
        _ => panic!("Unimplemented character."),
    }
}

pub fn get_offset(word: &str) -> i64 {
    return word.chars().rev().enumerate()
        .map(|(i, character)| get_base(character) * (64 as i64).pow((i as u32))).sum();
}

