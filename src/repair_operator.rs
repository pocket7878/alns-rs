use super::state::State;

pub trait RepairOperator<S: State> {
  fn apply(&mut self, solution: &S) -> S;
}