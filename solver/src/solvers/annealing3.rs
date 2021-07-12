use crate::common::*;
use geo::algorithm::coords_iter::CoordsIter;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

static SEED: [u8; 32] = [
    0xfd, 0x00, 0xf1, 0x5c, 0xde, 0x01, 0x11, 0xc6, 0xc3, 0xea, 0xfb, 0xbf, 0xf3, 0xca, 0xd8, 0x32,
    0x6a, 0xe3, 0x07, 0x99, 0xc5, 0xe0, 0x52, 0xe4, 0xaa, 0x35, 0x07, 0x99, 0xe3, 0x2b, 0x9d, 0xc6,
];

fn tscore(solution: &Vec<Point>, input: &Input) -> (f64, f64) {
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

    (
        dislike / (input.hole.exterior().coords_count() as f64),
        -(vx + vy),
    )
}
fn ascore(value: (f64, f64), progress: f64) -> f64 {
    value.0 * progress + (1.0 - progress) * value.1
}

pub fn solve(
    input: &Input,
    mut solution: Vec<Point>,
    time_limit: Duration,
    fix_seed: bool,
    initial_temperature: f64,
) -> (Vec<Point>, f64) {
    let n = solution.len();
    let mut rng = if fix_seed {
        SmallRng::from_seed(SEED)
    } else {
        SmallRng::from_entropy()
    };
    let mut current_score = tscore(&solution, &input);
    let out_edges = make_out_edges(&input.figure.edges, n);
    let original_vertices = &input.figure.vertices;
    let mut orders = vec![vec![]; n];
    for i in 0..n {
        orders[i] = make_determined_order(&out_edges, Some(i));
    }
    let start_at = Instant::now();

    let mut best_solution = solution.clone();
    let mut best_score = current_score;

    let mut progress = 0.0;
    let mut temperature = initial_temperature;
    eprintln!("initial_temperature = {}", initial_temperature);

    let distance_sums = calc_distance_sums(&out_edges, original_vertices.len());
    let distance_total: usize = distance_sums.iter().sum();
    // eprintln!("{} {:?}", distance_total, distance_sums);

    let mut iter = 0;
    let mut move_count = 0;
    loop {
        // check time limit
        iter += 1;
        if iter % 100 == 0 {
            let elapsed = Instant::now() - start_at;
            if best_score.0 == 0.0 || elapsed >= time_limit {
                eprintln!("iter = {}, move_count = {}", iter, move_count);
                let dislike = calculate_dislike(&best_solution, &input.hole);
                return (best_solution, dislike);
            }

            // tweak temperature
            progress = elapsed.as_secs_f64() / time_limit.as_secs_f64();
            temperature = initial_temperature * (1.0 - progress) * (-progress).exp2();
        }

        // move to neighbor
        let r = rng.gen::<f64>();
        if r > progress {
            let mut i = 0;
            {
                let r = rng.gen::<usize>() % distance_total;
                let mut sum = 0;
                for index in 0..n {
                    sum += distance_sums[index];
                    if r < sum {
                        i = index;
                        break;
                    }
                }
            }
            let w = rng.gen::<usize>() % 40 + 5;
            let next_solution =
                random_move_one_point(i, w, &solution, &input, &mut rng, &out_edges, &orders);
            if next_solution.is_none() {
                continue;
            }
            move_count += 1;
            let next_solution = next_solution.unwrap();

            // calculate score. FIXME: slow
            let new_score = tscore(&next_solution, &input);

            let accept = {
                let current = ascore(current_score, progress);
                let new = ascore(new_score, progress);
                if new < current {
                    true
                } else {
                    // new_score >= current_score
                    let delta = new - current;
                    let accept_prob = (-delta / temperature).exp();
                    rng.gen::<f64>() < accept_prob
                }
            };

            if accept {
                // accept candidate
                current_score = new_score;
                solution = next_solution;
            }
        } else {
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
            let new_score = tscore(&solution, &input);

            let accept = {
                let current = ascore(current_score, progress);
                let new = ascore(new_score, progress);
                if new < current {
                    true
                } else {
                    // new_score >= current_score
                    let delta = new - current;
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

fn random_move_one_point(
    from: usize,
    w: usize,
    solution: &Vec<Point>,
    input: &Input,
    rng: &mut SmallRng,
    out_edges: &Vec<Vec<usize>>,
    orders: &Vec<Vec<usize>>,
) -> Option<Vec<Point>> {
    let mut gx: f64 = 0.0;
    let mut gy: f64 = 0.0;
    for p in solution.iter() {
        gx += p.x();
        gy += p.y();
    }
    gx /= solution.len() as f64;
    gy /= solution.len() as f64;
    let g = Point::new(gx, gy);
    let mut vect1 = solution[from] - g;
    vect1 = vect1 / squared_distance(&solution[from], &g).sqrt();

    let mut np = solution[from];
    for iter in 0..5 {
        let dx = (rng.gen::<usize>() % (w * 2 + 1)) as f64 - w as f64;
        let dy = (rng.gen::<usize>() % (w * 2 + 1)) as f64 - w as f64;
        let vect2 = Point::new(dx, dy) / (dx * dx + dy * dy).sqrt();
        np = solution[from] + Point::new(dx, dy);
        if vect1.dot(vect2) > 0.4 - iter as f64 * 0.2 {
            break;
        }
    }
    if solution[from] == np {
        return None;
    }

    let mut solution = solution.clone();
    let old = solution[from];
    solution[from] = np;
    let next_solution =
        fix_allowed_distance_violation(from, &solution, &input, &out_edges, &orders);
    solution[from] = old;
    return next_solution;
}

fn calc_distance_sums(edges: &Vec<Vec<usize>>, n: usize) -> Vec<usize> {
    let mut ret = vec![0; n];
    for start in 0..n {
        let mut visited = vec![false; n];
        visited[start] = true;
        let mut que = VecDeque::new();
        que.push_back((start, 0));
        while let Some((from, dist)) = que.pop_front() {
            ret[start] += dist * dist;
            // ret[start] += dist;
            for &to in edges[from].iter() {
                if visited[to] {
                    continue;
                }
                visited[to] = true;
                que.push_back((to, dist + 1));
            }
        }
    }
    return ret;
}
