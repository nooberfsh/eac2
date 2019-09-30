use crate::parser::{Error, NonTerminal, ParseTree, Terminal, CFG};

pub fn backtrack_parse(cfg: &CFG, tokens: &[Terminal]) -> Result<ParseTree, Error> {
    unimplemented!()
}
