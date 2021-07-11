pub type Point = geo::Point<f64>;
pub type Polygon = geo::Polygon<f64>;
pub type Line = geo::Line<f64>;
use geo::algorithm::contains::Contains;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Edge {
    pub v: usize,
    pub w: usize,
}

impl Edge {
    pub fn new(v: usize, w: usize) -> Edge {
        Edge { v, w }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BonusType {
    Globalist,
    BreakALeg,
    WallHack,
}
impl BonusType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "GLOBALIST" => BonusType::Globalist,
            "BREAK_A_LEG" => BonusType::BreakALeg,
            "WALLHACK" => BonusType::WallHack,
            _ => panic!("Invalid Bonus String {}", s),
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            BonusType::Globalist => "GLOBALIST".to_string(),
            BonusType::BreakALeg => "BREAK_A_LEG".to_string(),
            BonusType::WallHack => "WALLHACK".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Figure {
    pub edges: Vec<Edge>,
    pub vertices: Vec<Point>,
}

#[derive(Debug, Clone)]
pub struct Bonus {
    pub position: Point,
    pub bonus: String,
    pub problem: i64,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub hole: Polygon,
    pub figure: Figure,
    pub epsilon: i64,
    pub bonuses: Vec<Bonus>,
}

pub fn squared_distance(a: &Point, b: &Point) -> f64 {
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();
    dx * dx + dy * dy
}

pub fn distance(a: &Point, b: &Point) -> f64 {
    squared_distance(a, b).sqrt()
}

pub fn calculate_dislike(vertices: &[Point], hole: &Polygon) -> f64 {
    let mut s = 0.0;
    for h in hole.exterior().points_iter().skip(1) {
        s += vertices
            .iter()
            .map(|v| squared_distance(v, &h))
            .fold(0.0 / 0.0, |m, x| x.min(m));
    }
    s
}

pub fn does_line_fit_in_hole(p1: &Point, p2: &Point, hole: &Polygon) -> bool {
    let line = Line::new(*p1, *p2);
    if !hole.contains(&line) {
        if !hole.exterior().contains(&line) {
            return false;
        }
    }
    true
}

pub fn calc_global_allowed_distance(vertices: &Vec<Point>, figure: &Figure) -> f64 {
    let mut sum = 0.0;
    for e in figure.edges.iter() {
        let p1 = vertices[e.v];
        let p2 = vertices[e.w];
        let original_p1 = figure.vertices[e.v];
        let original_p2 = figure.vertices[e.w];
        let sd = squared_distance(&p1, &p2);
        let original_sd = squared_distance(&original_p1, &original_p2);
        sum += (1.0 - sd / original_sd).abs()
    }
    return sum;
}
pub fn does_global_allowed_distance(vertices: &Vec<Point>, figure: &Figure, epsilon: i64) -> bool {
    let eps = 1e-7;
    let sum = calc_global_allowed_distance(vertices, figure);
    let ret = sum + eps < figure.edges.len() as f64 * epsilon as f64 / 1000000.0;
    ret
}

pub fn does_figure_fit_in_hole(figure: &Figure, hole: &Polygon, wall_hack: bool) -> bool {
    let mut wall_hack_point: Option<Point> = None;
    if wall_hack {
        for p in figure.vertices.iter() {
            if !hole.contains(p) && !hole.exterior().contains(p) {
                wall_hack_point = Some(*p);
                break;
            }
        }
    }
    for e in figure.edges.iter() {
        let p1 = figure.vertices[e.v];
        let p2 = figure.vertices[e.w];
        if let Some(wp) = wall_hack_point {
            if wp == p1 || wp == p2 {
                continue;
            }
        }
        let line = Line::new(p1, p2);
        if !hole.contains(&line) {
            if !hole.exterior().contains(&line) {
                return false;
            }
        }
    }
    true
}

pub fn does_valid_pose(
    vertices: &Vec<Point>,
    figure: &Figure,
    hole: &Polygon,
    epsilon: i64,
    used_bonus_types: &Vec<BonusType>,
    break_leg: Option<Edge>,
) -> bool {
    let use_globalist = used_bonus_types.iter().any(|b| *b == BonusType::Globalist);
    let use_wall_hack = used_bonus_types.iter().any(|b| *b == BonusType::WallHack);
    let use_break_leg = used_bonus_types.iter().any(|b| *b == BonusType::BreakALeg);
    assert!(!(use_globalist && use_break_leg)); // 両方は同時に使えない
    if use_break_leg {
        assert!(break_leg.is_some());
    } else {
        assert!(break_leg.is_none());
    }

    if use_globalist {
        if !does_global_allowed_distance(&vertices, &figure, epsilon) {
            return false;
        }
    } else {
        // Normal Edge
        for e in figure.edges.iter() {
            if let Some(break_leg_edge) = break_leg {
                if *e == break_leg_edge {
                    continue;
                }
            }
            let p1 = vertices[e.v];
            let p2 = vertices[e.w];
            let original_p1 = figure.vertices[e.v];
            let original_p2 = figure.vertices[e.w];
            if !is_allowed_distance(&p1, &p2, &original_p1, &original_p2, epsilon, false) {
                return false;
            }
        }

        // Break Leg Edge
        if let Some(break_leg_edge) = break_leg {
            let k = figure.vertices.len();
            let edges = [
                Edge::new(break_leg_edge.v, k),
                Edge::new(break_leg_edge.w, k),
            ];
            for e in edges.iter() {
                let p1 = vertices[e.v];
                let p2 = vertices[e.w];
                let original_p1 = figure.vertices[e.v];
                let original_p2 = figure.vertices[e.w];
                if !is_allowed_distance(&p1, &p2, &original_p1, &original_p2, epsilon, true) {
                    return false;
                }
            }
        }
    }
    let f = Figure {
        edges: figure.edges.clone(),
        vertices: vertices.clone(),
    };
    return does_figure_fit_in_hole(&f, &hole, use_wall_hack);
}

#[test]
pub fn test_does_valid_pose() {
    let mut ps1 = vec![];
    ps1.push(Point::new(34.0, 22.0));
    ps1.push(Point::new(10.0, 24.0));
    ps1.push(Point::new(11.0, 21.0));
    ps1.push(Point::new(23.0, 5.0));
    ps1.push(Point::new(0.0, 0.0));
    let input = crate::inout::parse_input(
        &r#"{"hole":[[23,0],[32,2],[24,6],[31,9],[36,12],[36,26],[29,18],[24,22],[21,27],[30,32],[18,34],[10,38],[12,30],[6,28],[0,32],[0,20],[8,22],[5,14],[1,6],[0,0],[6,0],[12,3],[17,0]],"epsilon":15010,"figure":{"edges":[[0,1],[0,2],[1,3],[2,4],[3,4]],"vertices":[[0,7],[0,31],[22,0],[22,38],[36,19]]}}"#,
    );
    assert!(!does_valid_pose(
        &ps1,
        &input.figure,
        &input.hole,
        input.epsilon
    ));
}

// #[test]
// fn test_contains() {
//     let l1 = Line::new(Point::new(0.0, 10.0), Point::new(20.0, 10.0));
//     // let l2 = Line::new(Point::new(13.0, 10.0), Point::new(20.0, 10.0));
//     let hole2 = Polygon::new(
//         geo::LineString::from(vec![
//             Point::new(0.0, 0.0),
//             Point::new(10.0, 0.0),
//             Point::new(10.0, 10.0),
//             Point::new(20.0, 10.0),
//             Point::new(20.0, 20.0),
//             Point::new(0.0, 20.0),
//             Point::new(0.0, 0.0),
//         ]),
//         vec![],
//     );
//     assert!(hole2.contains(&l1));
//     // bug?
//     // assert!(hole2.contains(&l2));
// }

#[test]
fn test_calculate_dislike() {
    let figure1 = Figure {
        edges: vec![],
        vertices: vec![Point::new(1.0, 1.0)],
    };
    let figure2 = Figure {
        edges: vec![],
        vertices: vec![
            Point::new(0.0, 0.0),
            Point::new(3.0, 0.0),
            Point::new(3.0, 3.0),
            Point::new(0.0, 3.0),
        ],
    };
    let figure3 = Figure {
        edges: vec![],
        vertices: vec![Point::new(0.0, 0.0), Point::new(3.0, 0.0)],
    };
    let hole1 = Polygon::new(
        geo::LineString::from(vec![
            Point::new(0.0, 0.0),
            Point::new(3.0, 0.0),
            Point::new(3.0, 3.0),
            Point::new(0.0, 3.0),
        ]),
        vec![],
    );
    assert!(calculate_dislike(&figure1.vertices, &hole1) == 20.0);
    assert!(calculate_dislike(&figure2.vertices, &hole1) == 0.0);
    assert!(calculate_dislike(&figure3.vertices, &hole1) == 18.0);
}

#[derive(Debug, Clone)]
pub struct Ring {
    center: Point,
    inner_radius: f64,
    outer_radius: f64,
}

const EPS: f64 = 1e-8;

impl Ring {
    #[allow(dead_code)]
    fn new(center: Point, inner_radius: f64, outer_radius: f64) -> Ring {
        assert!(inner_radius <= outer_radius);
        Ring {
            center,
            inner_radius,
            outer_radius,
        }
    }

    #[allow(dead_code)]
    pub fn from_epsilon(center: Point, epsilon: i64, original_squared_distance: f64) -> Ring {
        // |d'/d - 1| <= eps/1,000,000
        // -eps/1,000,000 <= d'/d - 1 <= eps/1,000,000
        // (1 - eps/1,000,000)*d <= d' <= (1 + eps/1,000,000)*d
        let sq_inner_radius =
            ((1.0 - epsilon as f64 / 1000000.0) * original_squared_distance).max(0.0);
        let sq_outer_radius = (1.0 + epsilon as f64 / 1000000.0) * original_squared_distance;
        let inner_radius = sq_inner_radius.sqrt() - EPS;
        let outer_radius = sq_outer_radius.sqrt() + EPS;
        Ring {
            center,
            inner_radius,
            outer_radius,
        }
    }
}

pub fn is_allowed_distance(
    p1: &Point,
    p2: &Point,
    original_p1: &Point,
    original_p2: &Point,
    epsilon: i64,
    break_leg: bool,
) -> bool {
    let mut sd = squared_distance(p1, p2);
    let original_sd = squared_distance(original_p1, original_p2);
    if break_leg {
        sd *= 4.0;
    }
    // |sd / original_sd - 1.0| <= epsilon / 1000000
    // -epsilon / 1,000,000 <= sd / original_sd - 1.0 <= epsilon / 1,000,000
    // 1.0 - eps / 1,000,000 <= sd / original_sd <= 1.0 + eps / 1,000,000
    // (1.0 - eps / 1,000,000) * original_sd <= sd <= (1.0 + eps / 1,000,000) * original_sd
    // (1,000,000 - eps) * original_sd <= sd * 1,000,000 <= (1,000,000 + eps) * original_sd
    let lo = (1000000.0 - epsilon as f64) * original_sd;
    let middle = sd * 1000000.0;
    let hi = (1000000.0 + epsilon as f64) * original_sd;
    lo <= middle && middle <= hi
}

pub fn calc_distance_ratio(
    p1: &Point,
    p2: &Point,
    original_p1: &Point,
    original_p2: &Point,
) -> f64 {
    let sd = squared_distance(&p1, &p2);
    let original_sd = squared_distance(&original_p1, &original_p2);
    return sd / original_sd - 1.0;
}

#[test]
fn test_is_allowed_distance() {
    let p1 = Point::new(10.0, 0.0);
    let p2 = Point::new(10.0, 10.0);
    let original_p1 = Point::new(0.0, 0.0);
    let original_p2 = Point::new(10.0, 0.0);
    assert!(is_allowed_distance(
        &p1,
        &p2,
        &original_p1,
        &original_p2,
        0,
        false
    ));
}

pub fn pow2(x: f64) -> f64 {
    x * x
}

pub fn each_ring_points(ring: &Ring, mut f: impl FnMut(Point)) {
    let y_min = (ring.center.y() - ring.outer_radius).ceil() as i64;
    let y_max = (ring.center.y() + ring.outer_radius).floor() as i64;
    let iy_min = (ring.center.y() - ring.inner_radius).floor() as i64;
    let iy_max = (ring.center.y() + ring.inner_radius).ceil() as i64;
    for y in y_min..=y_max {
        // (x - cx)^2 + (y - cy)^2 = r^2
        // x = cx +- sqrt(r^2 - (y - cy)^2)
        let s = (pow2(ring.outer_radius) - pow2(y as f64 - ring.center.y())).sqrt();
        let x_min = (ring.center.x() - s).ceil() as i64;
        let x_max = (ring.center.x() + s).floor() as i64;
        if iy_min < y && y < iy_max {
            let is = (pow2(ring.inner_radius) - pow2(y as f64 - ring.center.y())).sqrt();
            let ix_min = (ring.center.x() - is).floor() as i64;
            let ix_max = (ring.center.x() + is).ceil() as i64;
            for x in x_min..=ix_min {
                f(Point::new(x as f64, y as f64));
            }
            for x in ix_max..=x_max {
                f(Point::new(x as f64, y as f64));
            }
        } else {
            for x in x_min..=x_max {
                f(Point::new(x as f64, y as f64));
            }
        }
    }
}

#[allow(dead_code)]
pub fn ring_points(ring: &Ring) -> Vec<Point> {
    let mut ps = vec![];
    each_ring_points(ring, |p| ps.push(p));
    ps
}

#[test]
fn test_each_ring_points() {
    let mut points1: Vec<Point> = vec![];
    let f1 = |p| {
        points1.push(p);
    };
    let ring1 = Ring::new(Point::new(0.0, 0.0), 1.0, 1.0);
    each_ring_points(&ring1, f1);
    // println!("{:?}", points1);
    assert!(points1.len() == 4);
    assert!(points1[0] == Point::new(0.0, -1.0));
    assert!(points1[1] == Point::new(-1.0, 0.0));
    assert!(points1[2] == Point::new(1.0, 0.0));
    assert!(points1[3] == Point::new(0.0, 1.0));

    let mut points2: Vec<Point> = vec![];
    let f2 = |p| {
        points2.push(p);
    };
    let ring2 = Ring::new(Point::new(0.0, 0.0), 2.0, 2.0);
    each_ring_points(&ring2, f2);
    // println!("{:?}", points2);
    assert!(points2.len() == 4);
    assert!(points2[0] == Point::new(0.0, -2.0));
    assert!(points2[1] == Point::new(-2.0, 0.0));
    assert!(points2[2] == Point::new(2.0, 0.0));
    assert!(points2[3] == Point::new(0.0, 2.0));

    let mut points3: Vec<Point> = vec![];
    let f3 = |p| {
        points3.push(p);
    };
    let ring3 = Ring::new(Point::new(0.0, 0.0), 1.0, 2.0);
    each_ring_points(&ring3, f3);
    // println!("{:?}", points3);
    assert!(points3.len() == 12);
    assert!(points3[0] == Point::new(0.0, -2.0));
    assert!(points3[1] == Point::new(-1.0, -1.0));
    assert!(points3[2] == Point::new(0.0, -1.0));
    assert!(points3[3] == Point::new(1.0, -1.0));
    assert!(points3[4] == Point::new(-2.0, 0.0));
    assert!(points3[5] == Point::new(-1.0, 0.0));
    assert!(points3[6] == Point::new(1.0, 0.0));
    assert!(points3[7] == Point::new(2.0, 0.0));
    assert!(points3[8] == Point::new(-1.0, 1.0));
    assert!(points3[9] == Point::new(0.0, 1.0));
    assert!(points3[10] == Point::new(1.0, 1.0));
    assert!(points3[11] == Point::new(0.0, 2.0));

    let mut points4: Vec<Point> = vec![];
    let f4 = |p| {
        points4.push(p);
    };
    let ring4 = Ring::new(Point::new(1.0, 1.0), 1.0, 1.0);
    each_ring_points(&ring4, f4);
    // println!("{:?}", points4);
    assert!(points4.len() == 4);
    assert!(points4[0] == Point::new(1.0, 0.0));
    assert!(points4[1] == Point::new(0.0, 1.0));
    assert!(points4[2] == Point::new(2.0, 1.0));
    assert!(points4[3] == Point::new(1.0, 2.0));

    let mut points5: Vec<Point> = vec![];
    let f5 = |p| {
        points5.push(p);
    };
    let ring5 = Ring::new(Point::new(0.0, 0.0), 1.1, 1.3);
    each_ring_points(&ring5, f5);
    // println!("{:?}", points5);
    assert!(points5.len() == 0);

    let mut points6: Vec<Point> = vec![];
    let f6 = |p| {
        points6.push(p);
    };
    let ring6 = Ring::new(Point::new(0.0, 0.0), 0.0, 0.0);
    each_ring_points(&ring6, f6);
    // println!("{:?}", points6);
    assert!(points6.len() == 1);
    assert!(points6[0] == Point::new(0.0, 0.0));

    let mut points7: Vec<Point> = vec![];
    let f7 = |p| {
        points7.push(p);
    };
    let ring7 = Ring::new(Point::new(0.0, 0.0), 0.0, 1.0);
    each_ring_points(&ring7, f7);
    // println!("{:?}", points6);
    assert!(points7.len() == 5);
    assert!(points7[0] == Point::new(0.0, -1.0));
    assert!(points7[1] == Point::new(-1.0, 0.0));
    assert!(points7[2] == Point::new(0.0, 0.0));
    assert!(points7[3] == Point::new(1.0, 0.0));
    assert!(points7[4] == Point::new(0.0, 1.0));
}

#[test]
fn test_ring_points() {
    let p1 = Point::new(62.0, 43.0);
    let p2 = Point::new(61.0, 52.0);
    let original_squared_distance = 100.0;
    let ring = Ring::from_epsilon(p2, 180000, original_squared_distance);

    assert!(ring_points(&ring).contains(&p1));
}

pub fn make_out_edges(edges: &[Edge], n_vertices: usize) -> Vec<Vec<usize>> {
    let mut out_edges = vec![vec![]; n_vertices];
    for e in edges.iter() {
        out_edges[e.v].push(e.w);
        out_edges[e.w].push(e.v);
    }
    out_edges
}

pub fn make_determined_order(out_edges: &Vec<Vec<usize>>, start: Option<usize>) -> Vec<usize> {
    let n = out_edges.len();
    let mut order = vec![0; n];
    let mut determined = vec![false; n];
    let mut start_index = 0;
    if let Some(s) = start {
        order[0] = s;
        determined[s] = true;
        start_index = 1;
    }
    for i in start_index..n {
        let mut best = (0, 0, 0);
        for j in 0..n {
            if determined[j] {
                continue;
            }
            let mut jisu = 0;
            for &k in out_edges[j].iter() {
                if determined[k] {
                    jisu += 1;
                }
            }
            let score = (jisu, out_edges[j].len(), j);
            if best < score {
                best = score;
            }
        }
        order[i] = best.2;
        determined[order[i]] = true;
    }
    order
}

pub fn fix_allowed_distance_violation(
    start_point_index: usize,
    solution: &Vec<Point>,
    input: &Input,
) -> Option<Vec<Point>> {
    let mut solution = solution.clone();
    let n = input.figure.vertices.len();
    let out_edges = make_out_edges(&input.figure.edges, n);
    let order = make_determined_order(&out_edges, Some(start_point_index));
    let mut determined = vec![false; n];
    determined[start_point_index] = true;
    for index in 1..n {
        let from = order[index];
        determined[from] = true;
        let mut p = solution[from];
        for _iteration in 0..3 {
            for &to in out_edges[from].iter() {
                if !determined[to] {
                    continue;
                }
                let d1 = calc_distance_ratio(
                    &solution[to],
                    &p,
                    &input.figure.vertices[to],
                    &input.figure.vertices[from],
                );
                if d1.abs() + 1e-6 <= input.epsilon as f64 / 1000000.0 {
                    continue;
                }
                // 距離が条件を満たしてない場合は満たすように移動させる
                let distance_ratio = squared_distance(&solution[to], &p).sqrt()
                    / squared_distance(&input.figure.vertices[to], &input.figure.vertices[from])
                        .sqrt()
                    - 1.0;
                let vect = (solution[to] - solution[from])
                    * distance_ratio
                    * (input.epsilon as f64 / 1000000.0).sqrt();
                // eprintln!(
                //     "{} {} {} {:?}",
                //     squared_distance(&solution[to], &p).sqrt(),
                //     squared_distance(&input.figure.vertices[to], &input.figure.vertices[from])
                //         .sqrt(),
                //     distance_ratio,
                //     vect
                // );
                p = p + vect;
            }
        }
        let mut ok = false;
        // マンハッタン距離r以内の全点を試す
        'outer_loop: for r in 0i64..5i64 {
            for dx in -r..=r {
                let mut dys = vec![r - dx.abs(), -r + dx.abs()];
                dys.dedup();
                for &dy in dys.iter() {
                    let candidate_p = Point::new(
                        (p.x() + dx as f64 + 0.5).floor(),
                        (p.y() + dy as f64 + 0.5).floor(),
                    );
                    if is_allowed_distance_point_move(
                        from,
                        &candidate_p,
                        &solution,
                        &input.figure.vertices,
                        &out_edges,
                        &input.hole,
                        input.epsilon,
                        &determined,
                    ) {
                        // if dx != 0 || dy != 0 {
                        //     eprintln!("move: {} {:?}", from, candidate_p - solution[from]);
                        // }
                        solution[from] = candidate_p;
                        ok = true;
                        break 'outer_loop;
                    }
                }
            }
        }
        if !ok {
            return None;
        }
    }

    if !does_valid_pose(
        &solution,
        &input.figure,
        &input.hole,
        input.epsilon,
        &vec![],
        None,
    ) {
        return None;
    }

    return Some(solution);
}

fn is_allowed_distance_point_move(
    index: usize,
    p: &Point,
    solution: &[Point],
    original_vertices: &[Point],
    out_edges: &[Vec<usize>],
    hole: &Polygon,
    epsilon: i64,
    determined: &[bool],
) -> bool {
    let ok1 = out_edges[index].iter().all(|&dst| {
        !determined[dst]
            || is_allowed_distance(
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
        .all(|&dst| !determined[dst] || does_line_fit_in_hole(&p, &solution[dst], hole));
    if !ok2 {
        return false;
    }
    return true;
}
