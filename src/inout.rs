use crate::common::*;
use serde::{Deserialize, Serialize};
use std::io::Read;

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

#[allow(dead_code)]
pub fn figure_to_pose_json(figure: &Figure) -> String {
    let vertices: Vec<Vec<i64>> = figure
        .vertices
        .iter()
        .map(|p| vec![p.x() as i64, p.y() as i64])
        .collect();
    let pose_json = PoseJSON { vertices };
    serde_json::to_string(&pose_json).unwrap()
}

pub fn vertices_to_pose_json(vertices: &[Point]) -> String {
    let vs: Vec<Vec<i64>> = vertices
        .iter()
        .map(|p| vec![p.x() as i64, p.y() as i64])
        .collect();
    let pose_json = PoseJSON { vertices: vs };
    serde_json::to_string(&pose_json).unwrap()
}
