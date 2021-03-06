pub mod backtrack_parse;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Terminal {
    name: String,
}

impl Terminal {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Terminal {name: name.into()}
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NonTerminal {
    name: String,
}

impl NonTerminal {
    pub fn new<S: Into<String>>(name: S) -> Self {
        NonTerminal {name: name.into()}
    }

    pub fn fork(&self) -> NonTerminal {
        NonTerminal {
            name: self.name.clone() + "@",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Element {
    T(Terminal),
    NT(NonTerminal),
    Empty,
}

#[derive(Debug, Clone)]
pub struct Production {
    pub left: NonTerminal,
    pub right: Vec<Element>,
}

impl Production {
    fn new(left: NonTerminal, right: Vec<Element>) -> Self {
        Production { left, right }
    }
}

#[derive(Debug, Clone)]
pub struct ProdBlock {
    pub left: NonTerminal,
    pub productions: Vec<Production>,
}

impl ProdBlock {
    pub fn new(left: NonTerminal, productions: Vec<Production>) -> Self {
        ProdBlock { left, productions }
    }
}

#[derive(Debug, Clone)]
pub struct CFG {
    pub start: NonTerminal,
    pub non_terminals: Vec<NonTerminal>,
    pub terminals: Vec<Terminal>,
    pub productions: Vec<ProdBlock>,
}

#[derive(Debug, Clone)]
pub struct CFGWithoutLeftRecursion {
    cfg: CFG,
}

pub struct ParseTree;
pub struct Error;

///////////////////////// eliminate left recursion /////////////////////////////////////////////////

pub fn eliminate_direct_left_recursion(block: ProdBlock) -> (ProdBlock, Option<ProdBlock>) {
    let left = block.left.clone();
    let (mut recursive, mut non_recursive): (Vec<_>, _) = block
        .productions
        .into_iter()
        .partition(|p| p.right[0] == left);

    if recursive.is_empty() {
        let blk = ProdBlock::new(left, non_recursive);
        return (blk, None);
    }

    assert!(!non_recursive.is_empty());

    let left_ext = left.fork();

    for prod in &mut non_recursive {
        prod.right.push(Element::NT(left_ext.clone()));
    }
    let left_block = ProdBlock::new(left.clone(), non_recursive);

    for prod in &mut recursive {
        let mut elements: Vec<_> = prod.right.drain(1..).collect();
        elements.push(Element::NT(left_ext.clone()));
        prod.right = elements;
    }
    recursive.push(Production::new(left_ext, vec![Element::Empty]));
    let right_block = ProdBlock::new(left, recursive);

    (left_block, Some(right_block))
}

pub fn eliminate_left_recursion(mut cfg: CFG) -> CFGWithoutLeftRecursion {
    if cfg.productions.len() == 1 {
        return CFGWithoutLeftRecursion { cfg };
    }

    let mut new_pbs = vec![];
    let mut forks = vec![];

    for i in 0..cfg.productions.len() {
        let new_pb = replace_multi(&cfg.productions[..i], cfg.productions[i].clone());
        let (l, r) = eliminate_direct_left_recursion(new_pb);
        new_pbs.push(l);
        if let Some(fork) = r {
            cfg.non_terminals.push(fork.left.clone());
            forks.push(fork);
        }
    }

    new_pbs.extend(forks);
    CFGWithoutLeftRecursion { cfg }
}

fn replace_multi(lhs: &[ProdBlock], rhs: ProdBlock) -> ProdBlock {
    lhs.iter().fold(rhs, |acc, lhs| replace_pb(lhs, acc))
}

fn replace_pb(lhs: &ProdBlock, rhs: ProdBlock) -> ProdBlock {
    let mut productions = vec![];
    let left = rhs.left;
    for prod in rhs.productions {
        productions.extend(replace_prod(lhs, prod))
    }

    ProdBlock::new(left, productions)
}

fn replace_prod(lhs: &ProdBlock, prod: Production) -> Vec<Production> {
    let mut ret = vec![];
    if prod.right[0] == lhs.left {
        for prod in &lhs.productions {
            let mut replica = prod.right.clone();
            replica.extend_from_slice(&prod.right[1..]);
            ret.push(Production::new(prod.left.clone(), replica));
        }
    } else {
        ret.push(prod)
    }
    ret
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        match (self, other) {
            (Element::T(l), Element::T(r)) if l.name == r.name => true,
            (Element::NT(l), Element::NT(r)) if l.name == r.name => true,
            (Element::Empty, Element::Empty) => true,
            _ => false,
        }
    }
}

impl PartialEq<NonTerminal> for Element {
    fn eq(&self, other: &NonTerminal) -> bool {
        match self {
            Element::NT(nt) if nt.name == other.name => true,
            _ => false,
        }
    }
}

impl PartialEq<Terminal> for Element {
    fn eq(&self, other: &Terminal) -> bool {
        match self {
            Element::T(t) if t.name == other.name => true,
            _ => false,
        }
    }
}

impl Eq for Element {}
