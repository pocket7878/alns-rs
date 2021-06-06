use super::state::State;

pub trait AcceptanceCriteria<S: State> {
  fn accept(&mut self, current: &S, temporary: &S) -> bool;
}