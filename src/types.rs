use std::fmt::Debug;

pub trait Term: Debug {}

#[derive(PartialEq, Eq, Debug)]
pub struct Atom {
    pub name: String,
}

impl Term for Atom {}

#[derive(PartialEq, Eq, Debug)]
pub struct Variable {
    pub name: String,
}

impl Term for Variable {}

// P(X, Y, Z, ...)
#[derive(Debug)]
pub struct Predicate {
    pub name: Atom,
    pub arguments: List,
}

impl Term for Predicate {}

#[derive(Debug)]
pub struct Clause {
    pub result: Box<Term>,
    pub conditions: List,
}

impl Term for Clause {}

#[derive(Debug)]
pub enum List {
    Nil,
    Cons(Box<Term>, Box<List>),
}

impl Term for List {}
