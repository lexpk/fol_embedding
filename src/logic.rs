use std::collections::HashMap;

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
enum Term {
    Variable(usize),
    Function(usize, Vec<Term>),
}

type Substitution = HashMap<usize, Term>;
type Unifier = (Substitution, Substitution);

impl Term {
    fn apply(&self, t: &Substitution) -> Term {
        match self {
            Term::Variable(x) => match t.get(x) {
                Some(t) => t.clone(),
                None => Term::Variable(*x),
            },
            Term::Function(f, args) => Term::Function(
                *f,
                args.iter().map(|arg| arg.apply(t)).collect::<Vec<Term>>(),
            ),
        }
    }
}

fn mgu(term1: &Term, term2: &Term) -> Option<Unifier> {
    match (term1, term2) {
        (Term::Variable(x), Term::Variable(y)) => {
            if x == y {
                Some((HashMap::new(), HashMap::new()))
            } else {
                Some((
                    vec![(x.clone(), Term::Variable(y.clone()))]
                        .into_iter()
                        .collect(),
                    HashMap::new(),
                ))
            }
        }
        (Term::Variable(x), t) => Some((
            vec![(x.clone(), t.clone())].into_iter().collect(),
            HashMap::new(),
        )),
        (t, Term::Variable(x)) => Some((
            HashMap::new(),
            vec![(x.clone(), t.clone())].into_iter().collect(),
        )),
        (Term::Function(f1, args1), Term::Function(f2, args2)) => {
            if f1 != f2 || args1.len() != args2.len() {
                None
            } else {
                let mut sigma1 = HashMap::new();
                let mut sigma2 = HashMap::new();
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    (sigma1, sigma2) = match mgu(&arg1.apply(&sigma1), &arg2.apply(&sigma2)) {
                        Some((sigma1, sigma2)) => (sigma1, sigma2),
                        None => {
                            return None;
                        }
                    };
                }
                Some((sigma1, sigma2))
            }
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Equation {
    lhs: Term,
    rhs: Term,
}

impl Equation {
    fn apply(&self, t: &Substitution) -> Equation {
        Equation {
            lhs: self.lhs.apply(t),
            rhs: self.rhs.apply(t),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Sequent {
    lhs: Vec<Equation>,
    rhs: Vec<Equation>,
}

impl Sequent {
    fn apply(&self, t: &Substitution) -> Sequent {
        Sequent {
            lhs: self.lhs.iter().map(|eq| eq.apply(t)).collect(),
            rhs: self.rhs.iter().map(|eq| eq.apply(t)).collect(),
        }
    }
}

///Structure representing a particular problem instance.
#[derive(Clone)]
pub struct Environment {
    context: Context,
    sequents: Vec<Sequent>,
    variablecount: usize,
}

impl Environment {
    ///Create a new, empty Environment.
    pub fn new() -> Environment {
        Environment {
            context: Context::new(),
            sequents: Vec::new(),
            variablecount: 0,
        }
    }

    pub fn declare_sort(&mut self, sortname: &str) {
        if !self.context.sortid.contains_key(sortname) {
            self.context
                .sortid
                .insert(String::from(sortname), self.context.sortid.len());
            self.context
                .sortname
                .insert(self.context.sortid.len(), String::from(sortname));
        }
    }

    pub fn declare_function(
        &mut self,
        functionname: &str,
        argsorts: Vec<&str>,
        resultsort: &str,
    ) -> Result<()> {
        self.declare_sort(resultsort);
        for sort in argsorts.iter() {
            self.declare_sort(sort);
        }
        if !self.context.functionid.contains_key(functionname) {
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
                self.context.sortid.get(resultsort).unwrap().clone(),
            );
            self.context.argsorts.insert(
                self.context.argsorts.len(),
                argsorts
                    .into_iter()
                    .map(|s| self.context.sortid.get(s).unwrap().clone())
                    .collect(),
            );
            Ok(())
        } else {
            Err(Error::AlreadyDeclared(functionname.to_owned()))
        }
    }

    fn read_term(&mut self, s: &str, variables: &mut HashMap<String, usize>) -> Result<Term> {
        let t = s.trim();
        if !t.starts_with('(') {
            match self.context.functionid.get(t) {
                Some(id) => {
                    return Ok(Term::Function(id.clone(), Vec::new()));
                }
                None => match variables.get(t) {
                    Some(id) => {
                        return Ok(Term::Variable(id.clone()));
                    }
                    None => {
                        variables.insert(String::from(t), self.variablecount);
                        self.variablecount += 1;
                        return Ok(Term::Variable(self.variablecount - 1));
                    }
                },
            }
        }
        let mut acc = String::new();
        let mut tokens = Vec::new();
        let mut depth = 0;
        for c in s.chars() {
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
            args.push(self.read_term(s.as_str(), variables)?);
        }
        let term = Term::Function(
            match self.context.functionid.get(name.as_str()) {
                Some(i) => i.clone(),
                None => return Err(Error::Undeclared(String::from(name))),
            },
            args,
        );
        return Ok(term);
    }

    fn show_term(&self, t: &Term) -> String {
        match t {
            Term::Function(id, args) => {
                if args.len() == 0 {
                    self.context.functionname.get(&id).unwrap().clone()
                } else {
                    format!(
                        "({} {})",
                        self.context.functionname.get(&id).unwrap().clone(),
                        args.iter()
                            .map(|x| self.show_term(x))
                            .collect::<Vec<String>>()
                            .join(" ")
                    )
                }
            }
            Term::Variable(id) => format!("?{}", id),
        }
    }

    fn read_equation(&mut self, eq: &str) -> Result<Equation> {
        let mut variables = HashMap::new();
        let mut tokens = eq.split('=');
        let lhs: Term;
        let rhs: Term;
        match tokens.next() {
            Some(t) => lhs = self.read_term(t, &mut variables)?,
            None => return Err(Error::IllegalSequent(String::from(eq))),
        }
        match tokens.next() {
            Some(t) => rhs = self.read_term(t, &mut variables)?,
            None => return Err(Error::IllegalSequent(String::from(eq))),
        }
        match tokens.next() {
            Some(_) => return Err(Error::IllegalSequent(String::from(eq))),
            None => return Ok(Equation { lhs, rhs }),
        }
    }

    fn show_equation(&self, eq: &Equation) -> String {
        format!(
            "(= {} {})",
            self.show_term(&eq.lhs),
            self.show_term(&eq.rhs)
        )
    }

    fn read_equations(&mut self, eqs: &str) -> Result<Vec<Equation>> {
        let mut equations = Vec::new();
        for eq in eqs.split(',') {
            equations.push(self.read_equation(eq)?);
        }
        Ok(equations)
    }

    fn show_equations(&self, eqs: &Vec<Equation>) -> String {
        eqs.iter()
            .map(|x| self.show_equation(x))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn read_sequent(&mut self, s: &str) -> Result<Sequent> {
        let mut tokens = s.split("=>");
        let lhs: Vec<Equation>;
        let rhs: Vec<Equation>;
        match tokens.next() {
            Some(t) => lhs = self.read_equations(t)?,
            None => return Err(Error::IllegalSequent(String::from(s))),
        }
        match tokens.next() {
            Some(t) => rhs = self.read_equations(t)?,
            None => return Err(Error::IllegalSequent(String::from(s))),
        }
        match tokens.next() {
            Some(_) => return Err(Error::IllegalSequent(String::from(s))),
            None => return Ok(Sequent { lhs, rhs }),
        }
    }

    fn show_sequent(&self, s: &Sequent) -> String {
        format!(
            "{} => {}",
            self.show_equations(&s.lhs),
            self.show_equations(&s.rhs)
        )
    }

    pub fn declare_sequent(&mut self, s: &str) -> Result<()> {
        let seq = self.read_sequent(s)?;
        self.sequents.push(seq);
        Ok(())
    }
}

impl<'a> ToString for Environment {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for (name, id) in self.context.sortid.iter() {
            s.push_str(&format!("(declare-sort {} {}))", name, id));
        }
        for (name, id) in self.context.functionid.iter() {
            s.push_str(&format!(
                "(declare-fun {} ({}) {})",
                name,
                self.context
                    .argsorts
                    .get(id)
                    .unwrap()
                    .iter()
                    .map(|x| self.context.sortname.get(x).unwrap().clone())
                    .collect::<Vec<String>>()
                    .join(" "),
                self.context
                    .sortname
                    .get(self.context.resultsort.get(id).unwrap())
                    .unwrap()
            ));
        }
        for seq in self.sequents.iter() {
            s.push_str(&format!("(assert {})", self.show_sequent(seq)));
        }
        return s;
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Undeclared(String),
    AlreadyDeclared(String),
    DeclaredTwice(String),
    IllegalSequent(String),
}
