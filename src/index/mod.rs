mod parsing;
pub use parsing::parse;

use crate::DictError;
use std::{collections::HashMap, io::BufRead};

pub struct Index {
    pub words: HashMap<String, (u64, u64)>,
}

impl Index {
    pub fn new<R: BufRead>(reader: R) -> Result<Self, DictError> {
        parsing::parse(reader)
    }
}
