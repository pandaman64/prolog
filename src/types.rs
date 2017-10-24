use std::collections::HashMap;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Atom {
    pub name: String,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Variable {
    pub name: String,
}

// P(X, Y, Z, ...)
#[derive(Clone, Debug)]
pub struct Predicate {
    pub name: Atom,
    pub arguments: List,
}

#[derive(Clone, Debug)]
pub struct Clause {
    pub result: Predicate,
    pub conditions: List,
}

#[derive(Clone, Debug)]
pub enum List {
    Nil,
    Cons(Box<Term>, Box<List>),
}

#[derive(Clone, Debug)]
pub enum Term {
    Atom(Atom),
    Var(Variable),
    Pred(Predicate),
    Clause(Clause),
    List(List),
}

type Assignment = HashMap<Variable, Term>;
type UnifyResult = Result<Assignment, String>;

impl List {
    fn unify(&self, other: &Self) -> UnifyResult {
        use List::*;
        match (self, other) {
            (&Nil, &Nil) => Ok(HashMap::new()),
            (&Cons(ref lx, ref lxs), &Cons(ref rx, ref rxs)) => {
                let mut head = lx.unify(rx)?;
                let mut tail = lxs.unify(rxs)?;
                for (k, v) in tail.drain() {
                    head.insert(k, v);
                }
                Ok(head)
            }
            _ => Err("cannot unify lists".to_string()),
        }
    }
}

impl Term {
    pub fn unify(&self, other: &Self) -> UnifyResult {
        use Term::*;
        match (self, other) {
            (&Var(ref lhs), ref rhs) => {
                let mut assignment = HashMap::new();
                assignment.insert(lhs.clone(), (*rhs).clone());
                Ok(assignment)
            }
            (ref lhs, &Var(ref rhs)) => {
                let mut assignment = HashMap::new();
                assignment.insert(rhs.clone(), (*lhs).clone());
                Ok(assignment)
            }
            (&Atom(ref lhs), &Atom(ref rhs)) if *lhs == *rhs => Ok(HashMap::new()),
            (&Pred(ref lhs), &Pred(ref rhs)) if lhs.name == rhs.name => {
                lhs.arguments.unify(&rhs.arguments)
            }
            (&Clause(_), _) | (_, &Clause(_)) => Err("cannot deal with clauses here".to_string()),
            (&List(ref lhs), &List(ref rhs)) => lhs.unify(rhs),
            _ => Err("cannot unify".to_string()),
        }
    }
}
