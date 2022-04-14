mod parsing;

use crate::DictError;
use std::{collections::HashMap, io::{Seek, Read, BufReader}};

pub struct Index {
    pub words: HashMap<String, (u64, u64)>,
}

impl Index {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self, DictError> {
        let buf_reader = BufReader::new(reader);
        parsing::parse(buf_reader)
    }
}

