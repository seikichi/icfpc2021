use crate::common::*;
use geo::algorithm::contains::Contains;

struct Solver {
    original_vertices: Vec<Point>, // readonly
    out_edges: Vec<Vec<usize>>,    // readonly
    epsilon: i64,                  // readonly
    hole: Polygon,                 // readonly
}

pub fn solve(input: &Input) -> Option<(Vec<Point>, f64)> {
    let solver = Solver {
        original_vertices: input.figure.vertices.clone(),
        out_edges: make_out_edges(&input.figure.edges, input.figure.vertices.len()),
        epsilon: input.epsilon,
        hole: input.hole.clone(),
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
        &self,
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

        let ret = each_point_in_hole(&self.hole, |p| {
            vertices[src] = p;

            // verify
            let mut ok = true;
            for &dst in self.out_edges[src].iter() {
                if !visited[dst] {
                    continue;
                }
                // check
                if !is_allowed_distance(
                    &vertices[src],
                    &vertices[dst],
                    &self.original_vertices[src],
                    &self.original_vertices[dst],
                    self.epsilon,
                ) {
                    ok = false;
                    break;
                }
            }

            if ok {
                if self.naive_dfs(i + 1, vertices, visited, order) {
                    return true;
                }
            }

            false
        });

        visited[src] = false;
        ret
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
