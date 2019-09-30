use crate::parser::{CFG, Terminal, ParseTree, Error, NonTerminal};

pub fn backtrack_parse(cfg: &CFG, tokens: &[Terminal]) -> Result<ParseTree, Error> {
    unimplemented!()
}

