use std::collections::{HashSet, BTreeSet, HashMap};

use lazy_static::lazy_static;

use super::*;

lazy_static! {
    static ref GRAMMER: Vec<(&'static str, Vec<&'static str>)> = vec! {
        ("Goal", vec!["Expr"]),

        ("Expr", vec!["Expr", "+", "Term"]),
        ("Expr", vec!["Expr", "-", "Term"]),
        ("Expr", vec!["Term"]),

        ("Term", vec!["Term", "*", "Factor"]),
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
        ("Term@", vec!["*", "Factor", "Term@"]),
        ("Term@", vec!["/", "Factor", "Term@"]),
        ("Term@", vec!["empty@@"]),

        ("Factor", vec!["(", "Expr", ")"]),
        ("Factor", vec!["num"]),
        ("Factor", vec!["name"])
    };

    static ref FIRST: Vec<(&'static str, Vec<&'static str>)> = vec! {
        ("Goal", vec!["(", "name", "num"]),
        ("Expr", vec!["(", "name", "num"]),
        ("Expr@", vec!["+", "-", "empty@@"]),
        ("Term", vec!["(", "name", "num"]),
        ("Term@", vec!["*", "/", "empty@@"]),
        ("Factor", vec!["(", "name", "num"]),
    };

    static ref FOLLOW: Vec<(&'static str, Vec<&'static str>)> = vec! {
        ("Goal", vec!["eof@@"]),
        ("Expr", vec!["eof@@", ")"]),
        ("Expr@", vec!["eof@@", ")"]),
        ("Term", vec!["eof@@", "+", "-", ")"]),
        ("Term@", vec!["eof@@", "+", "-", ")"]),
        ("Factor", vec!["eof@@", "+", "-", "*", "/", ")"]),
    };
}
