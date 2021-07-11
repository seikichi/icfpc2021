use crate::common::*;
use std::collections::HashSet;
use geo::algorithm::contains::Contains;

pub fn solve(input: &Input) -> Option<(Vec<Point>, f64)> {
    let n = input.figure.vertices.len();

    //let original_vertices = input.figure.vertices.clone();
    let out_edges = make_out_edges(&input.figure.edges, input.figure.vertices.len());
    //let holl_points = all_points_in_hole(&input.hole);

    //let mut vertices = input.figure.vertices.clone();
    //let mut visited = vec![false; n];

    let (bridges, tecomp) = decompose_by_bridges(&out_edges);
    let vertex2tecomp = make_vertex_to_tecomp_id(&tecomp, n);
    let tecomp_out_edges = make_tecomp_out_edges(&bridges, &tecomp, &vertex2tecomp);

    eprintln!("bridges = {:?}", bridges);
    eprintln!("tecomp = {:?}", tecomp);

    let solver = Solver {
        vertex_count: out_edges.len(),
        out_edges, bridges, tecomp, vertex2tecomp, tecomp_out_edges,
        epsilon: input.epsilon,
        original: input.figure.vertices.clone(),
        hole: input.hole.clone(),
    };

    let order = solver.reorder();
    eprintln!("reorder = {:?}", order);

    assert_eq!(order.len(), input.figure.edges.len());

    let mut solution = input.figure.vertices.clone();
    let mut determined = vec![false; n];

    for pos in all_points_in_hole(&input.hole) {
        let v = order[0].v;
        solution[v] = pos;
        determined[v] = true;
        if solver.dfs(0, &order, &mut solution, &mut determined) {
            let dislike = calculate_dislike(&solution, &input.hole);
            return Some((solution, dislike));
        }
    }

    None
}

struct Solver {
    vertex_count: usize,
    out_edges: Vec<Vec<usize>>,
    bridges: Vec<Edge>,
    tecomp: Vec<Vec<usize>>,
    vertex2tecomp: Vec<usize>,
    tecomp_out_edges: Vec<Vec<(usize, Edge)>>,
    epsilon: i64,
    original: Vec<Point>,
    hole: Polygon,
}

impl Solver {
    // まず、edge を見ていく順番を求める
    fn reorder(&self) -> Vec<Edge> {
        let mut tecomp_visited = vec![false; self.tecomp.len()];
        //let mut vertex_visited = vec![false; self.vertex_count];
        let mut used_edges: HashSet<Edge> = HashSet::new();
        let mut order: Vec<Edge> = vec![];
        let tecomp_id = self.vertex2tecomp[0];
        self.reorder_tecomps(
            tecomp_id, 0,
            &mut tecomp_visited, &mut used_edges,
            &mut order);
        order
    }

    fn reorder_tecomps(
        &self, tecomp_id: usize, start_vertex: usize,
        tecomp_visited: &mut [bool], used_edges: &mut HashSet<Edge>,
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
        &self, tecomp_id: usize, v: usize,
        used_edges: &mut HashSet<Edge>,
        order: &mut Vec<Edge>,
    ) {
        for &w in self.out_edges[v].iter() {
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

    fn dfs(
        &self, i: usize,
        order: &[Edge],
        solution: &mut [Point], determined: &mut [bool],
    ) -> bool {
        if i == self.vertex_count {
            return true;
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
                return self.dfs(i + 1, order, solution, determined);
            } else {
                return false;
            }
        }

        // 頂点 dst の位置を決める
        determined[dst] = true;

        let p0 = solution[src];
        let op0 = self.original[src];
        let op1 = self.original[dst];
        let ring = Ring::from_epsilon(p0, self.epsilon, squared_distance(&op0, &op1));

        // TODO: ヒューリスティックを入れる
        for p1 in ring_points(&ring).iter() {
            if does_line_fit_in_hole(&p0, &p1, &self.hole) {
                solution[dst] = *p1;
                if self.dfs(i + 1, order, solution, determined) {
                    determined[dst] = false;
                    return true;
                }
            }
        }

        determined[dst] = false;
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

fn all_points_in_hole(hole: &Polygon) -> Vec<Point> {
    let mut ps = vec![];
    each_point_in_hole(hole, |p| {
        ps.push(p);
    });
    return ps;
}

// 橋でグラフを分割する。(橋の集合, 各連結成分の頂点集合) が返される。
// from http://www.prefield.com/algorithm/graph/bridge.html
fn decompose_by_bridges(out_edges: &[Vec<usize>]) -> (Vec<Edge>, Vec<Vec<usize>>) {

    fn visit(out_edges: &[Vec<usize>], v: usize, u: usize,
        brdg: &mut Vec<Edge>, tecomp: &mut Vec<Vec<usize>>,
        roots: &mut Vec<usize>, s: &mut Vec<usize>, in_s: &mut Vec<bool>,
        num: &mut Vec<usize>, time: &mut usize)
    {
        *time += 1;
        num[v] = *time;

        s.push(v);
        in_s[v] = true;

        roots.push(v);

        for &w in out_edges[v].iter() {
            if num[w] == 0 {
                visit(out_edges, w, v, brdg, tecomp, roots, s, in_s, num, time);
            } else if u != w && in_s[w] {
                while num[*roots.last().unwrap()] > num[w] {
                    roots.pop();
                }
            }
        }

        if v == *roots.last().unwrap() {
            brdg.push(Edge { v: u, w: v });
            tecomp.push(vec![]);

            loop {
                let w = *s.last().unwrap();
                s.pop();
                in_s[w] = false;
                tecomp.last_mut().unwrap().push(w);
                if v == w {
                    break
                }
            }

            roots.pop();
        }
    }

    let n = out_edges.len();
    let mut num = vec![0; n];
    let mut in_s = vec![false; n];
    let mut roots: Vec<usize> = vec![]; // used as stack
    let mut s: Vec<usize> = vec![]; // used as stack
    let mut time = 0;
    let mut brdg: Vec<Edge> = vec![];
    let mut tecomp: Vec<Vec<usize>> = vec![];
    for u in 0..n {
        if num[u] == 0 {
            visit(out_edges, u, n, &mut brdg, &mut tecomp, &mut roots, &mut s, &mut in_s, &mut num, &mut time);
            brdg.pop();
        }
    }

    (brdg, tecomp)
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

fn make_tecomp_out_edges(bridges: &[Edge], tecomp: &[Vec<usize>], vertex2tecomp: &[usize]) -> Vec<Vec<(usize, Edge)>> {
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