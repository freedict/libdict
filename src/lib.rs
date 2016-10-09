use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

pub mod indexing;


pub fn main() {
    //let word2index = HashMap::new();
    let f = File::open("german-english.dict").unwrap();
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let mut split = l.split("\t");
        let word = match split.next() {
            Some(x) => x,
            _ => continue,
        };
        let word = split.next().unwrap();
        let begin_offset = indexing::get_offset(split.next().unwrap());
        let length = indexing::get_offset(split.next().unwrap());
        println!("'{}' starts at {} and has a length of {} bytes", word, begin_offset.unwrap(), length.unwrap());
    }
}

