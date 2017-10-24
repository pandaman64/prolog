use std::fmt::Debug;
use std::iter::Peekable;
use std::io::{self, BufRead};

trait Term: Debug {}

#[derive(PartialEq, Eq, Debug)]
struct Atom {
    name: String,
}

impl Term for Atom {}

#[derive(PartialEq, Eq, Debug)]
struct Variable {
    name: String,
}

impl Term for Variable {}

// P(X, Y, Z, ...)
#[derive(Debug)]
struct Predicate {
    name: Atom,
    arguments: List,
}

impl Term for Predicate {}

#[derive(Debug)]
struct Clause {
    result: Box<Term>,
    conditions: List,
}

impl Term for Clause {}

#[derive(Debug)]
enum List {
    Nil,
    Cons(Box<Term>, Box<List>),
}

impl Term for List {}

#[derive(Debug)]
struct Knowledge {
    atoms: Vec<Atom>,
    predicates: Vec<Predicate>,
}

type ParseError = ();
type ParseResult = Result<Box<Term>, ParseError>;

/* Parser
 * query := assertion | question
 * question := ?- term '.'
 * assertion := clause '.'
 * clause := term [':-' term (',' term)* ]
 * term := atom | variable | list
 * atom := <lowercase> <id_char>*
 * variable := <uppercase> <id_char>*
 */
fn consume_spaces<I: Iterator<Item = char>>(iter: &mut Peekable<I>) {
    loop {
        // kill the reference
        if let Some(c) = iter.peek().map(|x| *x) {
            if c.is_whitespace() {
                iter.next();
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

fn identifier<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> String {
    consume_spaces(iter);
    let mut s = String::new();
    loop {
        match iter.peek() {
            Some(&c) if c.is_alphabetic() => {
                iter.next();
                s.push(c)
            }
            _ => return s,
        }
    }
}

fn atom<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Atom, ParseError> {
    Ok(Atom { name: identifier(iter) })
}

fn variable<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Variable, ParseError> {
    Ok(Variable { name: identifier(iter) })
}

fn arguments_impl<I: Iterator<Item = char>>(
    iter: &mut Peekable<I>,
    end: char,
) -> Result<List, ParseError> {
    consume_spaces(iter);
    match iter.peek() {
        None => Err(()),
        Some(&c) if c == end => {
            iter.next();
            Ok(List::Nil)
        }
        Some(&',') => {
            iter.next();
            let arg = term(iter)?;
            arguments_impl(iter, end).map(|args| List::Cons(arg, Box::new(args)))
        }
        _ => Err(()),
    }
}

// ')'も読む
fn arguments<I: Iterator<Item = char>>(
    iter: &mut Peekable<I>,
    end: char,
) -> Result<List, ParseError> {
    consume_spaces(iter);
    match iter.peek() {
        None => Err(()),
        _ => {
            let first = term(iter)?;
            arguments_impl(iter, end).map(|args| List::Cons(first, Box::new(args)))
        }
    }
}

fn predicate<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> ParseResult {
    consume_spaces(iter);
    let p = atom(iter)?;

    consume_spaces(iter);
    match iter.peek() {
        Some(&'(') => {
            iter.next();
            let args = arguments(iter, ')')?;

            Ok(Box::new(Predicate {
                name: p,
                arguments: args,
            }))
        }
        _ => Ok(Box::new(p)),
    }
}

fn term<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> ParseResult {
    consume_spaces(iter);

    // kill the reference
    match iter.peek().map(|x| *x) {
        None => Err(()),
        Some(c) => {
            if c.is_lowercase() {
                predicate(iter)
            } else if c.is_uppercase() {
                Ok(Box::new(variable(iter)?))
            } else {
                Err(())
            }
        }
    }
}

fn clause<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> ParseResult {
    let result = term(iter)?;
    consume_spaces(iter);
    match iter.peek() {
        Some(&'.') => Ok(result),
        Some(&':') => {
            iter.next();
            if let Some('-') = iter.next() {
                let conditions = arguments(iter, '.')?;
                Ok(Box::new(Clause {
                    result: result,
                    conditions: conditions,
                }))
            } else {
                Err(())
            }
        }
        _ => Err(()),
    }
}

#[derive(Debug)]
enum Command {
    Assertion(Box<Term>),
    Question(Box<Term>),
}

fn parse_line<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Command, ParseError> {
    consume_spaces(iter);

    match iter.peek() {
        Some(&'?') => {
            iter.next();
            if let Some('-') = iter.next() {
                let q = term(iter)?;
                consume_spaces(iter);
                if let Some(&'.') = iter.peek() {
                    iter.next();
                    return Ok(Command::Question(q));
                }
            }
            Err(())
        }
        Some(_) => clause(iter).map(Command::Assertion),
        _ => Err(()),
    }
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
