use pyo3::{exceptions::PyValueError, prelude::*, PyErr};

mod logic;
use logic::{Environment, Error};

#[pyclass(name = "Environment")]
struct PyEnvironment {
    env: Environment,
}

#[pymethods]
impl PyEnvironment {
    #[new]
    fn new() -> Self {
        PyEnvironment {
            env: Environment::new(),
        }
    }

    fn __str__(&self, _py: Python) -> String {
        self.env.to_string()
    }

    fn declare_function(
        &mut self,
        name: String,
        argument_type: Vec<String>,
        result_type: String,
    ) -> PyResult<()> {
        self.env
            .declare_function(name, argument_type, result_type)?;
        Ok(())
    }

    fn declare_axiom(&mut self, l: String, r: String) -> PyResult<()> {
        self.env.declare_axiom(l, r)?;
        Ok(())
    }
}

#[pymodule]
fn learned_equational_rewriting(_py: Python, m: &PyModule) -> PyResult<()> {
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
        }
    }
}
