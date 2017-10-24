use std::iter::Peekable;

use types::*;

type ParseError = ();
type ParseResult = Result<Term, ParseError>;

#[derive(Debug)]
pub enum Command {
    Assertion(Term),
    Question(Term),
}

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

fn identifier_character(c: char) -> bool {
    c.is_alphabetic() || c.is_numeric() || c == '_' || c == '-'
}

fn identifier<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> String {
    consume_spaces(iter);
    let mut s = String::new();
    loop {
        match iter.peek() {
            Some(&c) if identifier_character(c) => {
                iter.next();
                s.push(c)
            }
            _ => return s,
        }
    }
}

fn atom<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Atom, ParseError> {
    Ok(Atom::new(identifier(iter)))
}

fn variable<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Variable, ParseError> {
    Ok(Variable::new(identifier(iter)))
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
            arguments_impl(iter, end).map(|args| List::Cons(Box::new(arg), Box::new(args)))
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
            arguments_impl(iter, end).map(|args| List::Cons(Box::new(first), Box::new(args)))
        }
    }
}

fn predicate<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Predicate, ParseError> {
    consume_spaces(iter);
    let p = atom(iter)?;

    consume_spaces(iter);
    match iter.peek() {
        Some(&'(') => {
            iter.next();
            let args = arguments(iter, ')')?;

            Ok(Predicate {
                name: p,
                arguments: args,
            })
        }
        _ => Ok(Predicate {
            name: p,
            arguments: List::Nil,
        }),
    }
}

fn term<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> ParseResult {
    consume_spaces(iter);

    // kill the reference
    match iter.peek().map(|x| *x) {
        None => Err(()),
        Some(c) => {
            if c.is_lowercase() {
                predicate(iter).map(Term::Pred)
            } else if c.is_uppercase() {
                Ok(Term::Var(variable(iter)?))
            } else {
                Err(())
            }
        }
    }
}

fn clause<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> ParseResult {
    let result = predicate(iter)?;
    consume_spaces(iter);
    match iter.peek() {
        Some(&'.') => Ok(Term::Pred(result)),
        Some(&':') => {
            iter.next();
            if let Some('-') = iter.next() {
                let conditions = arguments(iter, '.')?;
                Ok(Term::Clause(Clause {
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

pub fn parse_line<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Result<Command, ParseError> {
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
