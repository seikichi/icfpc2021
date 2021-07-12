use crate::common::*;
use geo::algorithm::contains::Contains;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::{Duration, Instant};

type Vector2d = geo::Coordinate<f64>;

static SEED: [u8; 32] = [
    0xfd, 0x00, 0xf1, 0x5c, 0xde, 0x01, 0x11, 0xc6, 0xc3, 0xea, 0xfb, 0xbf, 0xf3, 0xca, 0xd8, 0x32,
    0x6a, 0xe3, 0x07, 0x99, 0xc5, 0xe0, 0x52, 0xe4, 0xaa, 0x35, 0x07, 0x99, 0xe3, 0x2b, 0x9d, 0xc6,
];

pub fn solve(input: &Input, time_limit: Duration) -> Option<(Vec<Point>, f64)> {
    let n = input.figure.vertices.len();

    //let original_vertices = input.figure.vertices.clone();
    let out_edges = make_out_edges(&input.figure.edges, input.figure.vertices.len());
    //let holl_points = all_points_in_hole(&input.hole);

    //let mut vertices = input.figure.vertices.clone();
    //let mut visited = vec![false; n];

    let (bridges, tecomp) = decompose_by_bridges(&out_edges);
    let vertex2tecomp = make_vertex_to_tecomp_id(&tecomp, n);
    let tecomp_out_edges = make_tecomp_out_edges(&bridges, &tecomp, &vertex2tecomp);

    //eprintln!("bridges = {:?}", bridges);
    //eprintln!("tecomp = {:?}", tecomp);

    let solver = Solver {
        vertex_count: out_edges.len(),
        edge_count: input.figure.edges.len(),
        out_edges,
        bridges,
        tecomp,
        vertex2tecomp,
        tecomp_out_edges,
        epsilon: input.epsilon,
        original: input.figure.vertices.clone(),
        hole: input.hole.clone(),
        boundary_terminals: HashSet::from_iter(input.hole.exterior().points_iter().map(|p| (p.x() as i64, p.y() as i64))),
        time_limit: time_limit,
        start_at: Instant::now(),
    };

    let order = solver.reorder();
    eprintln!("reorder = {:?}", order);
    assert_eq!(order.len(), input.figure.edges.len());

    let possible_ranges = solver.calculate_possible_ranges(&order);

    //eprintln!("possible_ranges = {:?}", possible_ranges);

    solver.search(&order, &possible_ranges)
}

/*
#[derive(Debug, Clone)]
struct State {
    i: usize,
    dislike: f64,
    solution: Vec<Point>,
    determined: Vec<bool>,
}
*/

#[derive(Debug, Clone, Copy, PartialEq)]
struct PossibleRange {
    center_index: usize,
    radius: f64,
    free: bool,
}

struct Solver {
    vertex_count: usize,
    edge_count: usize,
    out_edges: Vec<Vec<usize>>,
    bridges: Vec<Edge>,
    tecomp: Vec<Vec<usize>>,
    vertex2tecomp: Vec<usize>,
    tecomp_out_edges: Vec<Vec<(usize, Edge)>>,
    epsilon: i64,
    original: Vec<Point>,
    hole: Polygon,
    boundary_terminals: HashSet<(i64, i64)>,
    time_limit: Duration,
    start_at: Instant,
}

impl Solver {
    // まず、edge を見ていく順番を求める
    fn reorder(&self) -> Vec<Edge> {
        let mut tecomp_visited = vec![false; self.tecomp.len()];
        //let mut vertex_visited = vec![false; self.vertex_count];
        let mut used_edges: HashSet<Edge> = HashSet::new();
        let mut order: Vec<Edge> = vec![];

        let mut start_vertex = 0;
        let mut best_oe = self.out_edges[0].len();
        for v in 0..self.vertex_count {
            let oe = self.out_edges[v].len();
            if oe > best_oe {
                start_vertex = v;
                best_oe = oe;
            }
        }

        let tecomp_id = self.vertex2tecomp[start_vertex];
        self.reorder_tecomps(
            tecomp_id,
            start_vertex,
            &mut tecomp_visited,
            &mut used_edges,
            &mut order,
        );
        order
    }

    fn reorder_tecomps(
        &self,
        tecomp_id: usize,
        start_vertex: usize,
        tecomp_visited: &mut [bool],
        used_edges: &mut HashSet<Edge>,
        order: &mut Vec<Edge>,
    ) {
        tecomp_visited[tecomp_id] = true;

        self.reorder_single_tecomp(tecomp_id, start_vertex, used_edges, order);

        for (tecomp_dst, bridge) in self.tecomp_out_edges[tecomp_id].iter() {
            if tecomp_visited[*tecomp_dst] {
                continue;
            }
            order.push(*bridge);
            self.reorder_tecomps(*tecomp_dst, bridge.w, tecomp_visited, used_edges, order);
        }
    }

