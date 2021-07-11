use crate::common::*;
use geo::{Closest, Coordinate};
use geo::prelude::*;
use geo::contains::Contains;
use std::time::{Duration, Instant};
use rand::prelude::*;
use rand::seq::SliceRandom;

static SEED: [u8; 32] = [
    0xfd, 0x00, 0xf1, 0x5c, 0xde, 0x01, 0x11, 0xc6, 0xc3, 0xea, 0xfb, 0xbf, 0xf3, 0xca, 0xd8, 0x32,
    0x6a, 0xe3, 0x07, 0x99, 0xc5, 0xe0, 0x52, 0xe4, 0xaa, 0x35, 0x07, 0x99, 0xe3, 0x2b, 0x9d, 0xc6,
];

type Vector2d = Coordinate<f64>;

fn vec2d(x: f64, y: f64) -> Vector2d { Vector2d { x, y } }

pub fn solve(input: &Input, time_limit: Duration) -> (Vec<Point>, f64) {
    let mut solution = input.figure.vertices.clone();

    let n = solution.len();
    //let mut rng = SmallRng::from_seed(SEED);

    let out_edges = make_out_edges(&input.figure.edges, n);
    let original_vertices = &input.figure.vertices;
    let start_at = Instant::now();

    let mut current_score = calculate_dislike(&solution, &input.hole);
    let mut best_solution = solution.clone();
    let mut best_score = current_score;

    let mut progress = 0.0;

    let mut velocities = vec![vec2d(0.0, 0.0); n];

    let scale_factor = {
        let mut min_x: f64 = 1e20;
        let mut min_y: f64 = 1e20;
        let mut max_x: f64 = -1e20;
        let mut max_y: f64 = -1e20;
        for p in input.hole.exterior().points_iter() {
            min_x = min_x.min(p.x());
            min_y = min_y.min(p.y());
            max_x = max_x.max(p.x());
            max_y = max_y.max(p.y());
        }
        (max_x - min_x).max(max_y - min_y).max(1.0)
    };

    let mut iter = 0;
    loop {
        // check time limit
        iter += 1;
        if iter % 100 == 0 {
            let elapsed = Instant::now() - start_at;
            if best_score == 0.0 || elapsed >= time_limit {
                eprintln!("iter = {}", iter);
                let dislike = calculate_dislike(&best_solution, &input.hole);
                return (best_solution, dislike);
            }

            // tweak temperature
            progress = elapsed.as_secs_f64() / time_limit.as_secs_f64();
        }

        // move
        for i in 0..n {
            // i番目の点に加わる力を求める
            let mut force = vec2d(0.0, 0.0);
            let p0 = solution[i];
            let op0 = original_vertices[i];
            
            // 1. エッジで繋がっている点からの力
            let k = 1000.0 + 10000.0 * progress; // バネ定数
            for &neighbor in out_edges[i].iter() {
                let p1 = solution[neighbor];
                let op1 = original_vertices[neighbor];
                let x = distance(&p1, &p0) - distance(&op1, &op0);
                let f = k * x;
                let dir = (p1.0 - p0.0) / (distance(&p1, &p0) + 1e-8);
                force = force + dir * f;
            }

            // 2. 端点からの引力
            let g = 100.0; // 引力の係数
            for terminal in input.hole.exterior().points_iter() {
                let dist = distance(&terminal, &p0);
                let f = g / (pow2(dist) + 1e-8);
                let dir = (terminal.0 - p0.0) / (dist + 1e-8);
                force = force + dir * f;
            }

            // 3. ホールの外にいるときに戻る方向に働く力
            let r = 1000.0 + 10000.0 * progress; // この力の係数
            if !input.hole.contains(&p0) {
                if let Closest::Intersection(p1) | Closest::SinglePoint(p1) = input.hole.closest_point(&p0) {
                    let dist = distance(&p0, &p1);
                    let f = r;
                    let dir = (p1.0 - p0.0) / (dist + 1e-8);
                    force = force + dir * f;
                }
            }

            force = force * scale_factor;

            let mass = 10000.0; // 質量
            let time_delta = 0.005; // 1フレームの時間
            let a = force / mass;
            velocities[i] = velocities[i] + a * time_delta;
            solution[i] = (solution[i].0 + velocities[i] * time_delta).into();
        }

        // calculate score.
        current_score = calculate_dislike(&solution, &input.hole);
        if current_score < best_score {
            best_score = current_score;
            best_solution = solution.clone();
        }
    }
}
