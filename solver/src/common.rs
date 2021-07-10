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

#[derive(Debug, Clone)]
pub struct Figure {
    pub edges: Vec<Edge>,
    pub vertices: Vec<Point>,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub hole: Polygon,
    pub figure: Figure,
    pub epsilon: i64,
}

pub fn squared_distance(a: &Point, b: &Point) -> f64 {
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();
    dx * dx + dy * dy
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

pub fn does_figure_fit_in_hole(figure: &Figure, hole: &Polygon) -> bool {
    for e in figure.edges.iter() {
        let p1 = figure.vertices[e.v];
        let p2 = figure.vertices[e.w];
        let line = Line::new(p1, p2);
        if !hole.contains(&line) {
            if !hole.exterior().contains(&line) {
                return false;
            }
        }
    }
    true
}

pub fn does_pose_fit_in_hole(vertices: &Vec<Point>, figure: &Figure, hole: &Polygon) -> bool {
    let f = Figure {
        edges: figure.edges.clone(),
        vertices: vertices.clone(),
    };
    return does_figure_fit_in_hole(&f, &hole);
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
        let inner_radius = sq_inner_radius.sqrt();
        let outer_radius = sq_outer_radius.sqrt();
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
) -> bool {
    let sd = squared_distance(p1, p2);
    let original_sd = squared_distance(original_p1, original_p2);
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

#[test]
fn test_is_allowed_distance() {
    let p1 = Point::new(10.0, 0.0);
    let p2 = Point::new(10.0, 10.0);
    let original_p1 = Point::new(0.0, 0.0);
    let original_p2 = Point::new(10.0, 0.0);
    assert!(is_allowed_distance(&p1, &p2, &original_p1, &original_p2, 0));
}

fn pow2(x: f64) -> f64 {
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

pub fn make_out_edges(edges: &[Edge], n_vertices: usize) -> Vec<Vec<usize>> {
    let mut out_edges = vec![vec![]; n_vertices];
    for e in edges.iter() {
        out_edges[e.v].push(e.w);
        out_edges[e.w].push(e.v);
    }
    out_edges
}