    fn reorder_single_tecomp(
        &self,
        tecomp_id: usize,
        v: usize,
        used_edges: &mut HashSet<Edge>,
        order: &mut Vec<Edge>,
    ) {
        let mut out_edges = self.out_edges[v].clone();
        out_edges.sort_by_key(|w| self.out_edges[*w].len());

        for &w in out_edges.iter().rev() {
            let e = Edge::new(v, w);
            if used_edges.contains(&e) {
                continue;
            }
            if self.vertex2tecomp[w] != tecomp_id {
                continue;
            }
            order.push(e);
            used_edges.insert(e);
            used_edges.insert(Edge::new(w, v));
            self.reorder_single_tecomp(tecomp_id, w, used_edges, order);
        }
    }

    // order の各 edge に対して、dst が存在してよい範囲を計算する
    fn calculate_possible_ranges(&self, order: &[Edge]) -> Vec<PossibleRange> {
        let mut possible_ranges = vec![PossibleRange { center_index: 0, radius: 0.0, free: false }; self.edge_count];

        let mut determined = vec![false; self.vertex_count];
        determined[order[0].v] = true;

        let mut last = 0;

        let bridges: HashSet<Edge> = HashSet::from_iter(self.bridges.iter().copied());

        for i in 0..self.edge_count {
            let src = order[i].v;
            let dst = order[i].w;

            if bridges.contains(&Edge::new(src, dst)) || bridges.contains(&Edge::new(dst, src)) {
                possible_ranges[i] = PossibleRange { center_index: 0, radius: 0.0, free: true };

            } else if determined[dst] {
                let center_index = dst;
                let mut sum_len = 0.0;
                possible_ranges[i] = PossibleRange { center_index, radius: 0.0, free: false };
                for j in (last+1..=i).rev() {
                    let o_src = self.original[order[j].v];
                    let o_dst = self.original[order[j].w];
                    let sq_dist = squared_distance(&o_src, &o_dst);
                    let ring = Ring::from_epsilon(Point::new(0.0, 0.0), self.epsilon, sq_dist);
                    sum_len += ring.outer_radius;
                    possible_ranges[j-1] = PossibleRange { center_index, radius: sum_len, free: false };
                }
                last = i + 1;
            }

            determined[dst] = true;
        }
        possible_ranges
    }

    fn search(
        &self, order: &[Edge], possible_ranges: &[PossibleRange]
    ) -> Option<(Vec<Point>, f64)> {
        let mut rng = SmallRng::from_seed(SEED);
        let mut candidates: Vec<Point> = self.hole.exterior().points_iter().collect();

        let mut hole_points = all_points_in_hole(&self.hole);
        hole_points.shuffle(&mut rng);
        candidates.extend(hole_points.iter().take(20));

        let mut best_solution = None;
        let mut best_dislike = 1e20;

        for &pos in candidates.iter() {
            let mut solution = self.original.clone();
            let mut determined = vec![false; self.vertex_count];
            let v = order[0].v;

            solution[v] = pos;
            determined[v] = true;

            let mut n_iter = 0;

            if let Some((s, dislike)) = self.dfs(0, order, possible_ranges, &mut solution, &mut determined, &mut n_iter) {
                if dislike < best_dislike {
                    best_solution = Some(s);
                    best_dislike = dislike;
                }
            } else {
                eprintln!("not found... (n_iter = {}, start = {:?})", n_iter, pos);
            }

            // タイムリミットを超えていたらすぐに終了する
            if Instant::now() - self.start_at >= self.time_limit {
                eprintln!("time limit exceeded. return early.");
                return best_solution.map(|s| (s, best_dislike));
            }
        }

        best_solution.map(|s| (s, best_dislike))
    }

