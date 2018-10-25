pub mod recursive_descent;
pub mod ll1;

use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Eq, PartialEq, Clone, Hash, Ord, PartialOrd, Debug)]
pub struct Terminal(String);
#[derive(Eq, PartialEq, Clone, Hash, Ord, PartialOrd, Debug)]
pub struct NoneTerminal(String);

#[derive(Eq, PartialEq, Clone, Hash, Debug, Ord, PartialOrd)]
pub enum Token {
    T(Terminal),
    NT(NoneTerminal),
}

impl Token {
    pub fn is_terminal(&self) -> bool {
        match self {
            Token::T(_) => true,
            _ => false,
        }
    }

    pub fn is_non_terminal(&self) -> bool {
        match self {
            Token::NT(_) => true,
            _ => false,
        }
    }

    pub fn is_eof(&self) -> bool {
        match self {
            Token::T(t) => t.is_eof(),
            _ => false
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Token::T(t) => t.is_empty(),
            _ => false
        }
    }
}
impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for NoneTerminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Production {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Token::NT(nt) => write!(f, "{}", nt),
            Token::T(t) => write!(f, "{}", t),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialOrd, Ord, PartialEq)]
pub struct Production {
    non_terminal: NoneTerminal,
    tokens: Vec<Token>,
}

impl Production {
    pub fn new(nt: NoneTerminal, tokens: Vec<Token>) -> Self {
        Production {
            non_terminal: nt,
            tokens
        }
    }

    fn to_string(&self) -> String {
        let mut ret = self.non_terminal.0.clone();
        ret += " -> ";
        for t in &self.tokens {
            let s = match t {
                Token::T(t) => &t.0,
                Token::NT(nt) => &nt.0,
            };
            ret += s;
            ret += " ";
        }
        ret
    }
}

#[derive(Debug)]
pub struct CFG {
    pub terminals: Vec<Terminal>,
    pub non_terminals: Vec<NoneTerminal>,
    pub productions: HashMap<NoneTerminal, Vec<Production>>,
    pub start: NoneTerminal,
}

#[derive(Debug)]
pub struct NoneLeftRecursionCFG(pub CFG);

impl CFG {
    pub fn new(prods: HashMap<NoneTerminal, Vec<Production>>) -> Self {
        let mut ts = HashSet::new();
        let mut nts = HashSet::new();
        for (nt, _) in &prods {
            nts.insert(nt.clone());
        }

        for (_, ps) in &prods {
            for p in ps {
                for t in &p.tokens {
                    if let Token::T(t) = t {
                        ts.insert(t.clone());
                    }
                }
            }
        }
        CFG {
            terminals: ts.into_iter().collect(),
            non_terminals: nts.into_iter().collect(),
            productions: prods,
            start: NoneTerminal::start(),
        }
    }

    pub fn into_non_left_recursion(self) -> NoneLeftRecursionCFG {
        let mut prods = HashMap::new();
        for current in &self.non_terminals {
            let mut ps = vec![];
            for prod in self.productions.get(current).unwrap() {
                match &prod.tokens[0] {
                    Token::NT(nt) if self.is_preceed(current, nt) => {
                            let processed = prods.get(nt).unwrap();
                            let new = replace(prod, &processed);
                            ps.extend(new);
                    }
                    _ => ps.push(prod.clone())
                }
            }
            let (lhs, rhs) = eliminate_direct_left_recursion(ps);
            prods.insert(current.clone(), lhs);
            if let Some(rhs) = rhs {
                prods.insert(rhs[0].non_terminal.clone(), rhs);
            }
        }
        NoneLeftRecursionCFG(Self::new(prods))
    }

    pub fn to_string(&self) -> String {
        let mut ret = String::new();
        for ps in self.productions.values() {
            for p in ps {
                ret += &p.to_string();
                ret += "\n";
            }
            ret += "\n";
        }
        ret
    }

    fn is_preceed(&self, lhs: &NoneTerminal, rhs: &NoneTerminal) -> bool {
        if lhs == rhs {
            return false
        }
        for nt in &self.non_terminals {
            if nt == rhs {
                return true;
            } else if nt == lhs {
                return false;
            }
        }
        unreachable!()
    }
}

impl NoneTerminal {
    pub fn new<T: Into<String>>(t: T) -> Self {
        NoneTerminal(t.into())
    }

    pub fn fork(&self) -> Self {
        let s = self.0.clone() + "@";
        Self::new(s)
    }

    pub fn start() -> Self {
        Self::new("Goal")
    }
}

impl Terminal {
    pub fn new<T: Into<String>>(t: T) -> Self {
        Terminal(t.into())
    }

    pub fn empty() -> Self {
        Self::new("empty@@")
    }

    pub fn is_empty(&self) -> bool {
        self.0 == "empty@@"
    }

    pub fn eof() -> Self {
        Self::new("eof@@")
    }

    pub fn is_eof(&self) -> bool {
        self.0 == "eof@@"
    }
}

pub fn terminal<T: Into<String>>(t: T) -> Terminal {
    Terminal(t.into())
}

pub fn none_terminal<T: Into<String>>(t: T) -> NoneTerminal {
    NoneTerminal(t.into())
}

fn replace(prod: &Production, replicas: &Vec<Production>) -> Vec<Production> {
    let mut ret = replicas.clone();
    for p in &mut ret {
        p.tokens.extend_from_slice(&prod.tokens[1..]);
        p.non_terminal = prod.non_terminal.clone();
    }
    ret
}

