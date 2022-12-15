use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

///Stores logical context.
#[derive(Clone, PartialEq, Eq)]
struct Context {
    sortname: HashMap<usize, String>,
    sortid: HashMap<String, usize>,
    functionname: HashMap<usize, String>,
    functionid: HashMap<String, usize>,
    argsorts: HashMap<usize, Vec<usize>>,
    resultsort: HashMap<usize, usize>,
}

impl Context {
    fn new() -> Context {
        Context {
            sortname: HashMap::new(),
            sortid: HashMap::new(),
            functionname: HashMap::new(),
            functionid: HashMap::new(),
            argsorts: HashMap::new(),
            resultsort: HashMap::new(),
        }
    }
}

///Structure for representing terms.
#[derive(Clone, Hash, PartialEq, Eq)]
struct Term {
    id: usize,
    args: Vec<Arc<Term>>,
}

///Structure representing a particular problem instance.
#[derive(Clone)]
pub struct Environment {
    context: Context,
    terms: HashSet<Arc<Term>>,
    axioms: Vec<(Arc<Term>, Arc<Term>)>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            context: Context::new(),
            terms: HashSet::new(),
            axioms: Vec::new(),
        }
    }

    fn declare_sort(&mut self, sortname: String) {
        if !self.context.sortid.contains_key(&sortname) {
            self.context
                .sortid
                .insert(sortname.clone(), self.context.sortid.len());
            self.context
                .sortname
                .insert(self.context.sortid.len(), sortname.clone());
        }
    }

    pub fn declare_function(
        &mut self,
        functionname: String,
        argsorts: Vec<String>,
        resultsort: String,
    ) -> Result<()> {
        self.declare_sort(resultsort.to_owned());
        for sort in argsorts.iter() {
            self.declare_sort(String::from(sort));
        }
        if !self.context.functionid.contains_key(&functionname) {
            self.context.functionid.insert(
                functionname.clone().to_owned(),
                self.context.functionid.len(),
            );
            self.context.functionname.insert(
                self.context.functionname.len(),
                functionname.clone().to_owned(),
            );
            self.context.resultsort.insert(
                self.context.resultsort.len(),
                self.context.sortid.get(&resultsort).unwrap().clone(),
            );
            self.context.argsorts.insert(
                self.context.argsorts.len(),
                argsorts
                    .iter()
                    .map(|s| self.context.sortid.get(s).unwrap().clone())
                    .collect(),
            );
            Ok(())
        } else {
            Err(Error::AlreadyDeclared(functionname.to_owned()))
        }
    }

    fn read_term(&mut self, s: String) -> Result<Arc<Term>> {
        let t = s.trim();
        if !t.starts_with('(') {
            match self.context.functionid.get(t) {
                Some(id) => {
                    let result = Arc::new(Term {
                        id: id.clone(),
                        args: Vec::new(),
                    });
                    self.terms.insert(result.clone());
                    return Ok(Arc::clone(self.terms.get(&result).unwrap()));
                }
                None => todo!(),
            }
        }
        let mut acc = String::new();
        let mut tokens = Vec::new();
        let mut depth = 0;
        for c in s.as_str().chars() {
            match (depth, c) {
                (0, '(') => depth += 1,
                (1, ')') => {
                    depth -= 1;
                    tokens.push(acc);
                    acc = String::new();
                }
                (_, '(') => {
                    depth += 1;
                    acc.push(c);
                }
                (_, ')') => {
                    depth -= 1;
                    acc.push(c);
                }
                (1, ' ') => {
                    tokens.push(acc);
                    acc = String::new();
                }
                (_, _) => acc.push(c),
            }
        }
        let mut tokens = tokens.into_iter();
        let name = tokens.next().unwrap();
        let mut args = Vec::new();
        for s in tokens {
            args.push(self.read_term(s.clone())?);
        }
        let term = Arc::new(Term {
            id: match self.context.functionid.get(&name) {
                Some(i) => i.clone(),
                None => return Err(Error::Undeclared(name)),
            },
            args: args,
        });
        self.terms.insert(term.clone());
        return Ok(self.terms.get(&term).unwrap().clone());
    }

    fn show_term(&self, t: &Term) -> String {
        if t.args.is_empty() {
            self.context.functionname.get(&t.id).unwrap().clone()
        } else {
            format!(
                "({} {})",
                self.context.functionname.get(&t.id).unwrap(),
                t.args
                    .iter()
                    .map(|x| self.show_term(x.as_ref()))
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        }
    }

    pub fn declare_axiom(&mut self, l: String, r: String) -> Result<()> {
        let left = self.read_term(l.clone())?;
        let right = self.read_term(r.clone())?;
        if !self.axioms.contains(&(left.clone(), right.clone())) {
            self.axioms.push((left, right));
        }
        Ok(())
    }
}

impl ToString for Environment {
    fn to_string(&self) -> String {
        format!(
            "Axioms:\n{}",
            self.axioms
                .iter()
                .map(|(x, y)| format!("(= {} {})", self.show_term(&x), self.show_term(&y)))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Undeclared(String),
    AlreadyDeclared(String),
    DeclaredTwice(String),
}
