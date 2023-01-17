use pyo3::{exceptions::PyValueError, prelude::*, PyErr};

mod logic;
use logic::{Environment, Error};

#[pyclass(name = "Environment")]
struct PyEnvironment {
    env: Environment,
    #[pyo3(get)]
    sorts: Vec<String>,
    #[pyo3(get)]
    functions: Vec<(String, Vec<String>, String)>,
    #[pyo3(get)]
    sequents: Vec<(Vec<(PyObject, PyObject)>, Vec<(PyObject, PyObject)>)>,
}

#[pymethods]
impl PyEnvironment {
    #[new]
    fn new() -> Self {
        PyEnvironment {
            env: Environment::new(),
            sorts: Vec::new(),
            functions: Vec::new(),
            sequents: Vec::new(),
        }
    }

    fn __str__(&self, _py: Python) -> String {
        self.env.to_string()
    }

    fn declare_function(
        &mut self,
        name: &str,
        argument_type: Vec<&str>,
        result_type: &str,
    ) -> PyResult<()> {
        match self.env.declare_function(name, argument_type, result_type)? {
            (newsorts, (name, argument_type, result_type)) => {
                self.sorts.extend(newsorts);
                self.functions.push((name, argument_type, result_type));
            }
        }
        Ok(())
    }

    fn declare_sequent(&mut self,  _py: Python, s: &str) -> PyResult<()> {
        self.sequents.push(self.env.declare_sequent(_py, s)?);
        Ok(())
    }
}

#[pymodule]
fn fol_embedding(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEnvironment>()?;
    Ok(())
}

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        match err {
            Error::Undeclared(s) => PyValueError::new_err(format!("{} has not been declared!", s)),
            Error::AlreadyDeclared(s) => {
                PyValueError::new_err(format!("{} has already been declared!", s))
            }
            Error::DeclaredTwice(s) => {
                PyValueError::new_err(format!("{} has been declared twice!", s))
            }
            Error::IllegalSequent(s) => PyValueError::new_err(format!("Illegal Sequent {}", s)),
            Error::IllegalTerm(s) => PyValueError::new_err(format!("Illegal Term {}", s)),
        }
    }
}