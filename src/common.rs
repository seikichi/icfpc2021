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

pub fn calculate_dislike(figure: &Figure, hole: &Polygon) -> f64 {
    let mut s = 0.0;
    for h in hole.exterior().points_iter().skip(1) {
        s += figure
            .vertices
            .iter()
            .map(|v| squared_distance(v, &h))
            .fold(0.0 / 0.0, |m, x| x.min(m));
    }
    s
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
    assert!(calculate_dislike(&figure1, &hole1) == 20.0);
    assert!(calculate_dislike(&figure2, &hole1) == 0.0);
    assert!(calculate_dislike(&figure3, &hole1) == 18.0);
}

struct Ring {
    center: Point,
    inner_radius: f64,
    outer_radius: f64,
}

fn pow2(x: f64) -> f64 { x * x }

fn each_ring_points(ring: Ring, mut f: impl FnMut(Point)) {
    let y_min = (ring.center.y() - ring.outer_radius).ceil() as i64;
    let y_max = (ring.center.y() - ring.outer_radius).floor() as i64;
    let iy_min = (ring.center.y() - ring.inner_radius).ceil() as i64;
    let iy_max = (ring.center.y() - ring.inner_radius).floor() as i64;
    for y in y_min..=y_max {
        // (x - cx)^2 + (y - cy)^2 = r^2
        // x = cx +- sqrt(r^2 - (y - cy)^2)
        let s = (pow2(ring.outer_radius) - pow2(y as f64 - ring.center.y())).sqrt();
        let x_min = (ring.center.x() - s).ceil() as i64;
        let x_max = (ring.center.x() + s).floor() as i64;
        if iy_min <= y && y <= iy_max {
            let is = (pow2(ring.inner_radius) - pow2(y as f64 - ring.center.y())).sqrt();
            let ix_min = (ring.center.x() - is).ceil() as i64;
            let ix_max = (ring.center.x() + is).floor() as i64;
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
