use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

pub mod indexing;


pub fn main() {
    let f = File::open("german-english.dict").unwrap();
    let file = BufReader::new(&f);
    let index = indexing::parse_index(file);
}

