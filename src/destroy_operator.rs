use super::state::State;

pub trait DestroyOperator<S: State> {
  fn apply(&mut self, solution: &S) -> S;
}
