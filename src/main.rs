use std::io::{self, BufRead};

mod types;
mod parser;
mod display;

use types::*;
use parser::*;

fn main() {
    set_debug(true);
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
                        match question.derive(&knowledge) {
                            Err(error) => println!("false: {}", error),
                            Ok(subst) => {
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
