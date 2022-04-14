use super::Index;
use crate::DictError;
use std::collections::HashMap;
use std::io::BufRead;

#[derive(Default)]
struct Context {
    line: usize,
    pos: usize,
}

pub fn parse<R: BufRead>(reader: R) -> Result<Index, DictError> {
    let mut ctx = Context::default();
    let mut words = HashMap::new();

    for line in reader.lines() {
        let (word, start_offset, length) = parse_line(&mut ctx, line?)?;
        words.insert(word, (start_offset, length));
    }

    Ok(Index { words })
}

fn parse_line(ctx: &mut Context, line: String) -> Result<(String, u64, u64), DictError> {
    let mut split = line.split('\t');

    // 1st column
    let word = split
        .next()
        .ok_or(DictError::MissingColumnInIndex(ctx.line))?;

    // 2nd column - offset into file
    ctx.pos = word.len();
    let s = split
        .next()
        .ok_or(DictError::MissingColumnInIndex(ctx.line))?;
    let start_offset = decode_number(&ctx, s)?;

    // 3rd column - entry length
    ctx.pos += s.len();
    let s = split
        .next()
        .ok_or(DictError::MissingColumnInIndex(ctx.line))?;
    let length = decode_number(&ctx, s)?;

    // Advance context to new line
    ctx.line += 1;
    ctx.pos = 0;

    Ok((word.into(), start_offset, length))
}

fn decode_number(ctx: &Context, s: &str) -> Result<u64, DictError> {
    let mut index = 0u64;
    for (i, ch) in s.chars().rev().enumerate() {
        index += get_base(ctx, ch)? * 64u64.pow(i as u32);
    }

    Ok(index)
}

fn get_base(ctx: &Context, ch: char) -> Result<u64, DictError> {
    match ch {
        'A'..='Z' => Ok((ch as u64) - 65), // 'A' should become 0
        'a'..='z' => Ok((ch as u64) - 71), // 'a' should become 26
        '0'..='9' => Ok((ch as u64) + 4),  // 0 should become 52
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(DictError::InvalidCharacter(ch, ctx.line, ctx.pos)),
    }
}
