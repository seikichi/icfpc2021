use crate::common::*;
use geo::algorithm::contains::Contains;

struct Solver {
    original_vertices: Vec<Point>, // readonly
    edges: Vec<Edge>, // readonly
    out_edges: Vec<Vec<usize>>, // readonly
    epsilon: i64, // readonly
    vertices: Vec<Point>, // mutable
    visited: Vec<bool>, // mutable
}

pub fn solve(input: &Input) {   
    let mut solver = Solver {
        original_vertices: input.figure.vertices.clone(),
        edges: input.figure.edges.clone(),
        out_edges: make_out_edges(&input.figure.edges, input.figure.vertices.len()),
        epsilon: input.epsilon,
        vertices: input.figure.vertices.clone(),
        visited: vec![false; input.figure.vertices.len()],
    };

    each_point_in_hole(&input.hole, |v| {
        solver.vertices[0] = v;
        solver.visited[0] = true;
        solver.dfs(0);
    });
}

fn make_out_edges(edges: &[Edge], n_vertices: usize) -> Vec<Vec<usize>> {
    let mut out_edges = vec![vec![]; n_vertices];
    for e in edges.iter() {
        out_edges[e.v].push(e.w);
        out_edges[e.w].push(e.v);
    }
    out_edges
}

impl Solver {
    fn dfs(&mut self, i: usize) {
        for &j in self.out_edges[i].iter() {
            if self.visited[j] {
                continue;
            }
            let v = self.vertices[i];
            let ov = self.original_vertices[i];
            let ow = self.original_vertices[j];
            let original_squared_distance = squared_distance(&ov, &ow);
            let ring = Ring::from_epsilon(v, self.epsilon, original_squared_distance);
            //each_ring_points(&ring, |p| {
            //    self.vertices[j] = p;
            //    self.visited[j] = true;
            //    self.dfs(j);
            //});
            self.visited[j] = false;
        }
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
                f(p);
            }
        }
    }
}
