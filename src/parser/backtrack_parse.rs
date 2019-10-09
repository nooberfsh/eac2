use crate::parser::{Error, ParseTree, Terminal, CFG};

pub fn backtrack_parse(_cfg: &CFG, _tokens: &[Terminal]) -> Result<ParseTree, Error> {
    unimplemented!()
}
