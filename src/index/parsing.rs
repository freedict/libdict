use super::{Entry, IndexError, metadata::MetadataIndex, Location};
use std::io::BufRead;
use IndexError::*;

#[derive(Default)]
struct Context {
    line: usize,
    pos: usize,
}

pub fn parse_metadata(reader: &mut impl BufRead) -> Result<MetadataIndex, IndexError> {
    let mut metadata = MetadataIndex::default();
    let mut ctx = Context::default();
    let mut line = String::new();
    let mut reading_info = false;

    while let Ok(num_read) = reader.read_line(&mut line) {
        if num_read == 0 { break }

        let entry = parse_line(&mut ctx, line.trim_end())?;

        let info_section = if entry.headword.starts_with("00-database-") {
            Some(&entry.headword[12..])
        } else if entry.headword.starts_with("00database") {
            Some(&entry.headword[10..])
        } else {
            None
        };

        if let Some(info_section) = info_section {
            match info_section {
                "info" => metadata.info = Some(entry.location),
                "short" => metadata.short_name = Some(entry.location),
                "url" => metadata.url = Some(entry.location),
                "allchars" => metadata.all_chars = true,
                s if s.contains("case") => metadata.case_sensitive = true,
                s if s.contains("dictfmt") => metadata.should_normalize = true,
                _ => {} // Ignore if there is an unsupported metadata entry
            }

            reading_info = true;
        } else {
            if reading_info { break }
        }

        line.clear();
    }

    Ok(metadata)
}

pub fn parse(reader: &mut impl BufRead) -> Result<Vec<Entry>, IndexError> {
    let mut ctx = Context::default();
    let mut entries = Vec::new();
    let mut line = String::new();

    while let Ok(num_read) = reader.read_line(&mut line) {
        if num_read == 0 { break }

        let entry = parse_line(&mut ctx, line.trim_end())?;
        line.clear();
        
        // Ignore metadata entries
        if entry.headword.starts_with("00") { continue }

        entries.push(entry);
    }

    Ok(entries)
}

fn parse_line(ctx: &mut Context, line: &str) -> Result<Entry, IndexError> {
    let mut split = line.split('\t');

    // 1st column
    let word = split.next().ok_or(MissingColumnInIndex(ctx.line))?;

    // 2nd column - offset into file
    ctx.pos = word.len();
    let s = split.next().ok_or(MissingColumnInIndex(ctx.line))?;
    let offset = decode_number(&ctx, s)?;

    // 3rd column - entry length
    ctx.pos += s.len();
    let s = split.next().ok_or(MissingColumnInIndex(ctx.line))?;
    let size = decode_number(&ctx, s)?;
    let location = Location { offset, size };

    // 4th column (optional) - original headword
    let original = split.next().map(String::from);

    // Advance context to new line
    ctx.line += 1;
    ctx.pos = 0;

    Ok(Entry { headword: word.into(), location, original })
}

fn decode_number(ctx: &Context, s: &str) -> Result<u64, IndexError> {
    let mut index = 0u64;
    for (i, ch) in s.chars().rev().enumerate() {
        index += get_base(ctx, ch)? * 64u64.pow(i as u32);
    }

    Ok(index)
}

fn get_base(ctx: &Context, ch: char) -> Result<u64, IndexError> {
    match ch {
        'A'..='Z' => Ok((ch as u64) - 65), // 'A' should become 0
        'a'..='z' => Ok((ch as u64) - 71), // 'a' should become 26
        '0'..='9' => Ok((ch as u64) + 4),  // 0 should become 52
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(InvalidCharacter(ch, ctx.line, ctx.pos)),
    }
}
