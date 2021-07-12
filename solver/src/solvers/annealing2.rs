use crate::common::*;
use geo::algorithm::coords_iter::CoordsIter;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

static SEED: [u8; 32] = [
    0xfd, 0x00, 0xf1, 0x5c, 0xde, 0x01, 0x11, 0xc6, 0xc3, 0xea, 0xfb, 0xbf, 0xf3, 0xca, 0xd8, 0x32,
    0x6a, 0xe3, 0x07, 0x99, 0xc5, 0xe0, 0x52, 0xe4, 0xaa, 0x35, 0x07, 0x99, 0xe3, 0x2b, 0x9d, 0xc6,
];

fn ascore(solution: &Vec<Point>, input: &Input) -> f64 {
    let dislike = calculate_dislike(&solution, &input.hole);

    let mut gx: f64 = 0.0;
    let mut gy: f64 = 0.0;
    for p in solution.iter() {
        gx += p.x();
        gy += p.y();
    }
    gx /= solution.len() as f64;
    gy /= solution.len() as f64;

    let mut vx: f64 = 0.0;
    let mut vy: f64 = 0.0;
    for p in solution.iter() {
        vx += pow2(p.x() - gx);
        vy += pow2(p.y() - gy);
    }
    vx /= solution.len() as f64;
    vy /= solution.len() as f64;

    dislike / (input.hole.exterior().coords_count() as f64) - (vx + vy) * 1.0
}

