use std::fmt::{Display, Formatter, Result};

use types::*;

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name)
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}{}", self.name, self.id)?;
        match *self.assignment.borrow() {
            None => write!(f, "[None]"),
            Some(ref term) => {
                write!(f, "[")?;
                term.fmt(f)?;
                write!(f, "]")
            },
        }
    }
}

impl Display for Predicate {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}({})", self.name, self.arguments)
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} :- {}", self.result, self.conditions)
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use List::*;
        match self {
            &Nil => Ok(()),
            &Cons(ref head, ref tail) => {
                write!(f, "{}", head)?;
                match **tail {
                    Nil => Ok(()),
                    _ => {
                        write!(f, ", ")?;
                        tail.fmt(f)
                    }
                }
            }
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Term::*;
        match self {
            &Var(ref var) => var.fmt(f),
            &Pred(ref pred) => pred.fmt(f),
            &List(ref list) => list.fmt(f),
        }
    }
}
