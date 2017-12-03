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
        // assigned varaible should not be instantiated
        if let Some(_) = *self.assignment.borrow() {
            self.clone()
        } else {
            dict.entry(self.clone())
                .or_insert_with(|| Self::brand_new(self.name.clone()))
                .clone()
        }
    }

    pub fn assign(&mut self, mut term: Term) -> Result<(), DeriveError> {
        let assignment =
            match &mut *self.assignment.borrow_mut() {
                &mut None => Some(term),
                &mut Some(ref mut other) => {
                    other.unify(&mut term)?;
                    None
                }
            };
        if let Some(term) = assignment {
            *self.assignment.borrow_mut() = Some(term);
        }
        self.compress();
        debug_println!("assign {} <= {}", self, self.assignment.borrow().as_ref().unwrap());
        Ok(())
    }

    fn compress(&self) {
        use Term::*;
        let assignment = 
            if let &mut Some(ref mut term) = &mut *self.assignment.borrow_mut() {
                match *term {
                    Var(ref mut v) => {
                        v.compress();
                        if let &mut Some(ref t) = &mut *v.assignment.borrow_mut() {
                            t.clone()
                        } else {
                            return;
                        }
                    },
                    _ => return,
                }
            } else {
                return;
            };
        *self.assignment.borrow_mut() = Some(assignment);
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
    fn variables(&self) -> Variables {
        self.arguments.variables()
    }

    pub fn derive(&self, knowledge: &[Clause]) -> Result<Variables, DeriveError> {
        debug_println!("derive {}", self);
        shift();
        for mut fact in knowledge.iter().map(|c| c.instantiate(&mut HashMap::new())) {
            let mut target = self.clone();
            // this changes the shared state of variables within self
            // so we need to some reset
            if let Ok(()) = target.unify(&mut fact.result) {
                // discard the variables in conditions 
                // because only the top level variables will be returned
                if let Ok(_) = fact.conditions.derive(knowledge) {
                    unshift();
                    let vs = target.variables();
                    for v in vs.iter() {
                        v.compress();
                    }
                    return Ok(vs);
                }
            }
        }
        unshift();
        Err("No matching facts".into())
    }

    fn unify(&mut self, other: &mut Self) -> Result<(), DeriveError> {
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
                (&mut Cons(ref mut self_head, ref mut self_tail), &mut Cons(ref mut other_head, ref mut other_tail)) => if let Ok(()) = self_head.unify(other_head) {
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
    pub fn variables(&self) -> Variables {
        use Term::*;

        match *self {
            Var(ref v) => {
                let mut ret = HashSet::new();
                ret.insert(v.clone());
                ret
            },
            Pred(ref p) => p.variables(),
            List(ref l) => l.variables(),
        }
    }

    pub fn derive(&self, knowledge: &[Clause]) -> Result<Variables, DeriveError> {
        use Term::*;
        match self {
            &Var(ref v) => {
                // anything can be derived
                let mut ret = HashSet::new();
                ret.insert(v.clone());
                Ok(ret)
            },
            &Pred(ref pred) => pred.derive(knowledge),
            &List(ref list) => list.derive(knowledge),
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

    pub fn unify(&mut self, other: &mut Self) -> Result<(), DeriveError> {
        debug_println!("TERM: self = {}, other = {}", self, other);
        use Term::*;

        match (self, other) {
            (&mut Var(ref mut v), ref mut o) => {
                // TODO: need occurs check
                v.assign(o.clone())
            },
            (ref mut this, &mut Var(ref mut v)) => {
                // TODO: need occurs check
                v.assign(this.clone())
            },
            (&mut Pred(ref mut this), &mut Pred(ref mut o)) => this.unify(o),
            (&mut List(ref mut this), &mut List(ref mut o)) => this.unify(o),
            _ => Err("Term type doesn't match".into()),
        }
    }
}

impl List {
    fn variables(&self) -> Variables {
        use List::*;
        match *self {
            Nil => HashSet::new(),
            Cons(ref head, ref tail) => {
                let mut ret = head.variables();
                for v in tail.variables().into_iter() {
                    ret.insert(v);
                }
                ret
            }
        }
    }

    pub fn unify(&mut self, other: &mut Self) -> Result<(), DeriveError> {
        debug_println!("LIST: self = {}, other = {}", self, other);
        use List::*;
        match (self, other) {
            (&mut Nil, &mut Nil) => Ok(()),
            (&mut Cons(ref mut self_head, ref mut self_tail), &mut Cons(ref mut other_head, ref mut other_tail)) =>  {
                self_head.unify(other_head)?;
                self_tail.unify(other_tail)?;
                Ok(())
            },
            _ => Err("List size doesn't match".into())
        }
    }

    // derivation of a list means derivation of the conjunction of each element
    pub fn derive(&self, knowledge: &[Clause]) -> Result<Variables, DeriveError> {
        use List::*;
        match self {
            &Nil => Ok(HashSet::new()),
            &Cons(ref head, ref tail) => {
                let mut ret = head.derive(knowledge)?;
                for v in tail.derive(knowledge)?.into_iter() {
                    ret.insert(v);
                }
                Ok(ret)
            }
        }
    }
}

