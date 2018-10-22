use std::collections::HashMap;

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct Terminal;
#[derive(Eq, PartialEq, Clone, Hash)]
pub struct NoneTerminal;

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum Token {
    T(Terminal),
    NT(NoneTerminal),
}

impl Token {
    fn is_terminal(&self) -> bool {
        match self {
            Token::T(_) => true,
            _ => false,
        }
    }

    fn is_non_terminal(&self) -> bool {
        match self {
            Token::NT(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Production {
    non_terminal: NoneTerminal,
    tokens: Vec<Token>,
}

pub struct CFG {
    terminals: Vec<Terminal>,
    non_terminals: Vec<NoneTerminal>,
    productions: HashMap<NoneTerminal, Vec<Production>>,
    start: NoneTerminal,
}

pub struct NoneLeftRecursionCFG(CFG);

impl CFG {
    pub fn into_non_left_recursion(mut self) -> NoneLeftRecursionCFG {
        let mut prods = HashMap::new();
        for nt in &self.non_terminals {
            let mut ps = vec![];
            for prod in self.productions.get(nt).unwrap() {
                match &prod.tokens[0] {
                    Token::NT(nt) => {
                        if self.is_preceed(nt, nt) {
                            let processed = prods.get(nt).unwrap();
                            let new = replace(prod, &processed);
                            ps.extend(new);
                        }
                    }
                    _ => continue,
                }
            }
            let (lhs, rhs) = eliminate_direct_left_recursion(ps);
            prods.insert(nt.clone(), lhs);
            prods.insert(rhs[0].non_terminal.clone(), rhs);
        }
        self.productions = prods;
        NoneLeftRecursionCFG(self)
    }

    fn is_preceed(&self, lhs: &NoneTerminal, rhs: &NoneTerminal) -> bool {
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

fn fork(nt: &NoneTerminal) -> NoneTerminal {
    unimplemented!()
}

fn new_empty_terminal() -> Terminal {
    unimplemented!()
}

impl Terminal {
    fn new(c: char) -> Self {unimplemented!()}
    fn empty() -> Self {unimplemented!()}
    fn eof() -> Self {unimplemented!()}
}

fn replace(prod: &Production, prods: &Vec<Production>) -> Vec<Production> {
    let mut ret = prods.clone();
    for p in &mut ret {
        p.tokens.extend_from_slice(&prod.tokens[1..]);
        p.non_terminal = prod.non_terminal.clone();
    }
    ret
}

fn eliminate_direct_left_recursion(prods: Vec<Production>) -> (Vec<Production>, Vec<Production>) {
    let (mut lret, mut rret) = (vec![], vec![]);
    let nt = prods[0].non_terminal.clone();
    for p in prods {
        if p.tokens[0] == Token::NT(nt.clone()) {
            rret.push(p)
        } else {
            lret.push(p)
        }
    }
    let new_nt = fork(&nt);
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
    (lret, rret)
}

pub type First = HashMap<Token, Vec<Terminal>>;
pub type Follow = HashMap<NoneTerminal, Vec<Terminal>>;

pub fn first(cfg: &NoneLeftRecursionCFG) -> First {
    let cfg = &cfg.0;
    let mut first = First::new();
    for t in cfg.terminals.clone() {
        first.insert(Token::T(t.clone()), vec![t]);
    }
    first.insert(Token::T(Terminal::empty()), vec![Terminal::empty()]);
    first.insert(Token::T(Terminal::eof()), vec![Terminal::eof()]);
    let mut updated = true;
    let productions: Vec<_> = cfg.productions.values().flatten().collect();
    while updated {
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
            if insert_and_check_if_updated(&mut first, &key, rhs) {
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
        for p in &productions {
            let mut trailer = first.get(&Token::NT(p.non_terminal.clone())).unwrap();
            for t in p.tokens.iter().rev() {
                match t {
                    Token::NT(nt) => {
                        if follow_insert_and_check_if_updated(&mut follow, nt, trailer) {
                            updated = true;
                        }
                        let trai = first.get(t).clone().unwrap();
                        if exist_empty(follow.get(nt).unwrap()) {
                            remove_empty(&mut trai);
                            trailer.extend(trai);
                        } else {
                            trailer = trai;
                        }
                    }
                    _ => trailer = first.get(t).clone().unwrap(),
                }
            }
        }
    }
    follow
}

fn exist_empty(ts: &Vec<Terminal>) -> bool {
    for t in ts {
        if t == &Terminal::empty() {
            return true
        }
    }
    false
}

fn remove_empty(ts: &mut Vec<Terminal>) {
    ts.remove_item(&Terminal::empty());
}

fn first_insert_and_check_if_updated(first: &mut First, key: &Token, rhs: Vec<Terminal>) -> bool {
    let first = first.get_mut(key).unwrap();
    unimplemented!()
}

fn follow_insert_and_check_if_updated(follow: &mut Follow, key: &NoneTerminal, trailer: &Vec<Terminal>) {
    unimplemented!()
}
