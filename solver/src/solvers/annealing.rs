use crate::common::*;
use std::time::{Duration, Instant};
use rand::prelude::*;

static SEED: [u8; 32] = [
    0xfd, 0x00, 0xf1, 0x5c, 0xde, 0x01, 0x11, 0xc6, 0xc3, 0xea, 0xfb, 0xbf, 0xf3, 0xca, 0xd8, 0x32,
    0x6a, 0xe3, 0x07, 0x99, 0xc5, 0xe0, 0x52, 0xe4, 0xaa, 0x35, 0x07, 0x99, 0xe3, 0x2b, 0x9d, 0xc6,
];

pub fn solve(input: &Input, mut solution: Vec<Point>, time_limit: Duration) -> (Vec<Point>, f64) {
    let n = solution.len();
    let mut rng = SmallRng::from_seed(SEED);
    let mut current_score = calculate_dislike(&solution, &input.hole);
    let out_edges = make_out_edges(&input.figure.edges, n);
    let original_vertices = &input.figure.vertices;
    let start_at = Instant::now();

    let mut best_solution = solution.clone();
    let mut best_score = current_score;

    let initial_temperature = 10000.0;
    let mut temperature = initial_temperature;

    let mut iter = 0;
    loop {
        // check time limit
        iter += 1;
        if iter % 100 == 0 {
            let elapsed = Instant::now() - start_at;
            if best_score == 0.0 || elapsed >= time_limit {
                return (best_solution, best_score);
            }

            // tweak temperature
            let progress = elapsed.as_secs_f64() / time_limit.as_secs_f64();
            temperature = initial_temperature * (1.0 - progress) * (-progress).exp2();
        }

        // move to neighbor
        let i = rng.gen::<usize>() % n;
        let candidates = make_next_candidates(i, original_vertices, &input.hole, input.epsilon, &solution, &out_edges);
        let candidate = candidates[rng.gen_range(0..candidates.len())];

        // calculate score. FIXME: slow
        let old = solution[i];
        solution[i] = candidate;
        let new_score = calculate_dislike(&solution, &input.hole);

        let accept = {
            if new_score < current_score {
                true
            } else {
                // new_score >= current_score
                let delta = new_score - current_score;
                let accept_prob = (-delta / temperature).exp();
                rng.gen::<f64>() < accept_prob
            }
        };

        if accept {
            // accept candidate
            current_score = new_score;
        } else {
            // reject candidate
            solution[i] = old;
        }

        if current_score < best_score {
            best_score = current_score;
            best_solution = solution.clone();
        }
    }
}

fn make_next_candidates(
    i: usize,
    original_vertices: &[Point],
    hole: &Polygon,
    epsilon: i64,
    solution: &[Point],
    out_edges: &[Vec<usize>],
) -> Vec<Point> {
    let some_neighbor = out_edges[i][0];
    let original_squared_distance = squared_distance(&original_vertices[i], &original_vertices[some_neighbor]);
    let ring = Ring::from_epsilon(solution[some_neighbor], epsilon, original_squared_distance);

    let mut candidates = vec![];
    for &p in ring_points(&ring).iter() {
        let ok = out_edges[i].iter().all(|&dst| {
            is_allowed_distance(
                    &p,
                    &solution[dst],
                    &original_vertices[i],
                    &original_vertices[dst],
                    epsilon,
            ) && does_line_fit_in_hole(&p, &solution[dst], hole)
        });
        if ok {
            candidates.push(p);
        }
    }

    candidates
}