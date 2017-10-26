use std::collections::HashMap;
use std::cell::RefCell;

thread_local!{
    static ID: RefCell<usize> = RefCell::new(1);
}

fn next_id() -> usize {
    ID.with(|x| {
        let ret = *x.borrow();
        *x.borrow_mut() += 1;
        ret
    })
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Atom {
    pub name: String,
}

impl Atom {
    pub fn new(name: String) -> Self {
        Atom { name: name }
    }

    pub fn instantiate(&self, _: &mut HashMap<Variable, Variable>) -> Self {
        self.clone()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub id: usize,
}

impl Variable {
    pub fn new(name: String, id: usize) -> Self {
        Variable { name: name, id: id }
    }

    pub fn brand_new(name: String) -> Self {
        Variable {
            name: name,
            id: next_id(),
        }
    }

    pub fn instantiate(&self, dict: &mut HashMap<Variable, Variable>) -> Self {
        dict.entry(self.clone())
            .or_insert_with(|| Self::brand_new(self.name.clone()))
            .clone()
    }
}

// P(X, Y, Z, ...)
#[derive(Clone, Debug)]
pub struct Predicate {
    pub name: Atom,
    pub arguments: List,
}

impl Predicate {
    pub fn instantiate(&self, dict: &mut HashMap<Variable, Variable>) -> Self {
        Predicate {
            name: self.name.instantiate(dict),
            arguments: self.arguments.instantiate(dict),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Clause {
    pub result: Predicate,
    pub conditions: List,
}

impl Clause {
    pub fn instantiate(&self, dict: &mut HashMap<Variable, Variable>) -> Self {
        Clause {
            result: self.result.instantiate(dict),
            conditions: self.conditions.instantiate(dict),
        }
    }
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

    pub fn instantiate(&self, dict: &mut HashMap<Variable, Variable>) -> Self {
        use List::*;
        match self {
            &Nil => Nil,
            &Cons(ref term, ref tail) => {
                Cons(
                    Box::new(term.instantiate(dict)),
                    Box::new(tail.instantiate(dict)),
                )
            }
        }
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
type Knowledge = Vec<Clause>;

impl Predicate {
    fn unify(&self, other: &Self, knowledge: &Knowledge) -> UnifyResult {
        if self.name == other.name {
            self.arguments.unify(&other.arguments, knowledge)
        } else {
            Err("unifying different predicates".to_string())
        }
    }

    fn apply(&mut self, substitutions: &Assignment) {
        self.arguments.apply(substitutions)
    }
}

impl List {
    fn unify(&self, other: &Self, knowledge: &Knowledge) -> UnifyResult {
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

    fn apply(&mut self, substitutions: &Assignment) {
        if let List::Cons(ref mut head, ref mut tail) = *self {
            head.apply(substitutions);
            tail.apply(substitutions);
        }
    }
}

impl Term {
    pub fn instantiate(&self, dict: &mut HashMap<Variable, Variable>) -> Self {
        use Term::*;
        match self {
            &Atom(ref atom) => Atom(atom.instantiate(dict)),
            &Var(ref var) => Var(var.instantiate(dict)),
            &Pred(ref pred) => Pred(pred.instantiate(dict)),
            &Clause(ref clause) => Clause(clause.instantiate(dict)),
            &List(ref list) => List(list.instantiate(dict)),
        }
    }

    pub fn derive(&self, knowledge: &Knowledge) -> UnifyResult {
        for fact in knowledge.iter().map(
            |fact| fact.instantiate(&mut HashMap::new()),
        )
        {
            let unifications = self.unify(&Term::Clause(fact), knowledge);
            if unifications.is_ok() {
                return unifications;
            }
        }
        Err("cannot derive it".to_string())
    }

    pub fn unify(&self, other: &Self, knowledge: &Knowledge) -> UnifyResult {
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
                for mut condition in clause.conditions.iter().map(|c| {
                    c.instantiate(&mut HashMap::new())
                })
                {
                    condition.apply(&unifications);
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

    fn apply(&mut self, substitutions: &Assignment) {
        use Term::*;
        let replace = match *self {
            Var(ref v) => substitutions.get(v),
            Pred(ref mut pred) => {
                pred.apply(substitutions);
                None
            }
            List(ref mut list) => {
                list.apply(substitutions);
                None
            }
            _ => None,
        };

        if let Some(term) = replace {
            *self = term.clone();
        }
    }
}
