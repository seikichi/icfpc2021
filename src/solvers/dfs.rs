use crate::common::*;
use geo::algorithm::contains::Contains;

struct Solver {
    original_vertices: Vec<Point>, // readonly
    edges: Vec<Edge>,              // readonly
    out_edges: Vec<Vec<usize>>,    // readonly
    epsilon: i64,                  // readonly
    hole: Polygon,                 // readonly
}

pub fn solve(input: &Input) -> Option<(Vec<Point>, f64)> {
    let solver = Solver {
        original_vertices: input.figure.vertices.clone(),
        edges: input.figure.edges.clone(),
        out_edges: make_out_edges(&input.figure.edges, input.figure.vertices.len()),
        epsilon: input.epsilon,
        hole: input.hole.clone(),
    };
    let mut vertices = input.figure.vertices.clone();
    //let mut visited = vec![false; input.figure.vertices.len()];

    if solver.naive_dfs(0, &mut vertices) {
        Some((vertices, 0.0))
    } else {
        None
    }
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

    fn naive_dfs(&self, i: usize, vertices: &mut [Point]) -> bool {
        if i == self.original_vertices.len() {
            return true;
        }

        each_point_in_hole(&self.hole, |p| {
            vertices[i] = p;

            // verify
            let mut ok = true;
            for &w in self.out_edges[i].iter() {
                if w < i {
                    // check
                    if !is_allowed_distance(
                        &vertices[i],
                        &vertices[w],
                        &self.original_vertices[i],
                        &self.original_vertices[w],
                        self.epsilon,
                    ) {
                        ok = false;
                        break;
                    }
                }
            }

            if ok {
                if self.naive_dfs(i + 1, vertices) {
                    return true;
                }
            }

            false
        })
    }
}

fn each_point_in_hole(hole: &Polygon, mut f: impl FnMut(Point) -> bool) -> bool {
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
                if f(p) {
                    return true;
                }
            }
        }
    }
    false
}
