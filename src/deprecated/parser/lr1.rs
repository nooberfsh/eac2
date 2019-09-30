use std::collections::BTreeSet;
use std::collections::HashMap;

use super::*;

pub type State = usize;
type ItemSet = Vec<Item>;
type CanonicalCollection = Vec<ItemSet>;
type GotoTable = HashMap<(State, NoneTerminal), State>;
type ActionTable = HashMap<(State, Terminal), Action>;
type StateTransfer = HashMap<(State, Token), State>;

pub struct Error;

type Result = std::result::Result<(), Error>;

pub enum Action {
    Reduce(Production),
    Shift(State),
    Accept
}

enum Step {
    Token(Token),
    State(State),
}

pub fn build_action_and_goto_table(cfg: &CFG) -> (ActionTable, GotoTable) {
    let first = first(cfg);
    let (cc, transfer) = build_cc(cfg, &first);
    let (mut action, mut goto) = (ActionTable::new(), GotoTable::new());
    for (i, items) in cc.iter().enumerate() {
        for item in items {
            if item.right.is_empty() {
                let v = if item.nt == NoneTerminal::start() && item.lookahead == Terminal::eof() {
                    Action::Accept
                } else {
                    Action::Reduce(Production::new(item.nt.clone(), item.left.clone()))
                };
                action.insert((i, item.lookahead.clone()), v);
                continue;
            }
            if let Token::T(t) = &item.right[0] {
                let s = transfer.get(&(i, item.right[0].clone())).unwrap();
                action.insert((i, t.clone()), Action::Shift(*s));
            }
        }
        for nt in &cfg.non_terminals {
            if let Some(to) = transfer.get(&(i, Token::NT(nt.clone()))) {
                goto.insert((i, nt.clone()), *to);
            }
        }
    }
    (action, goto)
}

pub fn parse(tokens: &[Terminal], cfg: &CFG) -> Result {
    let (action, goto) = build_action_and_goto_table(cfg);
    let mut stack = vec![Step::State(0)];
    let mut tokens = Tokens::new(tokens);
    let mut s = 0;
    loop {
        let mut token = tokens.current();
        let act = match action.get(&(s, token.clone())) {
            Some(a) => a,
            _ => return Err(Error),
        };
        match act {
            Action::Accept => break,
            Action::Reduce(p) => {
                for _ in 0..(2*p.tokens.len()) {
                    stack.pop();
                }
                let nt = p.non_terminal.clone();
                s = match &stack[stack.len() - 1] {
                    Step::State(s) => *s,
                    _ => unreachable!(),
                };
                stack.push(Step::Token(Token::NT(nt.clone())));
                s = *goto.get(&(s, nt.clone())).unwrap();
                stack.push(Step::State(s));
            }
            Action::Shift(to) => {
                stack.push(Step::Token(Token::T(token.clone())));
                stack.push(Step::State(*to));
                s = *to;
                tokens.forward();
            }
        }
    }
    Ok(())
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Clone)]
struct Item {
    nt: NoneTerminal,
    left: Vec<Token>,
    right: Vec<Token>,
    lookahead: Terminal,
}

fn closure(items: &mut ItemSet, cfg: &CFG, first: &First) {
    let mut updated = true;
    while updated {
        updated = false;
        let mut generated = ItemSet::new();
        for item in &mut items.clone() {
            let right = &item.right;
            if right.is_empty() {
                continue;
            }

            let nt = match &right[0] {
                Token::NT(nt) => nt,
                _ => continue,
            };
            let lookahead = Token::T(item.lookahead.clone());
            let follow_token = if right.len() > 1 {
                vec![right[1].clone(), lookahead]
            } else {
                vec![lookahead]
            };
            let first = first_of_tokens(&follow_token, first);
            for p in cfg.productions.get(nt).unwrap() {
                for t in &first {
                    let item = Item {
                        nt: nt.clone(),
                        left: vec![],
                        right: p.tokens.clone(),
                        lookahead: t.clone(),
                    };
                    generated.push(item);
                }
            }
        }
        updated = append(items, generated)
    }
}

fn goto(items: &ItemSet, token: &Token, cfg: &CFG, first: &First) -> ItemSet {
    let mut ret = ItemSet::new();
    for item in items {
        if item.right.is_empty() || &item.right[0] != token {
            continue;
        }
        let mut item = item.clone();
        item.left.push(item.right[0].clone());
        item.right = Vec::from(&item.right[1..]);
        ret.push(item);
    }
    closure(&mut ret, cfg, first);
    ret
}

fn build_cc(cfg: &CFG, first: &First) -> (CanonicalCollection, StateTransfer) {
    let mut cc = CanonicalCollection::new();
    let mut transfer = StateTransfer::new();
    let seed = cfg
        .productions
        .get(&NoneTerminal::start())
        .unwrap()
        .clone()
        .into_iter()
        .map(|p| Item {
            nt: p.non_terminal,
            left: vec![],
            right: p.tokens,
            lookahead: Terminal::eof(),
        })
        .collect();
    cc.push(seed);
    
    let mut updated = true;
    let mut start_idx = 0;
    while updated {
        updated = false;
        let mut generated = CanonicalCollection::new();
        for (i, items) in cc.iter().enumerate().skip(start_idx) {
            let next_tokens = next_tokens(items);
            for token in &next_tokens {
                let gen = goto(items, token, cfg, first);
                let key = (i, token.clone());
                if let Some(state) = find(&cc, &gen) {
                    transfer.insert(key, state);
                } else if let Some(state) = find(&generated, &gen) {
                    transfer.insert(key, cc.len() + state);
                } else {
                    transfer.insert(key, cc.len() + generated.len());
                    generated.push(gen);
                }
            }
        }
        if !generated.is_empty() {
            start_idx += cc.len();
            cc.extend(generated);
            updated = true;
        }
    }
    (cc, transfer)
}

fn append(dst: &mut ItemSet, src: ItemSet) -> bool {
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

fn next_tokens(items: &ItemSet) -> Vec<Token> {
    items.iter().filter_map(|item|item.right.get(0)).map(|t|t.clone()).collect()
}

fn find(cc: &CanonicalCollection, items: &ItemSet) -> Option<usize> {
    for (i, c) in cc.iter().enumerate() {
        let l: BTreeSet<_>= c.iter().collect();
        let r: BTreeSet<_> = items.iter().collect();
        if l == r {
            return Some(i)
        }
    }
    None
}

struct Tokens<'a> {
    tokens: &'a [Terminal],
    idx: usize,
    eof: Terminal,
}

impl<'a> Tokens<'a> {
    fn new(tokens: &'a[Terminal]) -> Self {
        Tokens {
            tokens: tokens,
            idx: 0,
            eof: Terminal::eof(),
        }
    }

    fn forward(&mut self) {
        if self.idx != self.tokens.len() {
            self.idx += 1;
        }
    }

    fn current(&self) -> &Terminal {
        if self.idx == self.tokens.len() {
            &self.eof
        } else {
            &self.tokens[self.idx]
        }
    }
}
