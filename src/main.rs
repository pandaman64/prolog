use std::io::{self, BufRead};
use std::collections::HashSet;

mod types;
mod parser;
mod display;

use types::*;
use parser::*;

fn main() {
    set_debug(false);
    let mut knowledge = vec![];

    let stdin = io::stdin();
    let stdin = stdin.lock();
    for line in stdin.lines() {
        if let Ok(line) = line {
            if let Ok(result) = parse_line(&mut line.chars().peekable()) {
                match result {
                    Command::Assertion(assertion) => {
                        println!("accepted: {}", assertion);
                        knowledge.push(assertion)
                    }
                    Command::Question(question) => {
                        println!("asked: {}", question);
                        let mut subst = HashSet::new();
                        match question.derive(&knowledge, &mut subst) {
                            Err(error) => println!("false: {}", error),
                            Ok(()) => {
                                println!("true");
                                for v in subst.iter() {
                                    println!("  {}", v);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
