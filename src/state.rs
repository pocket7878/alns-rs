use std::hash::Hash;

pub trait State: Clone + Hash + Eq {
  fn objective(&self) -> i64;
}