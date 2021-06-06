mod acceptance;
mod destroy_operator;
mod repair_operator;
mod state;

use acceptance::AcceptanceCriteria;
use destroy_operator::DestroyOperator;
use rand::Rng;
use repair_operator::RepairOperator;
use state::State;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

fn select_with_weights<S>(arr: &Vec<Rc<S>>, weights: &[f64]) -> (usize, Rc<S>) {
  let mut upper_bound: f64 = 0.0;
  let mut sum_w = vec![0.0 as f64; weights.len()];
  for i in 0..weights.len() {
    upper_bound += weights[i];
    if i >= 1 {
      sum_w[i] = weights[i] + sum_w[i - 1];
    }
  }

  let mut rng = rand::thread_rng();
  let choice: f64 = rng.gen::<f64>() * upper_bound;
  let mut idx: usize = 0;
  for (i, w) in sum_w.iter().enumerate() {
    if choice < *w {
      idx = i;
      break;
    }
  }

  return (idx, arr[idx].clone());
}

pub struct Solver<S: State> {
  acceptance_criteria: Box<dyn AcceptanceCriteria<S>>,
  destroy_operators: Vec<Rc<RefCell<Box<dyn DestroyOperator<S>>>>>,
  repair_operators: Vec<Rc<RefCell<Box<dyn RepairOperator<S>>>>>,
}

impl<S: State> Solver<S> {
  pub fn new(acceptance_criteria: Box<dyn AcceptanceCriteria<S>>) -> Solver<S> {
    return Solver {
      acceptance_criteria: acceptance_criteria,
      destroy_operators: vec![],
      repair_operators: vec![],
    };
  }

  pub fn add_destroy_operator(&mut self, op: Box<dyn DestroyOperator<S>>) {
    self.destroy_operators.push(Rc::new(RefCell::new(op)));
  }

  pub fn add_repair_operator(&mut self, op: Box<dyn RepairOperator<S>>) {
    self.repair_operators.push(Rc::new(RefCell::new(op)));
  }

  pub fn run(
    mut self,
    initial_solution: S,
    weight_parameter: [f64; 3],
    reaction: f64,
    iterations: i64,
  ) -> S {
    let mut best_solution = initial_solution.clone();
    let mut current_solution = initial_solution.clone();
    let mut d_weights = vec![1.0 as f64; self.destroy_operators.len()];
    let mut r_weights = vec![1.0 as f64; self.repair_operators.len()];

    let segment_size = 200;
    let mut seg_d_count = vec![0; self.destroy_operators.len()];
    let mut seg_r_count = vec![0; self.repair_operators.len()];
    let mut seg_d_score = vec![1.0 as f64; self.destroy_operators.len()];
    let mut seg_r_score = vec![1.0 as f64; self.repair_operators.len()];
    let mut accepted_states_set: HashSet<S> = HashSet::new();
    let mut current_iteration = 1;

    while current_iteration <= iterations {
      let (didx, drc) = select_with_weights(&self.destroy_operators, &d_weights);
      let mut d = drc.borrow_mut();
      seg_d_count[didx] += 1;

      let (ridx, rrc) = select_with_weights(&self.repair_operators, &r_weights);
      let mut r = rrc.borrow_mut();
      seg_r_count[ridx] += 1;

      let t = r.apply(&d.apply(&current_solution));

      let mut iteration_score = 0.0;
      let mut accepted = false;
      let mut new_best = false;
      let mut not_accepted_before = false;
      let mut better_than_current = false;

      if !accepted_states_set.contains(&t) {
        not_accepted_before = true;
      }

      if t.objective() < current_solution.objective() {
        better_than_current = true;
      }

      if self.acceptance_criteria.accept(&current_solution, &t) {
        current_solution = t.clone();
        accepted = true;
        accepted_states_set.insert(t);
      }

      if current_solution.objective() < best_solution.objective() {
        best_solution = current_solution.clone();
        new_best = true;
      }

      // Update Observed score
      if accepted && new_best {
        iteration_score = weight_parameter[0];
      } else if not_accepted_before && better_than_current {
        iteration_score = weight_parameter[1];
      } else if not_accepted_before && !better_than_current && accepted {
        iteration_score = weight_parameter[2];
      }
      seg_d_score[didx] += iteration_score;
      seg_d_score[ridx] += iteration_score;

      if current_iteration % segment_size == 0 {
        // Segment finished update weights
        for di in 0..self.destroy_operators.len() {
          if seg_d_count[di] > 0 {
            d_weights[di] = (1.0 - reaction) * d_weights[di]
              + reaction * seg_d_score[di] / (seg_d_count[di] as f64);
          }
        }
        for ri in 0..self.repair_operators.len() {
          if seg_r_count[ri] > 0 {
            r_weights[ri] = (1.0 - reaction) * r_weights[ri]
              + reaction * seg_r_score[ri] / (seg_r_count[ri] as f64);
          }
        }
        // Reset segment scores
        seg_d_count = vec![0; self.destroy_operators.len()];
        seg_r_count = vec![0; self.repair_operators.len()];
        seg_d_score = vec![1.0 as f64; self.destroy_operators.len()];
        seg_r_score = vec![1.0 as f64; self.repair_operators.len()];
      }

      current_iteration += 1;
    }

    return best_solution;
  }
}
