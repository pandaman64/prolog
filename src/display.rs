use std::fmt::{Display, Formatter, Result};

use types::*;

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name)
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}{}", self.name, self.id)
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
            &Atom(ref atom) => atom.fmt(f),
            &Var(ref var) => var.fmt(f),
            &Pred(ref pred) => pred.fmt(f),
            &Clause(ref clause) => clause.fmt(f),
            &List(ref list) => list.fmt(f),
        }
    }
}

