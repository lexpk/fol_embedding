mod logic;
use logic::{Environment, Error};

use tch::{Tensor};
use tch::nn::{Module};

struct Embedding {
    env: Environment,
    sort_dimension: HashMap<usize, usize>,
    function_hidden_layers: HashMap<usize, Vec<usize>>,
    function_embedding: HashMap<usize, Module>,
}

fn term_embedding(&self, t: &Term) -> Module {
    match t {
        Term::Variable(v) => {
            let sort = self.env.get_sort(v).unwrap();
            let dimension = self.sort_dimension.get(&sort).unwrap();
            let embedding = self.function_embedding.get(&sort).unwrap();
            embedding.forward(&Tensor::zeros(&[1, *dimension], (tch::Kind::Float, tch::Device::Cpu)))
        }
        Term::Function(f, args) => {
            let sort = self.env.get_sort(f).unwrap();
            let dimension = self.sort_dimension.get(&sort).unwrap();
            let embedding = self.function_embedding.get(&sort).unwrap();
            let mut input = Tensor::zeros(&[1, *dimension], (tch::Kind::Float, tch::Device::Cpu));
            for arg in args {
                input = input.cat(&self.term_embedding(arg), 1);
            }
            embedding.forward(&input)
        }
    }
}
  