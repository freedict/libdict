This is an attempt to provide an library implementation for reading the dict.dz
format. It aims at providing efficient means to look up words in a dictionary in
the dict format, using the .index file to speed up the lookup process.

The plan is as follows:

1.  parse the format
2.  provide a convenient wrapper type, bundling all functionality in a
    user-friendly interface
    -   allow lookup of words
    -   allow retrieving a certain number of words starting with a specific
        prefix
3.  add caching support (cache words which have been looked up)
4.  provide C API

The dict format is not officially documented. As a first step, a python
implementation tries to re-engineer the format and a proper Rust implementation
is derived from this.

