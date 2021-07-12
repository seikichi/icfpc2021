use crate::common::*;
use crate::solvers;
use rand::prelude::*;

static SEED: [u8; 32] = [
    0xfd, 0x00, 0xf1, 0x5c, 0xde, 0x01, 0x11, 0xc6, 0xc3, 0xea, 0xfb, 0xbf, 0xf3, 0xca, 0xd8, 0x32,
    0x6a, 0xe3, 0x07, 0x99, 0xc5, 0xe0, 0x52, 0xe4, 0xaa, 0x35, 0x07, 0x99, 0xe3, 0x2b, 0x9d, 0xc6,
];

pub fn solve(input: &Input, fix_seed: bool) -> Option<(Vec<Point>, f64)> {
    let big_box = Polygon::new(
        geo::LineString::from(vec![
            Point::new(-1e+9, -1e+9),
            Point::new(-1e+9, 1e+9),
            Point::new(1e+9, 1e+9),
            Point::new(1e+9, -1e+9),
        ]),
        vec![],
    );
    let mut solution = input.figure.vertices.clone();
    let mut temp_input = input.clone();
    let mut rng = if fix_seed {
        SmallRng::from_seed(SEED)
    } else {
        SmallRng::from_entropy()
    };
    let n = solution.len();
    let mut best_variance = calc_variance(&solution);

    let out_edges = make_out_edges(&input.figure.edges, n);
    let mut orders = vec![vec![]; n];
    for i in 0..n {
        orders[i] = make_determined_order(&out_edges, Some(i));
    }

    for iter in 0..10000 {
        if iter % (n * 10) == 0 {
            let temp = temp_input.figure.vertices;
            temp_input.figure.vertices = solution.clone();
            let ret = solvers::orthgonal::solve(&temp_input);
            temp_input.figure.vertices = temp;
            if ret.is_some() {
                return ret;
            }
            // eprintln!("orthgonal is failed: {}", iter);
        }
        let from = rng.gen::<usize>() % n;
        let offset = 5;
        let dx = (rng.gen::<usize>() % (offset * 2 + 1)) as f64 - offset as f64;
        let dy = (rng.gen::<usize>() % (offset * 2 + 1)) as f64 - offset as f64;
        let np = solution[from] + Point::new(dx, dy);
        let old = solution[from];

        solution[from] = np;
        let temp = temp_input.hole;
        temp_input.hole = big_box.clone();
        let next_solution =
            fix_allowed_distance_violation(from, &solution, &temp_input, &out_edges, &orders);
        temp_input.hole = temp;
        solution[from] = old;
        if next_solution.is_none() {
            continue;
        }
        // eprintln!("move success1");
        let next_solution = next_solution.unwrap();
        let variance = calc_variance(&next_solution);
        if variance < best_variance {
            // eprintln!("move success: {} {} {}", iter, dx, dy);
            solution = next_solution;
            best_variance = variance;
        }
    }
    return None;
}

fn calc_variance(solution: &Vec<Point>) -> f64 {
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
    return vx + vy;
}
