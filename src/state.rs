use std::hash::Hash;

pub type Objective = u64;

pub trait State: Clone + Hash + Eq {
  fn objective(&self) -> Objective;
}