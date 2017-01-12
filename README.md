LibDict -- Read *.dict(.dz) dictionary files
============================================

This crate (library) provides a Rust implementation for parsing the dict file
format, as used by the dictd server.


License
-------

Please see the file [LiCENSE.md](LICENSE.md) for more information.

Example Usage
-------------

Citing from the crates documentation:

```
fn main() {
    let index_file = "/usr/share/dictd/freedict-lat-deu.index";
    let dict_file = "/usr/share/dictd/freedict-lat-deu.dict.dz";
    let mut latdeu = dict::load_dictionary_from_file(dict_file, index_file).unwrap();
    // hey: rust!
    println!("{}", latdeu.lookup("ferrugo").unwrap());
}

```

