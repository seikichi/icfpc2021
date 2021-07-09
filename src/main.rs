use serde::{Deserialize, Serialize};
use std::io::Read;
use geo::algorithm::contains::Contains;

type Point = geo::Point<f64>;
type Polygon = geo::Polygon<f64>;
type Line = geo::Line<f64>;

#[derive(Debug, Serialize, Deserialize)]
struct InputJSON {
    hole: Vec<Vec<i64>>,
    figure: FigureJSON,
    epsilon: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct FigureJSON {
    edges: Vec<Vec<usize>>,
    vertices: Vec<Vec<i64>>,
}

#[derive(Debug, Clone)]
struct Input {
    hole: Polygon,
    figure: Figure,
    epsilon: i64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Edge {
    v: usize,
    w: usize,
}

impl Edge {
    fn new(v: usize, w: usize) -> Edge {
        Edge { v, w }
    }
}

#[derive(Debug, Clone)]
struct Figure {
    edges: Vec<Edge>,
    vertices: Vec<Point>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PoseJSON {
    vertices: Vec<Vec<i64>>,
}

fn read_input() -> Input {
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

fn does_figure_fit_in_hole(figure: &Figure, hole: &Polygon) -> bool {
    for e in figure.edges.iter() {
        let p1 = figure.vertices[e.v];
        let p2 = figure.vertices[e.w];
        let line = Line::new(p1, p2);
        if !hole.contains(&line) {
            return false;
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

fn try_all_translations(input: &Input) -> Option<Figure> {
    let mut figure = input.figure.clone();
    for dy in -100..=100 {
        for dx in -100..=100 {
            translate(&input.figure, dx as f64, dy as f64, &mut figure);
            if does_figure_fit_in_hole(&figure, &input.hole) {
                return Some(figure);
            }
        }
    }
    None
}

fn figure_to_pose_json(figure: &Figure) -> String {
    let vertices: Vec<Vec<i64>> = figure.vertices.iter().map(|p| vec![p.x() as i64, p.y() as i64]).collect();
    let pose_json = PoseJSON { vertices };
    serde_json::to_string(&pose_json).unwrap()
}

fn main() {
    let input = read_input();
    if let Some(solution) = try_all_translations(&input) {
        let j = figure_to_pose_json(&solution);
        println!("{}", j);
    } else {
        eprintln!("No solutions");
    }
}
