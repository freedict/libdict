use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

fn main() {
    let word2index = HashMap::new();
    let mut f = File::open("german-english.dict").unwrap();
    let mut file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let foo = l.split("\t")[0].clone();
        word2index
        println!("{}", l);
    }
}

//#[cfg(test)]
//mod tests {
//    #[test]
//    fn it_works() {
//    }
//}
