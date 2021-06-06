extern crate tspf;

use std::hash::{Hash, Hasher};
use std::path::Path;
use rand::Rng;
use rand::seq::SliceRandom;
use tspf::metric::MetricPoint;
use alns_rs::Solver;
use alns_rs::{State, Objective};
use alns_rs::acceptance::{HillClimbAcceptanceCriteria, SimulatedAnnealingAcceptanceCriteria};
use alns_rs::DestroyOperator;
use alns_rs::RepairOperator;

type VisitId = u64;
type Distance = u64;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct Problem {
    visits: Vec<VisitId>,
    distance_map: Vec<Vec<Distance>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct TSPState<'a> {
    problem: &'a Problem,
    route: Vec<VisitId>,
    bank: Vec<VisitId>,
}

impl<'a> Hash for TSPState<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.route.hash(state);
        let mut bs = self.bank.iter().collect::<Vec<_>>();
        bs.sort();
        bs.hash(state);
    }
}

impl<'a> TSPState<'a> {
    fn new(problem: &'a Problem) -> TSPState<'a> {
        return TSPState {
            problem: problem,
            route: problem.visits.clone(),
            bank: vec![],
        }
    }
}

impl<'a> State for TSPState<'a> {
    fn objective(&self) -> Objective {
        // Calc route cost
        let mut cost: u64 = 0;
        let route_len = self.route.len();
        for (i, v) in self.route.iter().enumerate() {
            let to = self.route[(i + 1 + route_len) % route_len];
            cost += self.problem.distance_map[*v as usize][to as usize];
        }

        return cost + 1000 * self.bank.len() as u64;
    }
}

/*
 * Destroy Operators
 */
// Random delete visit from route
struct RandomDestroyOperator {}

impl<'a> DestroyOperator<TSPState<'a>> for RandomDestroyOperator {
    fn apply(&mut self, state: &TSPState<'a>) -> TSPState<'a> {
        let destruction_degree = 0.25;
        let mut route = state.route.clone();
        let route_len = route.len();
        let number_of_removal = ((route_len as f64) * destruction_degree) as u64;

        let mut rng = rand::thread_rng();
        let mut bank = state.bank.clone();
        for _ in 0..number_of_removal {
            let delete_idx = rng.gen_range(0..route.len());
            let delete_visit_id = route[delete_idx];
            route.remove(delete_idx);
            bank.push(delete_visit_id);
        }

        return TSPState {
            problem: state.problem,
            route: route,
            bank: bank,
        }
    }
}

struct WorstRemovalOperator {}
impl<'a> DestroyOperator<TSPState<'a>> for WorstRemovalOperator {
    fn apply(&mut self, state: &TSPState<'a>) -> TSPState<'a> {
        let destruction_degree = 0.25;
        let mut bank = state.bank.clone();
        let mut route = state.route.clone();
        let route_len = route.len() as isize;
        let number_of_removal = ((route_len as f64) * destruction_degree) as u64;

        // Calc each node cost
        let mut node_costs: Vec<(VisitId, Distance)> = vec![];
        for (i, r) in route.iter().enumerate()  {
            let to = route[(((i as isize) + 1 + route_len) % route_len) as usize];
            let cost = state.problem.distance_map[*r as usize][to as usize];
            node_costs.push((*r, cost));
        }
        node_costs.sort_by(|a, b| {
            return b.1.partial_cmp(&a.1).unwrap();
        });

        for i in 0..number_of_removal {
            let visit_id = node_costs[i as usize].0;
            let index = route.iter().position(|&r| r == visit_id).unwrap();
            route.remove(index);
            bank.push(visit_id);
        }

        return TSPState {
            problem: state.problem,
            route: route,
            bank: bank,
        }
    }
}

struct PathRemovalOperator {}
impl<'a> DestroyOperator<TSPState<'a>> for PathRemovalOperator {
    fn apply(&mut self, state: &TSPState<'a>) -> TSPState<'a> {
        let destruction_degree = 0.25;
        let mut bank = state.bank.clone();
        let mut route = state.route.clone();
        let route_len = route.len() as isize;
        let number_of_removal = ((route_len as f64) * destruction_degree) as u64;

        let mut rng = rand::thread_rng();
        let start_idx = rng.gen_range(0..route.len());
        let mut remove_visits = vec![];
        remove_visits.push(route[start_idx]);
        for i in 1..number_of_removal {
            remove_visits.push(
                route[(((start_idx as isize) + (i as isize) + route_len) % route_len) as usize]
            );
        }

        for rv in remove_visits {
            let index = route.iter().position(|&r| r == rv).unwrap();
            route.remove(index);
            bank.push(rv);
        }

        return TSPState {
            problem: state.problem,
            route: route,
            bank: bank,
        }
    }
}

/*
 * Repair Operator
 */
// Greedy repair insert by distance
struct GreedyRepairOperator {}

impl<'a> RepairOperator<TSPState<'a>> for GreedyRepairOperator {
    fn apply(&mut self, state: &TSPState<'a>) -> TSPState<'a> {
        let mut rng = rand::thread_rng();
        let mut new_route = state.route.clone();
        let mut bank = state.bank.clone();
        bank.shuffle(&mut rng);
        // Insert each visit in bank into minimum cost position;
        for b in bank {
            let route_len = new_route.len() as i64;
            if route_len == 0 {
                new_route.push(b);
                continue;
            }

            let mut insert_point = 0;
            let mut minimum_cost = u64::MAX;
            for i in 0..new_route.len() {
                let from = new_route[(((i as i64) - 1 + route_len) % route_len) as usize];
                let to = new_route[i];
                let cost = state.problem.distance_map[b as usize][to as usize]
                            + state.problem.distance_map[from as usize][b as usize];
                if cost < minimum_cost {
                    minimum_cost = cost;
                    insert_point = i;
                }
            }
            new_route.insert(insert_point, b);
        }

        return TSPState {
            problem: state.problem,
            route: new_route,
            bank: vec![],
        }
    }
}

fn main() {
    // Load Problem
    let path = "data/xqf131.tsp";
    let tsp = tspf::TspBuilder::parse_path(Path::new(path)).unwrap();
    // Build distance map
    let points = tsp.node_coords();
    let mut distance_map = vec![vec![0 as u64; points.len()]; points.len()];
    for i in 0..points.len() {
        let from = points[i];
        for j in (i + 1)..points.len() {
            let to = points[j];
            let dist = ((to.x() - from.x()).powf(2.0) + (to.y() - from.y()).powf(2.0)).sqrt() as u64;
            distance_map[i][j] = dist;
            distance_map[j][i] = dist;
        }
    }
    // Build Problem
    let visits = (0..(tsp.dim() as u64)).collect::<Vec<VisitId>>();
    let problem = Problem {
        visits: visits.clone(),
        distance_map: distance_map,
    };
    // Build initial solution
    let empty_state = TSPState {
        problem: &problem,
        route: vec![],
        bank: visits,
    };
    let mut greedy_repair = GreedyRepairOperator{};
    let initial_state = greedy_repair.apply(&empty_state);
    println!("initial objective: {:?}", initial_state.objective());
    // Build solver
    let random_destroy = RandomDestroyOperator{};
    let worst_removal_destroy = WorstRemovalOperator{};
    let path_removal_destroy = PathRemovalOperator{};
    let simulated_annealing_acceptance = SimulatedAnnealingAcceptanceCriteria::new(
        initial_state.objective(),
        0.5,
        0.9998,
    );
    let hill_climbing_acceptance = HillClimbAcceptanceCriteria{};
    let mut solver = Solver::new(
        Box::new(hill_climbing_acceptance),
    );
    solver.add_repair_operator(Box::new(greedy_repair));
    solver.add_destroy_operator(Box::new(random_destroy));
    solver.add_destroy_operator(Box::new(worst_removal_destroy));
    solver.add_destroy_operator(Box::new(path_removal_destroy));
    let result = solver.run(
        initial_state,
        [3.0, 2.0, 1.0],
        0.8,
        50000,
    );
    println!("Result score: {:?}", result.objective());
}
