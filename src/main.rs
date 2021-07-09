use geo::algorithm::contains::Contains;
use serde::{Deserialize, Serialize};
use std::io::Read;

pub type Point = geo::Point<f64>;
pub type Polygon = geo::Polygon<f64>;
pub type Line = geo::Line<f64>;

#[derive(Debug, Serialize, Deserialize)]
pub struct InputJSON {
    pub hole: Vec<Vec<i64>>,
    pub figure: FigureJSON,
    pub epsilon: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FigureJSON {
    pub edges: Vec<Vec<usize>>,
    pub vertices: Vec<Vec<i64>>,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub hole: Polygon,
    pub figure: Figure,
    pub epsilon: i64,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct PoseJSON {
    pub vertices: Vec<Vec<i64>>,
}

pub fn read_input() -> Input {
    let mut data = String::new();
    std::io::stdin().read_to_string(&mut data).unwrap();

    let input_json: InputJSON = serde_json::from_str(&data).expect("failed to parse input as JSON");

    let hole: Vec<(f64, f64)> = input_json
        .hole
        .iter()
        .map(|p| (p[0] as f64, p[1] as f64))
        .collect();
    let edges: Vec<Edge> = input_json
        .figure
        .edges
        .iter()
        .map(|e| Edge::new(e[0], e[1]))
        .collect();
    let vertices: Vec<Point> = input_json
        .figure
        .vertices
        .iter()
        .map(|p| Point::new(p[0] as f64, p[1] as f64))
        .collect();

    Input {
        hole: Polygon::new(geo::LineString::from(hole), vec![]),
        figure: Figure { edges, vertices },
        epsilon: input_json.epsilon,
    }
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

fn translate(src: &Figure, dx: f64, dy: f64, dest: &mut Figure) {
    for i in 0..src.vertices.len() {
        let v = src.vertices[i];
        dest.vertices[i] = Point::new(v.x() + dx, v.y() + dy);
    }
}

fn rotate_90_in_place(figure: &mut Figure) {
    for v in figure.vertices.iter_mut() {
        *v = Point::new(-v.y(), v.x());
    }
}

fn mirror_x_in_place(figure: &mut Figure) {
    for v in figure.vertices.iter_mut() {
        *v = Point::new(-v.x(), v.y());
    }
}

fn try_all_translations(original_figure: &Figure, hole: &Polygon) -> Option<(Figure, f64)> {
    let mut figure = original_figure.clone();
    let mut best_figure = None;
    let mut best_dislike = 1e20;
    for dy in -100..=100 {
        for dx in -100..=100 {
            translate(original_figure, dx as f64, dy as f64, &mut figure);
            if does_figure_fit_in_hole(&figure, hole) {
                let dislike = calculate_dislike(&figure, hole);
                if dislike < best_dislike {
                    best_figure = Some(figure.clone());
                    best_dislike = dislike;
                }
            }
        }
    }
    best_figure.map(|f| (f, best_dislike))
}

fn try_all_translations_rotations_and_mirrors(
    original_figure: &Figure,
    hole: &Polygon,
) -> Option<(Figure, f64)> {
    let mut figure = original_figure.clone();
    let mut best_figure = None;
    let mut best_dislike = 1e20;
    for _i in 0..2 {
        for _j in 0..4 {
            if let Some((f, dislike)) = try_all_translations(&figure, hole) {
                if dislike < best_dislike {
                    best_figure = Some(f);
                    best_dislike = dislike;
                }
            }
            rotate_90_in_place(&mut figure);
        }
        mirror_x_in_place(&mut figure);
    }
    best_figure.map(|f| (f, best_dislike))
}

pub fn figure_to_pose_json(figure: &Figure) -> String {
    let vertices: Vec<Vec<i64>> = figure
        .vertices
        .iter()
        .map(|p| vec![p.x() as i64, p.y() as i64])
        .collect();
    let pose_json = PoseJSON { vertices };
    serde_json::to_string(&pose_json).unwrap()
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

fn main() {
    let input = read_input();
    if let Some((solution, dislike)) =
        try_all_translations_rotations_and_mirrors(&input.figure, &input.hole)
    {
        let j = figure_to_pose_json(&solution);
        println!("{}", j);
        eprintln!("dislike = {}", dislike);
    } else {
        eprintln!("No solutions");
        std::process::exit(1);
    }
}

#[test]
fn test_contains() {
    let l1 = Line::new(Point::new(0.0, 10.0), Point::new(20.0, 10.0));
    // let l2 = Line::new(Point::new(13.0, 10.0), Point::new(20.0, 10.0));
    let hole2 = Polygon::new(
        geo::LineString::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(20.0, 10.0),
            Point::new(20.0, 20.0),
            Point::new(0.0, 20.0),
            Point::new(0.0, 0.0),
        ]),
        vec![],
    );
    assert!(hole2.contains(&l1));
    // bug?
    // assert!(hole2.contains(&l2));
}
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
