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
    Var(Variable),
    Pred(Predicate),
    List(List),
}

pub struct Assignment(pub HashMap<Variable, Term>);

impl Assignment {
    fn new() -> Self {
        Assignment(HashMap::new())
    }
}

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
            (&Nil, &Nil) => Ok(Assignment::new()),
            (&Cons(ref lx, ref lxs), &Cons(ref rx, ref rxs)) => {
                let mut head = lx.unify(rx, knowledge)?;
                let tail = lxs.unify(rxs, knowledge)?;
                apply(&mut head, tail, knowledge)?;
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

fn apply(s1: &mut Assignment, mut s2: Assignment, knowledge: &Knowledge) -> Result<(), String> {
    /*
    println!("apply");
    for (k, v) in s2.0.iter() {
        println!("\t{} => {}", k, v);
    }
    println!("to");
    for (k, v) in s1.0.iter() {
        println!("\t{} => {}", k, v);
    }
    */
    for (_, v) in s1.0.iter_mut() {
        v.apply(&s2);
    }
    for (_, v) in s2.0.iter_mut() {
        v.apply(s1);
    }
    for (k, v2) in s2.0.drain() {
        let s = if let Some(v1) = s1.0.get(&k) {
            Some(v1.clone().unify(&v2, knowledge)?)
        } else {
            None
        };
        if let Some(s) = s {
            apply(s1, s, knowledge)?;
        } else {
            s1.0.insert(k, v2);
        }
    }
    /*
    println!("result");
    for (k, v) in s1.0.iter() {
        println!("\t{} => {}", k, v);
    }
    */
    Ok(())
}

impl Term {
    pub fn instantiate(&self, dict: &mut HashMap<Variable, Variable>) -> Self {
        use Term::*;
        match self {
            &Var(ref var) => Var(var.instantiate(dict)),
            &Pred(ref pred) => Pred(pred.instantiate(dict)),
            &List(ref list) => List(list.instantiate(dict)),
        }
    }

    pub fn derive(&self, knowledge: &Knowledge) -> UnifyResult {
        /*
        println!("deriving {} with:", self);
        for fact in knowledge.iter() {
            println!("\t{}", fact);
        }
        */

        for fact in knowledge.iter().map(
            |fact| fact.instantiate(&mut HashMap::new()),
        )
        {
            if let Ok(mut substitutions) = self.unify(&Term::Pred(fact.result.clone()), knowledge) {
                //println!("unifying {} and {} success", self, fact.result);
                let mut ok = true;
                for mut condition in fact.conditions.iter().map(Clone::clone) {
                    condition.apply(&substitutions);
                    match condition.derive(knowledge) {
                        Err(_) => {
                            ok = false;
                            break;
                        }
                        Ok(u) => apply(&mut substitutions, u, knowledge)?,
                    }
                }

                if ok {
                    return Ok(substitutions);
                }
            }
            //println!("unifying {} and {} failed", self, fact.result);
        }
        Err("cannot derive it".to_string())
    }

    pub fn unify(&self, other: &Self, knowledge: &Knowledge) -> UnifyResult {
        //println!("unifying {} and {}", self, other);
        use Term::*;
        match (self, other) {
            (&Var(ref v), other) => {
                let mut unifications = Assignment::new();
                //println!("add substution {} => {}", v, other);
                unifications.0.insert(v.clone(), other.clone());
                Ok(unifications)
            }
            (other, &Var(ref v)) => {
                let mut unifications = Assignment::new();
                //println!("add substution {} => {}", v, other);
                unifications.0.insert(v.clone(), other.clone());
                Ok(unifications)
            }
            (&Pred(ref lhs), &Pred(ref rhs)) => lhs.unify(rhs, knowledge),
            (&List(ref lhs), &List(ref rhs)) => lhs.unify(rhs, knowledge),
            _ => Err("cannot unify".to_string()),
        }
    }

    fn apply(&mut self, substitutions: &Assignment) {
        use Term::*;
        let replace = match *self {
            Var(ref v) => substitutions.0.get(v),
            Pred(ref mut pred) => {
                pred.apply(substitutions);
                None
            }
            List(ref mut list) => {
                list.apply(substitutions);
                None
            }
        };

        if let Some(term) = replace {
            //println!("replace {} with {}", self, term);
            *self = term.clone();
        }
    }
}
