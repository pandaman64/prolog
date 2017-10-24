use std::io::{self, BufRead};

mod types;
mod parser;

use types::*;
use parser::*;

#[derive(Debug)]
struct Knowledge {
    atoms: Vec<Atom>,
    predicates: Vec<Predicate>,
}

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    for line in stdin.lines() {
        println!(
            "{:?}",
            line.map(|line| parse_line(&mut line.chars().peekable()))
        );
    }
}