fn eliminate_direct_left_recursion(prods: Vec<Production>) -> (Vec<Production>, Option<Vec<Production>>) {
    let (mut lret, mut rret) = (vec![], vec![]);
    let nt = prods[0].non_terminal.clone();
    for p in prods {
        if p.tokens[0] == Token::NT(nt.clone()) {
            rret.push(p)
        } else {
            lret.push(p)
        }
    }
    if rret.is_empty() {
        return (lret, None);
    }
    let new_nt = nt.fork();
    for p in &mut rret {
        p.non_terminal = new_nt.clone();
        let mut tokens: Vec<_> = p.tokens.drain(1..).collect();
        tokens.push(Token::NT(new_nt.clone()));
        p.tokens = tokens
    }
    let e = Production {
        non_terminal: new_nt.clone(),
        tokens: vec![Token::T(Terminal::empty())],
    };
    rret.push(e);

    for p in &mut lret {
        p.tokens.push(Token::NT(new_nt.clone()))
    }
    (lret, Some(rret))
}

pub type First = HashMap<Token, Vec<Terminal>>;
pub type Follow = HashMap<NoneTerminal, Vec<Terminal>>;
pub type Predict = HashMap<(NoneTerminal, usize), Vec<Terminal>>;

pub fn predict(cfg: &NoneLeftRecursionCFG, first: &First, follow: &Follow) -> Predict {
    let cfg = &cfg.0;
    let mut predict = HashMap::new();
    for nt in &cfg.non_terminals {
        for (i, p) in cfg.productions.get(nt).unwrap().into_iter().enumerate() {
            let mut f = first_of_production(p, first) ;
            if exist_empty(&f) {
                let fol = follow.get(nt).unwrap().clone();
                f.extend(fol);
            }
            predict.insert((nt.clone(), i), f);
        }
    }
    predict
}

fn first_of_production(p: &Production, first: &First) -> Vec<Terminal> {
    let mut ret = vec![];
    for token in &p.tokens {
        let mut f = first.get(token).unwrap().clone();
        if !exist_empty(&f) {
            ret.extend(f);
            return ret;
        }
        remove_empty(&mut f);
        ret.extend(f)
    }
    ret.push(Terminal::empty());
    ret
}

pub fn first(cfg: &NoneLeftRecursionCFG) -> First {
    let cfg = &cfg.0;
    let mut first = First::new();
    for nt in cfg.non_terminals.clone() {
        first.insert(Token::NT(nt.clone()), vec![]);
    }
    for t in cfg.terminals.clone() {
        first.insert(Token::T(t.clone()), vec![t]);
    }
    first.insert(Token::T(Terminal::empty()), vec![Terminal::empty()]);
    first.insert(Token::T(Terminal::eof()), vec![Terminal::eof()]);
    let mut updated = true;
    let productions: Vec<_> = cfg.productions.values().flatten().collect();
    while updated {
        updated = false;
        for &p in &productions {
            let mut rhs = vec![];
            let key = Token::NT(p.non_terminal.clone());
            let tokens = &p.tokens;     
            let mut fir = first.get(&tokens[0]).unwrap();
            let mut f = fir.clone();
            remove_empty(&mut f);
            rhs.extend(f);
            let mut idx = 0;
            while exist_empty(fir) && idx < tokens.len() - 1 {
                let mut f = first.get(&tokens[idx + 1]).unwrap().clone();
                remove_empty(&mut f);
                rhs.extend(f);
                idx += 1;
                fir = first.get(&tokens[idx]).unwrap();
            }
            if idx == tokens.len() - 1 && exist_empty(first.get(&tokens[idx]).unwrap()) {
                rhs.push(Terminal::empty())
            }
            if append(first.get_mut(&key).unwrap(), rhs) {
                updated = true
            }
        }
    }
    first
}

pub fn follow(cfg: &NoneLeftRecursionCFG, first: &First) -> Follow {
    let cfg = &cfg.0;
    let mut follow: HashMap<_, _>= cfg.non_terminals.clone().into_iter().map(|nt| (nt, vec![])).collect();
    follow.insert(cfg.start.clone(), vec![Terminal::eof()]);

    let mut updated = true;
    let productions: Vec<_> = cfg.productions.values().flatten().collect();

    while updated {
        updated = false;
        for p in &productions {
            let mut trailer = follow.get(&p.non_terminal).unwrap().clone();
            for token in p.tokens.iter().rev() {
                match token {
                    Token::NT(nt) => {
                        if append(follow.get_mut(nt).unwrap(), trailer.clone()) {
                            updated = true
                        }
                        let mut trai = first.get(token).unwrap().clone();
                        if exist_empty(&trai) {
                            remove_empty(&mut trai);
                            trailer.extend(trai);
                        } else {
                            trailer = trai;
                        }
                    }
                    _ => trailer = first.get(token).unwrap().clone(),
                }
            }
        }
    }
    follow
}

fn exist_empty(ts: &Vec<Terminal>) -> bool {
    for t in ts {
        if t.is_empty() {
            return true
        }
    }
    false
}

fn remove_empty(ts: &mut Vec<Terminal>) {
    ts.remove_item(&Terminal::empty());
}

fn append(dst: &mut Vec<Terminal>, src: Vec<Terminal>) -> bool {
    let mut ret = false;  
    let mut hashed: HashSet<_> = dst.clone().into_iter().collect();
    for t in src {
        if hashed.get(&t).is_none() {
            dst.push(t.clone());
            hashed.insert(t);
            ret = true;
        }
    }
    ret
}

#[cfg(test)]
mod test;
