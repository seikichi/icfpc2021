use crate::common::*;
//use geo::algorithm::contains::Contains;

pub fn solve(input: &Input) -> Option<(Vec<Point>, f64)> {
    //let n = input.figure.vertices.len();

    //let original_vertices = input.figure.vertices.clone();
    let out_edges = make_out_edges(&input.figure.edges, input.figure.vertices.len());
    //let holl_points = all_points_in_hole(&input.hole);

    //let mut vertices = input.figure.vertices.clone();
    //let mut visited = vec![false; n];

    let (bridges, tecomp) = decompose_by_bridges(&out_edges);

    eprintln!("bridges = {:?}", bridges);
    eprintln!("tecomp = {:?}", tecomp);

    None
}

/*
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
*/

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
