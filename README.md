LibDict -- Read *.dict(.dz) dictionary files
============================================

This is an attempt to provide an library implementation for reading the
dict(.dz) dictionary format. It aims at providing efficient means to look up
search terms from a given dictionary.

The plan is as follows:

1.  parse the format
2.  provide a convenient wrapper type, bundling all functionality in a
    user-friendly interface
    -   allow lookup of words
    -   allow retrieving a certain number of words starting with a specific
        prefix
3.  add gzip support
4.  add caching support (cache words which have been looked up)
5.  provide C API


License
-------

Please see the file `LiCENSE.md` for more information.

