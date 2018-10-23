use std::collections::{HashSet, BTreeSet};

use lazy_static::lazy_static;

use super::*;

lazy_static! {
    static ref GRAMMER: Vec<(&'static str, Vec<&'static str>)> = vec! {
        ("Goal", vec!["Expr"]),

        ("Expr", vec!["Expr", "+", "Term"]),
        ("Expr", vec!["Expr", "-", "Term"]),
        ("Expr", vec!["Term"]),

        ("Term", vec!["Term", "x", "Factor"]),
        ("Term", vec!["Term", "/", "Factor"]),
        ("Term", vec!["Factor"]),

        ("Factor", vec!["(", "Expr", ")"]),
        ("Factor", vec!["num"]),
        ("Factor", vec!["name"])
    };

    static ref RIGHT_RECURSIVE_GRAMMER: Vec<(&'static str, Vec<&'static str>)> = vec! {
        ("Goal", vec!["Expr"]),

        ("Expr", vec!["Term", "Expr@"]),
        ("Expr@", vec!["+", "Term", "Expr@"]),
        ("Expr@", vec!["-", "Term", "Expr@"]),
        ("Expr@", vec!["empty@@"]),

        ("Term", vec!["Factor", "Term@"]),
        ("Term@", vec!["x", "Factor", "Term@"]),
        ("Term@", vec!["/", "Factor", "Term@"]),
        ("Term@", vec!["empty@@"]),

        ("Factor", vec!["(", "Expr", ")"]),
        ("Factor", vec!["num"]),
        ("Factor", vec!["name"])
    };
}

fn gen_cfg(grammer: &Vec<(&'static str, Vec<&'static str>)>) -> CFG {
    let mut nt_str = HashSet::new();
    let mut nts = Vec::new();
    let mut ts = HashSet::new();
    let mut ps = HashMap::new();
    for &(nt, _)  in grammer {
        if nt_str.insert(nt) {
            nts.push(NoneTerminal::new(nt))
        }
    }
    for &(nt, ref production) in grammer {
        let entry = ps.entry(NoneTerminal::new(nt)).or_insert_with(Vec::new);
        let tokens = production.clone().into_iter().map(|s|{
            if nt_str.get(s).is_some() {
                Token::NT(NoneTerminal::new(s))
            } else {
                let t = Terminal::new(s);
                ts.insert(t.clone());
                Token::T(t.clone())
            }
        }).collect();
        entry.push(Production::new(NoneTerminal::new(nt), tokens));
    }
    CFG {
        terminals: ts.into_iter().collect(),
        non_terminals: nts,
        productions: ps,
        start: NoneTerminal::start(),
    }
}

fn assert_cfg_eq(l: &CFG, r: &CFG) {
    let lt: BTreeSet<_>= l.terminals.iter().collect();
    let rt: BTreeSet<_>= r.terminals.iter().collect();
    assert_eq!(lt, rt);

    let lnt: BTreeSet<_> = l.non_terminals.iter().collect();
    let rnt: BTreeSet<_> = r.non_terminals.iter().collect();
    assert_eq!(lnt, rnt);

    let lp: BTreeSet<_> = l.productions.values().flatten().collect();
    let rp: BTreeSet<_> = r.productions.values().flatten().collect();
    assert_eq!(lp, rp)
}

#[test]
fn test_eleminate_left_recursion() {
    let input = gen_cfg(&GRAMMER).into_non_left_recursion();
    let expected = gen_cfg(&RIGHT_RECURSIVE_GRAMMER);
    assert_cfg_eq(&input.0, &expected)
}
