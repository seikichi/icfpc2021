use crate::common::*;
use geo::algorithm::centroid::Centroid;
use geo::algorithm::contains::Contains;
// use rand::rngs::SmallRng;
// use rand::seq::SliceRandom;
// use rand::{Rng, SeedableRng};

// static SEED: [u8; 32] = [
//     0xfd, 0x00, 0xf1, 0x5c, 0xde, 0x01, 0x11, 0xc6, 0xc3, 0xea, 0xfb, 0xbf, 0xf3, 0xca, 0xd8, 0x32,
//     0x6a, 0xe3, 0x07, 0x99, 0xc5, 0xe0, 0x52, 0xe4, 0xaa, 0x35, 0x07, 0x99, 0xe3, 0x2b, 0x9d, 0xc6,
// ];

struct Solver {
    original_vertices: Vec<Point>, // readonly
    out_edges: Vec<Vec<usize>>,    // readonly
    epsilon: i64,                  // readonly
    hole: Polygon,                 // readonly
    holl_points: Vec<Point>,       // readonly
    // rng: SmallRng,                 // mutable
}

pub fn solve(input: &Input) -> Option<(Vec<Point>, f64)> {
    let mut solver = Solver {
        original_vertices: input.figure.vertices.clone(),
        out_edges: make_out_edges(&input.figure.edges, input.figure.vertices.len()),
        epsilon: input.epsilon,
        hole: input.hole.clone(),
        holl_points: all_point_in_hole(&input.hole),
        // rng: SmallRng::from_seed(SEED),
    };
    let mut vertices = input.figure.vertices.clone();
    let mut visited = vec![false; input.figure.vertices.len()];
    let order = solver.reorder(vertices.len());

    if solver.naive_dfs(0, &mut vertices, &mut visited, &order) {
        let dislike = calculate_dislike(&vertices, &input.hole);
        Some((vertices, dislike))
    } else {
        None
    }
}

impl Solver {
    /*
    fn dfs(&self, i: usize, vertices: &mut [Point], visited: &mut [bool]) {
        for &j in self.out_edges[i].iter() {
            if visited[j] {
                continue;
            }
            let v = vertices[i];
            let ov = self.original_vertices[i];
            let ow = self.original_vertices[j];
            let original_squared_distance = squared_distance(&ov, &ow);
            let ring = Ring::from_epsilon(v, self.epsilon, original_squared_distance);
            for p in ring_points(&ring).iter() {
                // TODO: check p is valid
                vertices[j] = *p;
                visited[j] = true;
                self.dfs(j, vertices, visited);
            }
            visited[j] = false;
        }
    }
    */

    fn reorder(&self, n: usize) -> Vec<usize> {
        let mut order = vec![0; n];
        let mut determined = vec![false; n];
        for i in 0..n {
            let mut best = (0, 0, 0);
            for j in 0..n {
                if determined[j] {
                    continue;
                }
                let mut jisu = 0;
                for &k in self.out_edges[j].iter() {
                    if determined[k] {
                        jisu += 1;
                    }
                }
                let score = (jisu, self.out_edges[j].len(), j);
                if best < score {
                    best = score;
                }
            }
            order[i] = best.2;
            determined[order[i]] = true;
        }
        order
    }

    fn naive_dfs(
        &mut self,
        i: usize,
        vertices: &mut [Point],
        visited: &mut [bool],
        order: &[usize],
    ) -> bool {
        if i == self.original_vertices.len() {
            return true;
        }
        let src = order[i];
        visited[src] = true;

        let holl_points = self.holl_points.clone();
        //holl_points.shuffle(&mut self.rng);
        for &p in holl_points.iter() {
            vertices[src] = p;

            // verify
            let ok = self.out_edges[src].iter().all(|&dst| {
                !visited[dst]
                    || (is_allowed_distance(
                        &vertices[src],
                        &vertices[dst],
                        &self.original_vertices[src],
                        &self.original_vertices[dst],
                        self.epsilon,
                    ) && does_line_fit_in_hole(&vertices[src], &vertices[dst], &self.hole))
            });

            if ok {
                if self.naive_dfs(i + 1, vertices, visited, order) {
                    visited[src] = false;
                    return true;
                }
            }
        }

        visited[src] = false;
        false
    }
}

fn each_point_in_hole(hole: &Polygon, mut f: impl FnMut(Point)) {
    let mut min_x: f64 = 1e20;
    let mut max_x: f64 = -1e20;
    let mut min_y: f64 = 1e20;
    let mut max_y: f64 = -1e20;
    hole.exterior().points_iter().for_each(|p| {
        min_x = min_x.min(p.x());
        max_x = max_x.max(p.x());
        min_y = min_y.min(p.y());
        max_y = max_y.max(p.y());
    });
    for y in (min_y.ceil() as i64)..=(max_y as i64) {
        for x in (min_x.ceil() as i64)..=(max_x as i64) {
            let p = Point::new(x as f64, y as f64);
            if hole.contains(&p) || hole.exterior().contains(&p) {
                f(p)
            }
        }
    }
}

fn all_point_in_hole(hole: &Polygon) -> Vec<Point> {
    let mut ps = vec![];
    each_point_in_hole(hole, |p| {
        ps.push(p);
    });
    let c = hole.centroid().unwrap();
    ps.sort_by_key(|p| squared_distance(p, &c) as i64);
    return ps;
}
