pub mod backtrack_parse;


pub struct Terminal;
pub struct NonTerminal;

pub enum Element {
    T(Terminal),
    NT(NonTerminal),
    Empty
}

pub struct Production {
    pub left: NonTerminal,
    pub right: Vec<Element>,
}

pub struct ProdBlock {
    pub left: NonTerminal,
    pub productions: Vec<Production>,
}

pub struct CFG {
    pub start: NonTerminal,
    pub non_terminals: Vec<NonTerminal>,
    pub terminals: Vec<Terminal>,
    pub producttions: Vec<ProdBlock>,
}

pub struct ParseTree;
pub struct Error;