    fn dfs(
        &self,
        i: usize,
        order: &[Edge],
        possible_ranges: &[PossibleRange],
        solution: &mut Vec<Point>,
        determined: &mut [bool],
        n_iter: &mut i64,
    ) -> Option<(Vec<Point>, f64)> {
        if i == self.edge_count {
            let dislike = calculate_dislike(&solution, &self.hole);
            eprintln!("found!! dislike={}", dislike);
            return Some((solution.clone(), dislike));
        }

        // タイムリミット
        if *n_iter % 10000 == 0 {
            let elapsed = Instant::now() - self.start_at;
            if elapsed >= self.time_limit {
                return None;
            }
        }
        *n_iter += 1;
        // iteration の回数が多すぎるときは、初期点を選び方を変えたほうが良さそうなので
        // 探索を打ち切る
        if *n_iter > 1000000 {
            return None;
        }

        let src = order[i].v;
        let dst = order[i].w;

        if determined[dst] {
            // src も dst も確定している。
            // この辺が invalid だったらバックトラックしないといけない。
            let ok = is_allowed_distance(
                &solution[src],
                &solution[dst],
                &self.original[src],
                &self.original[dst],
                self.epsilon,
                false,
            ) && does_line_fit_in_hole(&solution[src], &solution[dst], &self.hole);
            if ok {
                return self.dfs(i+1, order, possible_ranges, solution, determined, n_iter);
            } else {
                return None;
            }
        }

        // 頂点 dst の位置を決める
        determined[dst] = true;

        let p0 = solution[src];
        let op0 = self.original[src];
        let op1 = self.original[dst];
        let ring = Ring::from_epsilon(p0, self.epsilon, squared_distance(&op0, &op1));

        let mut candidates = vec![];
        
        each_ring_points(&ring, |p| {
            let PossibleRange { center_index, radius, free } =  possible_ranges[i];
            let ok = {
                if free {
                    true
                } else {
                    distance(&solution[center_index], &p) <= radius
                }
            };
            if ok {
                candidates.push(p);
            }
        });

        // candidates をよさげな順番に並べたい
        candidates.sort_by_key(|p1| {
            // 端点が候補にあるならそれを優先的に選びたい
            if self.boundary_terminals.contains(&(p1.x() as i64, p1.y() as i64)) {
                return -100000000;
            }

            // すでに決まっているエッジの方向とはできるだけ違う方向に行きたい
            let v1 = p1.0 - p0.0;
            let mut sim = 0.0;
            for &w in self.out_edges[src].iter() {
                if determined[w] {
                    let p2 = solution[w];
                    let v2 = p2.0 - p0.0;
                    sim += cosine_sim(v1, v2);
                }
            }
            (sim * 100000.0) as i32
        });

        // 間引く
        let max_candidates = if self.vertex_count > 30 { 4 } else { 20 };
        if candidates.len() > max_candidates {
            candidates = candidates
                .iter()
                .step_by(candidates.len() / max_candidates)
                .copied()
                .collect();
        }

        for p1 in candidates.iter() {
            if does_line_fit_in_hole(&p0, &p1, &self.hole) {
                solution[dst] = *p1;
                if let Some(ret) = self.dfs(i + 1, order, possible_ranges, solution, determined, n_iter) {
                    determined[dst] = false;
                    return Some(ret);
                }
            }
        }

        determined[dst] = false;
        None
    }

