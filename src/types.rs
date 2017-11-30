use std::collections::{HashSet, HashMap};
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp::{PartialEq, Eq};
use std::hash::{Hash, Hasher};

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

thread_local!{
    static DEBUG: RefCell<bool> = RefCell::new(false);
    static LEVEL: RefCell<usize> = RefCell::new(0);
}

pub fn set_debug(b: bool) {
    DEBUG.with(|d| *d.borrow_mut() = b);
}

fn get_debug() -> bool {
    DEBUG.with(|d| *d.borrow())
}

fn shift() {
    LEVEL.with(|l| *l.borrow_mut() += 1);
}

fn unshift() {
    LEVEL.with(|l| *l.borrow_mut() -= 1);
}

fn get_level() -> usize {
    LEVEL.with(|l| *l.borrow())
}

macro_rules! debug_println {
    ($( $arg:expr ),*) => { 
        if get_debug() { 
            for _ in 0..get_level() {
                print!("  ");
            }
            println!($( $arg ),*);
        }
    }
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

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub id: usize,
    pub assignment: Rc<RefCell<Option<Term>>>
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name && self.id == other.id;
    }
}
impl Eq for Variable {}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.id.hash(state);
    }
}

impl Variable {
    pub fn new(name: String, id: usize) -> Self {
        Variable { name: name, id: id, assignment: Rc::new(RefCell::new(None)) }
    }

    pub fn brand_new(name: String) -> Self {
        Variable {
            name: name,
            id: next_id(),
            assignment: Rc::new(RefCell::new(None)),
        }
    }

    pub fn instantiate(&self, dict: &mut HashMap<Variable, Variable>) -> Self {
        dict.entry(self.clone())
            .or_insert_with(|| Self::brand_new(self.name.clone()))
            .clone()
    }

    pub fn assign(&mut self, mut term: Term, subst: &mut Variables) -> Result<(), DeriveError> {
        let assignment =
            match &mut *self.assignment.borrow_mut() {
                &mut None => Some(term),
                &mut Some(ref mut other) => return other.doit(&mut term, subst)
            };
        *self.assignment.borrow_mut() = assignment;
        Ok(())
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

type Variables = HashSet<Variable>;
type DeriveError = String;

impl Predicate {
    pub fn derive(&self, knowledge: &[Clause], subst: &mut Variables) -> Result<(), DeriveError> {
        debug_println!("derive {}", self);
        shift();
        for mut fact in knowledge.iter().map(|c| c.instantiate(&mut HashMap::new())) {
            if let Ok(_) = self.clone().doit(&mut fact.result, subst) {
                if let Ok(()) = fact.conditions.derive(knowledge, subst) {
                    unshift();
                    return Ok(());
                }
            }
        }
        unshift();
        Err("No matching facts".into())
    }

    fn doit(&mut self, other: &mut Self, subst: &mut Variables) -> Result<(), DeriveError> {
        debug_println!("PREDICATE: self = {}, other = {}", self, other); 

        use List::*;
        if self.name != other.name {
            return Err("Predicate name mismatch".into())
        }

        let mut self_args = &mut self.arguments;
        let mut other_args = &mut other.arguments;

        loop {
            match (self_args, other_args) {
                (&mut Nil, &mut Nil) => return Ok(()),
                (&mut Cons(ref mut self_head, ref mut self_tail), &mut Cons(ref mut other_head, ref mut other_tail)) => if let Ok(()) = self_head.doit(other_head, subst) {
                    self_args = self_tail;
                    other_args = other_tail;
                } else {
                    return Err("Failed to unify an element in a list".into())
                },
                _ => return Err("List size doesn't match".into()),
            }
        }
    }
}

impl Term {
    pub fn derive(&self, knowledge: &[Clause], subst: &mut Variables) -> Result<(), DeriveError> {
        use Term::*;
        match self {
            &Var(ref v) => {
                // anything can be derived
                subst.insert(v.clone());
                Ok(())
            },
            &Pred(ref pred) => pred.derive(knowledge, subst),
            &List(ref list) => list.derive(knowledge, subst),
        }
    }

    pub fn instantiate(&self, dict: &mut HashMap<Variable, Variable>) -> Self {
        use Term::*;
        match self {
            &Var(ref v) => Var(v.instantiate(dict)),
            &Pred(ref p) => Pred(p.instantiate(dict)),
            &List(ref l) => List(l.instantiate(dict)),
        }
    }

    pub fn doit(&mut self, other: &mut Self, subst: &mut Variables) -> Result<(), DeriveError> {
        debug_println!("TERM: self = {}, other = {}", self, other);
        use Term::*;

        match (self, other) {
            (&mut Var(ref mut v), ref mut o) => {
                // TODO: need occurs check
                subst.insert(v.clone());
                v.assign(o.clone(), subst)
            },
            (ref mut this, &mut Var(ref mut v)) => {
                // TODO: need occurs check
                subst.insert(v.clone());
                v.assign(this.clone(), subst)
            },
            (&mut Pred(ref mut this), &mut Pred(ref mut o)) => this.doit(o, subst),
            (&mut List(ref mut this), &mut List(ref mut o)) => this.doit(o, subst),
            _ => Err("Term type doesn't match".into()),
        }
    }
}

impl List {
    pub fn doit(&mut self, other: &mut Self, subst: &mut Variables) -> Result<(), DeriveError> {
        debug_println!("LIST: self = {}, other = {}", self, other);
        use List::*;
        match (self, other) {
            (&mut Nil, &mut Nil) => Ok(()),
            (&mut Cons(ref mut self_head, ref mut self_tail), &mut Cons(ref mut other_head, ref mut other_tail)) =>  {
                self_head.doit(other_head, subst)?;
                self_tail.doit(other_tail, subst)?;
                Ok(())
            },
            _ => Err("List size doesn't match".into())
        }
    }

    // derivation of a list means derivation of the conjunction of each element
    pub fn derive(&self, knowledge: &[Clause], subst: &mut Variables) -> Result<(), DeriveError> {
        use List::*;
        match self {
            &Nil => Ok(()),
            &Cons(ref head, ref tail) => {
                head.derive(knowledge, subst)?;
                tail.derive(knowledge, subst)
            }
        }
    }
}

