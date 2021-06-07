use rand::Rng;
use super::state::State;
use std::cmp;

pub trait AcceptanceCriteria<S: State> {
  fn accept(&mut self, best: &S, current: &S, candidate: &S) -> bool;
}

pub struct HillClimbAcceptanceCriteria {}
impl HillClimbAcceptanceCriteria {
    pub fn new() -> Self {
        return HillClimbAcceptanceCriteria {};
    }
}

impl<S: State> AcceptanceCriteria<S> for HillClimbAcceptanceCriteria {
    fn accept(&mut self, best: &S, current: &S, candidate: &S) -> bool {
        if candidate.objective() < current.objective() {
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
    fn accept(&mut self, best: &S, current: &S, candidate: &S) -> bool {
        let probability = ((current.objective() - candidate.objective()) as f64 / self.temperature).exp();
        self.temperature *= self.c;
        let mut rng = rand::thread_rng();
        let judge = rng.gen();
        return probability >= judge;
    }
}

pub struct RecordToRecordTravelAcceptanceCriteria {
    threshold: f64,
    end_threshold: f64,
    step: f64,
}

impl RecordToRecordTravelAcceptanceCriteria {
    pub fn new(start_th: f64, end_th: f64, iterations: u64) -> Self {
        let step = (start_th - end_th) / (iterations as f64);

        return RecordToRecordTravelAcceptanceCriteria {
            threshold: start_th,
            end_threshold: end_th,
            step: step,
        };
    }
}

impl<S: State> AcceptanceCriteria<S> for RecordToRecordTravelAcceptanceCriteria {
    fn accept(&mut self, best: &S, _current: &S, candidate: &S) -> bool {
        let candidate_obj = candidate.objective() as f64;
        let best_obj = best.objective() as f64;
        let score = (candidate_obj - best_obj) / candidate_obj;
        //println!("[DEBUG] candidate_obj: {}, best_obj: {}, (candidate - best_obj): {} score: {}, threshold: {}", candidate_obj, best_obj, candidate_obj - best_obj, score, self.threshold);
        let result;
        if score < self.threshold {
            result = true;
        } else {
            result = false;
        }
        let ths = vec![self.end_threshold, self.threshold - self.step];
        let new_th = ths.into_iter().fold(0.0/0.0, f64::max); 
        self.threshold = new_th;

        return result;
    }
}