    /*
    fn search(
        &self, order: &[Edge], possible_ranges: &[PossibleRange],
    ) -> Option<(Vec<Point>, f64)> {
        let mut queue: Vec<Vec<State>> = vec![Vec::new(); self.edge_count + 1];
        let mut rng = SmallRng::from_seed(SEED);
        let mut candidates: Vec<Point> = self.hole.exterior().points_iter().collect();

        let mut hole_points = all_points_in_hole(&self.hole);
        hole_points.shuffle(&mut rng);
        candidates.extend(hole_points.iter().take(20));

        for &pos in candidates.iter() {
            let mut solution = self.original.clone();
            let mut determined = vec![false; self.vertex_count];
            let v = order[0].v;

            solution[v] = pos;
            determined[v] = true;

            let dislike = calculate_dislike_determined_only(&solution, &self.hole, &determined);

            queue[0].push(State {
                i: 0,
                dislike,
                solution: solution,
                determined: determined,
            });
        }

        let mut best_solution = None;
        let mut best_dislike = 1e20;

        let max_iteration = 10000;
        for _iter in 0..max_iteration {
            for i in 0..self.edge_count {
                if queue[i].len() == 0 {
                    continue;
                }
                queue[i].sort_by(|a, b| a.dislike.partial_cmp(&b.dislike).unwrap());
                // queue[i].shuffle(&mut rng);
                queue[i].truncate(max_iteration);
                queue[i].reverse();

                let state = queue[i].pop().unwrap();
                if let Some((solution, dislike)) =
                    self.generate_next_states(state, order, possible_ranges, &mut queue[i + 1])
                {
                    if dislike < best_dislike {
                        best_solution = Some(solution);
                        best_dislike = dislike;
                    }
                }
            }
        }

        best_solution.map(|s| (s, best_dislike))
    }

    fn generate_next_states(
        &self,
        state: State,
        order: &[Edge],
        possible_ranges: &[PossibleRange],
        queue: &mut Vec<State>,
    ) -> Option<(Vec<Point>, f64)> {
        let State {
            i,
            dislike,
            mut solution,
            mut determined,
        } = state;

        if i == self.edge_count {
            return Some((solution, dislike));
        }

        let src = order[i].v;
        let dst = order[i].w;

        if determined[dst] {
            // src も dst も確定している。
            // この辺が invalid だったらバックトラックしないといけない。
            let ok = is_allowed_distance(
                &solution[src],
                &solution[dst],
                &self.original[src],
                &self.original[dst],
                self.epsilon,
                false,
            ) && does_line_fit_in_hole(&solution[src], &solution[dst], &self.hole);
            if ok {
                queue.push(State {
                    i: i + 1,
                    dislike,
                    solution: solution,
                    determined: determined,
                });
                return None;
            } else {
                return None;
            }
        }

        // 頂点 dst の位置を決める
        determined[dst] = true;

        let p0 = solution[src];
        let op0 = self.original[src];
        let op1 = self.original[dst];
        let ring = Ring::from_epsilon(p0, self.epsilon, squared_distance(&op0, &op1));

        let mut candidates = vec![];
        
        each_ring_points(&ring, |p| {
            let PossibleRange { center_index, radius } =  possible_ranges[i];
            let dist = distance(&solution[center_index], &p);
            if dist <= radius {
                candidates.push(p);
            }
        });

        // candidates をよさげな順番に並べたい
        candidates.sort_by_key(|p1| {
            // すでに決まっているエッジの方向とはできるだけ違う方向に行きたい
            let v1 = p1.0 - p0.0;
            let mut sim = 0.0;
            for &w in self.out_edges[src].iter() {
                if determined[w] {
                    let p2 = solution[w];
                    let v2 = p2.0 - p0.0;
                    sim += cosine_sim(v1, v2);
                }
            }
            (sim * 100000.0) as i32
        });

        // 間引く
        let max_candidates = 20;
        if candidates.len() > max_candidates {
            candidates = candidates
                .iter()
                .step_by(candidates.len() / max_candidates)
                .copied()
                .collect();
        }

        for p1 in candidates.iter() {
            if does_line_fit_in_hole(&p0, &p1, &self.hole) {
                solution[dst] = *p1;
                let new_dislike =
                    calculate_dislike_determined_only(&solution, &self.hole, &determined);
                queue.push(State {
                    i: i + 1,
                    dislike: new_dislike,
                    solution: solution.clone(),
                    determined: determined.clone(),
                });
            }
        }

        None
    }
    */
}

#[allow(dead_code)]
fn calculate_dislike_determined_only(
    vertices: &[Point],
    hole: &Polygon,
    determined: &[bool],
) -> f64 {
    let mut s = 0.0;
    for h in hole.exterior().points_iter().skip(1) {
        s += vertices
            .iter()
            .enumerate()
            .filter(|(i, _)| determined[*i])
            .map(|(_, v)| squared_distance(v, &h))
            .fold(0.0 / 0.0, |m, x| x.min(m));
    }
    s
}

fn dot(v1: Vector2d, v2: Vector2d) -> f64 {
    v1.x * v2.x + v1.y * v2.y
}

fn unit_vector(v: Vector2d) -> Vector2d {
    let l = dot(v, v).sqrt();
    v / l
}

fn cosine_sim(mut v1: Vector2d, mut v2: Vector2d) -> f64 {
    v1 = unit_vector(v1);
    v2 = unit_vector(v2);
    dot(v1, v2)
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

fn all_points_in_hole(hole: &Polygon) -> Vec<Point> {
    let mut ps = vec![];
    each_point_in_hole(hole, |p| {
        ps.push(p);
    });
    return ps;
}

fn make_vertex_to_tecomp_id(tecomp: &[Vec<usize>], n: usize) -> Vec<usize> {
    let mut vertex2tecomp = vec![0; n];
    for i in 0..tecomp.len() {
        for &v in tecomp[i].iter() {
            vertex2tecomp[v] = i;
        }
    }
    vertex2tecomp
}

fn make_tecomp_out_edges(
    bridges: &[Edge],
    tecomp: &[Vec<usize>],
    vertex2tecomp: &[usize],
) -> Vec<Vec<(usize, Edge)>> {
    let mut out_edges = vec![vec![]; tecomp.len()];
    for bridge in bridges.iter() {
        let rev_bridge = Edge::new(bridge.w, bridge.v);
        let src = vertex2tecomp[bridge.v];
        let dst = vertex2tecomp[bridge.w];
        out_edges[src].push((dst, *bridge));
        out_edges[dst].push((src, rev_bridge));
    }
    out_edges
}
