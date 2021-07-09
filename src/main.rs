use serde::{Deserialize, Serialize};
use std::io::Read;

type Point = geo::Point<f64>;

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
    hole: Vec<Point>,
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

fn main() {
    let mut data = String::new();
    std::io::stdin().read_to_string(&mut data).unwrap();

    let input_json: InputJSON = serde_json::from_str(&data).expect("failed to parse input as JSON");

    let hole: Vec<Point> = input_json
        .hole
        .iter()
        .map(|p| Point::new(p[0] as f64, p[1] as f64))
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

    let input = Input {
        hole,
        figure: Figure { edges, vertices },
        epsilon: input_json.epsilon,
    };

    println!("input = {:?}", input);
}
