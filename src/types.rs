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

impl List {
    fn iter(&self) -> ListIterator {
        ListIterator(self)
    }
}

struct ListIterator<'a>(&'a List);

impl<'a> Iterator for ListIterator<'a> {
    type Item = &'a Term;

    fn next(&mut self) -> Option<Self::Item> {
        use List::*;
        match self.0 {
            &Nil => None,
            &Cons(ref term, ref tail) => {
                self.0 = tail;
                Some(term)
            }
        }
    }
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

impl Predicate {
    fn unify(&self, other: &Self, knowledge: &Vec<Term>) -> UnifyResult {
        if self.name == other.name {
            self.arguments.unify(&other.arguments, knowledge)
        } else {
            Err("unifying different predicates".to_string())
        }
    }
}

impl List {
    fn unify(&self, other: &Self, knowledge: &Vec<Term>) -> UnifyResult {
        use List::*;
        match (self, other) {
            (&Nil, &Nil) => Ok(HashMap::new()),
            (&Cons(ref lx, ref lxs), &Cons(ref rx, ref rxs)) => {
                let mut head = lx.unify(rx, knowledge)?;
                let mut tail = lxs.unify(rxs, knowledge)?;
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
    pub fn derive(&self, knowledge: &Vec<Term>) -> UnifyResult {
        for fact in knowledge.iter() {
            let unifications = self.unify(fact, knowledge);
            if unifications.is_ok() {
                return unifications;
            }
        }
        Err("cannot derive it".to_string())
    }

    pub fn unify(&self, other: &Self, knowledge: &Vec<Term>) -> UnifyResult {
        use Term::*;
        match (self, other) {
            (&Var(ref lhs), ref rhs) => {
                let mut unifications = HashMap::new();
                unifications.insert(lhs.clone(), (*rhs).clone());
                Ok(unifications)
            }
            (ref lhs, &Var(ref rhs)) => {
                let mut unifications = HashMap::new();
                unifications.insert(rhs.clone(), (*lhs).clone());
                Ok(unifications)
            }
            (&Atom(ref lhs), &Atom(ref rhs)) if *lhs == *rhs => Ok(HashMap::new()),
            (&Pred(ref lhs), &Pred(ref rhs)) => lhs.unify(rhs, knowledge),
            (&Pred(ref pred), &Clause(ref clause)) |
            (&Clause(ref clause), &Pred(ref pred)) => {
                let mut unifications = pred.unify(&clause.result, knowledge)?;
                for condition in clause.conditions.iter() {
                    match condition.derive(knowledge) {
                        e @ Err(_) => return e,
                        Ok(mut u) => {
                            for (k, v) in u.drain() {
                                unifications.insert(k, v);
                            }
                        }
                    }
                }
                Ok(unifications)
            }
            (&List(ref lhs), &List(ref rhs)) => lhs.unify(rhs, knowledge),
            _ => Err("cannot unify".to_string()),
        }
    }
}