pub fn solve(
    input: &Input,
    mut solution: Vec<Point>,
    time_limit: Duration,
    fix_seed: bool,
) -> (Vec<Point>, f64) {
    let n = solution.len();
    let mut rng = if fix_seed {
        SmallRng::from_seed(SEED)
    } else {
        SmallRng::from_entropy()
    };
    let mut current_score = ascore(&solution, &input);
    let out_edges = make_out_edges(&input.figure.edges, n);
    let original_vertices = &input.figure.vertices;
    let start_at = Instant::now();

    let mut best_solution = solution.clone();
    let mut best_score = current_score;

    let initial_temperature = 10000.0;
    let mut temperature = initial_temperature;

    let mut iter = 0;
    let mut move_count = 0;
    loop {
        // check time limit
        iter += 1;
        if iter % 100 == 0 {
            let elapsed = Instant::now() - start_at;
            if best_score == 0.0 || elapsed >= time_limit {
                eprintln!("iter = {}, move_count = {}", iter, move_count);
                let dislike = calculate_dislike(&best_solution, &input.hole);
                return (best_solution, dislike);
            }

            // tweak temperature
            let progress = elapsed.as_secs_f64() / time_limit.as_secs_f64();
            temperature = initial_temperature * (1.0 - progress) * (-progress).exp2();
        }

        // move to neighbor
        let i = rng.gen::<usize>() % n;
        let candidate = make_next_candidates(
            i,
            original_vertices,
            &input.hole,
            input.epsilon,
            &solution,
            &out_edges,
            &mut rng,
        );
        if candidate != original_vertices[i] {
            move_count += 1;
        }

        // calculate score. FIXME: slow
        let old = solution[i];
        solution[i] = candidate;
        let new_score = ascore(&solution, &input);

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
    rng: &mut SmallRng,
) -> Point {
    let some_neighbor = out_edges[i][0];
    let original_squared_distance =
        squared_distance(&original_vertices[i], &original_vertices[some_neighbor]);
    if original_squared_distance < 100.0 || epsilon < 100000 {
        let ring = Ring::from_epsilon(solution[some_neighbor], epsilon, original_squared_distance);

        let mut points = ring_points(&ring);
        points.shuffle(rng);
        for &p in points.iter() {
            if !is_valid_point_move(i, &p, solution, original_vertices, out_edges, hole, epsilon) {
                continue;
            }
            return p;
        }
    } else {
        let od = original_squared_distance.sqrt();
        let low = od * (1.0 - epsilon as f64 / 1000000.0).sqrt();
        let high = od * (1.0 + epsilon as f64 / 1000000.0).sqrt();
        for _iter in 0..100 {
            let d = low + (high - low) * rng.gen::<f64>();
            let theta = 2.0 * std::f64::consts::PI * rng.gen::<f64>();
            let vect = Point::new(
                (theta.cos() * d + 0.5).floor(),
                (theta.sin() * d + 0.5).floor(),
            );
            let p = solution[some_neighbor] + vect;
            if !is_valid_point_move(i, &p, solution, original_vertices, out_edges, hole, epsilon) {
                continue;
            }
            return p;
        }
        return solution[i];
    }
    unreachable!()
}

fn is_valid_point_move(
    index: usize,
    p: &Point,
    solution: &[Point],
    original_vertices: &[Point],
    out_edges: &[Vec<usize>],
    hole: &Polygon,
    epsilon: i64,
) -> bool {
    let ok1 = out_edges[index].iter().all(|&dst| {
        is_allowed_distance(
            &p,
            &solution[dst],
            &original_vertices[index],
            &original_vertices[dst],
            epsilon,
            false,
        )
    });
    if !ok1 {
        return false;
    }
    let ok2 = out_edges[index]
        .iter()
        .all(|&dst| does_line_fit_in_hole(&p, &solution[dst], hole));
    if !ok2 {
        return false;
    }
    return true;
}

#[test]
fn test_minimum_cut() {
    let out_edges = vec![
        /* 0 */ vec![1, 2, 3],
        /* 1 */ vec![0, 2, 4],
        /* 2 */ vec![0, 1, 3, 5],
        /* 3 */ vec![0, 2],
        /* 4 */ vec![1, 5, 7],
        /* 5 */ vec![2, 4, 6],
        /* 6 */ vec![5, 7],
        /* 7 */ vec![4, 5, 6],
    ];
    let cut = minimum_cut(&out_edges, 0, 7);
    assert_eq!(cut, vec![
        Edge::new(1, 4),
        Edge::new(2, 5),
    ]);
}

fn minimum_cut(out_edges: &[Vec<usize>], s: usize, t: usize) -> Vec<Edge> {
    let n = out_edges.len();
    let mut flow = vec![vec![0; n]; n];
    let mut capacity = vec![vec![0; n]; n];

    maximum_flow(out_edges, s, t, &mut capacity, &mut flow);

    fn visit(
        v: usize,
        out_edges: &[Vec<usize>],
        capacity: &[Vec<i32>], flow: &[Vec<i32>],
        visited: &mut [bool],
    ) {
        if visited[v] {
            return;
        }
        visited[v] = true;

        for &w in out_edges[v].iter() {
            if visited[w] {
                continue;
            }
            if capacity[v][w] - flow[v][w] == 0 {
                continue;
            }
            visit(w, out_edges, capacity, flow, visited);
        }
    }

    let mut visited = vec![false; n];
    visit(s, out_edges, &capacity, &flow, &mut visited);

    let mut cut: Vec<Edge> = vec![];
    for v in 0..n {
        if !visited[v] {
            continue;
        }
        for &w in out_edges[v].iter() {
            if !visited[w] {
                cut.push(Edge::new(v, w));
            }
        }
    }

    cut
}

fn maximum_flow(
    out_edges: &[Vec<usize>], s: usize, t: usize,
    capacity: &mut [Vec<i32>], flow: &mut [Vec<i32>],
) -> i32 {

    fn residue(s: usize, t: usize, capacity: &[Vec<i32>], flow: &[Vec<i32>]) -> i32 {
        capacity[s][t] - flow[s][t]
    }

    fn augment(
        out_edges: &[Vec<usize>], capacity: &[Vec<i32>], flow: &mut [Vec<i32>],
        level: &[i32], finished: &mut [bool], u: usize, t: usize, cur: i32,
    ) -> i32 {
        if u == t || cur == 0 {
            return cur;
        }
        if finished[u] {
            return 0;
        }
        finished[u] = true;
        for &dst in out_edges[u].iter() {
            if level[dst] > level[u] {
                let f = augment(out_edges, capacity, flow, level, finished,
                    dst, t, cur.min(residue(u, dst, capacity, flow)));
                if f > 0 {
                    flow[u][dst] += f;
                    flow[dst][u] -= f;
                    finished[u] = false;
                    return f;
                }
            }
        }
        return 0;
    }

    let n = out_edges.len();

    for src in 0..n {
        for &dst in out_edges[src].iter() {
            capacity[src][dst] += 1;
        }
    }

    let mut total: i32 = 0;
    let mut cont = true;
    while cont {
        cont = false;

        // make layered network
        let mut level: Vec<i32> = vec![-1; n];
        level[s] = 0; 

        let mut queue = VecDeque::<usize>::new();
        queue.push_back(s);

        let mut d = n as i32;
        while queue.len() > 0 && level[*queue.front().unwrap()] < d {
            let u = queue.pop_back().unwrap();
            if u == t {
                d = level[u];
            }
            for &dst in out_edges[u].iter() {
                if residue(u, dst, capacity, flow) > 0 && level[dst] == -1 {
                    queue.push_back(dst);
                    level[dst] = level[u] + 1;
                }
            }
        }

        // make blocking flows
        let mut finished = vec![false; n];
        let mut f = 1;
        while f > 0 {
            f = augment(out_edges, capacity, flow, &mut level, &mut finished, s, t, 10000000);
            if f == 0 {
                break;
            }
            total += f;
            cont = true;
        }
    }
    total
}
