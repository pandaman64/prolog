use std::io::{self, BufRead};

mod types;
mod parser;

use types::*;
use parser::*;

fn main() {
    let mut knowledge = vec![];

    let stdin = io::stdin();
    let stdin = stdin.lock();
    for line in stdin.lines() {
        if let Ok(line) = line {
            if let Ok(result) = parse_line(&mut line.chars().peekable()) {
                match result {
                    Command::Assertion(assertion) => {
                        println!("accepted: {:?}", assertion);
                        knowledge.push(assertion)
                    }
                    Command::Question(question) => {
                        println!("asked: {:?}", question);
                        println!("derivation result: {:?}", question.derive(&knowledge));
                    }
                }
            }
        }
    }
}
