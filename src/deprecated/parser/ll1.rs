use std::collections::HashMap;

use super::*;

type Table<'a> = HashMap<NoneTerminal, HashMap<Terminal, Option<&'a Production>>>;

pub struct Error;

type Result = std::result::Result<(), Error>;

pub fn parse(tokens: &[Terminal], cfg: &NoneLeftRecursionCFG) -> Result {
    let table = contruct_talbe(cfg);
    let mut stack = vec![];
    stack.push(Token::T(Terminal::eof()));
    stack.push(Token::NT(NoneTerminal::start()));
    let mut tokens = Tokens::new(tokens);

    while let Some(token) = stack.pop() {
        let tok = tokens.current();
        if token.is_eof() && tok.is_eof() {
            return Ok(())
        } else if token.is_eof() || tok.is_eof() {
            return Err(Error)
        }
        match token {
            Token::NT(nt) => {
                if let &Some(p) = table.get(&nt).unwrap().get(tok).unwrap() {
                    if !p.tokens[0].is_empty() {
                        stack.extend(p.tokens.clone().into_iter().rev());
                    }
                } else {
                    return Err(Error)
                }
            }
            Token::T(t) => {
                if &t == tok {
                    tokens.forward();
                } else {
                    return Err(Error)
                }
            }
        }
    }
    Ok(())
}

pub fn contruct_talbe(cfg: &NoneLeftRecursionCFG) -> Table {
    let first = first(&cfg.0);
    let follow = follow(&cfg.0, &first);
    let predict = predict(cfg, &first, &follow);
    let cfg = &cfg.0;

    let mut table = HashMap::new();
    for nt in &cfg.non_terminals {
        let mut ps = HashMap::new();
        for t in &cfg.terminals {
            ps.insert(t.clone(), None);
        }

        for (i, p) in cfg.productions.get(nt).unwrap().into_iter().enumerate() {
            let pre = predict.get(&(nt.clone(), i)).unwrap();
            for t in pre  {
                ps.insert(t.clone(), Some(p));
            }
        }
        table.insert(nt.clone(), ps);
    }
    table
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
