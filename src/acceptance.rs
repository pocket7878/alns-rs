use rand::Rng;
use super::state::State;

pub trait AcceptanceCriteria<S: State> {
  fn accept(&mut self, current: &S, temporary: &S) -> bool;
}

pub struct HillClimbAcceptanceCriteria {}
impl HillClimbAcceptanceCriteria {
    pub fn new() -> Self {
        return HillClimbAcceptanceCriteria {};
    }
}

impl<S: State> AcceptanceCriteria<S> for HillClimbAcceptanceCriteria {
    fn accept(&mut self, current: &S, temporary: &S) -> bool {
        if temporary.objective() < current.objective() {
            return true;
        } else {
            return false;
        }
    }
}

pub struct SimulatedAnnealingAcceptanceCriteria {
    temperature: f64,
    c: f64,
}

impl SimulatedAnnealingAcceptanceCriteria {
    pub fn new(initial_objective: u64, w: f64, c: f64) -> Self {
        let ln5 = w.ln();
        let start_temp: f64 = (-0.05 * initial_objective as f64) / ln5;
        
        return SimulatedAnnealingAcceptanceCriteria {
            temperature: start_temp,
            c
        };
    }
}

impl<S: State> AcceptanceCriteria<S> for SimulatedAnnealingAcceptanceCriteria {
    fn accept(&mut self, current: &S, temporary: &S) -> bool {
        let probability = ((current.objective() - temporary.objective()) as f64 / self.temperature).exp();
        self.temperature *= self.c;
        let mut rng = rand::thread_rng();
        let judge = rng.gen();
        return probability >= judge;
    }
}