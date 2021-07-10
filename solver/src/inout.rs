use crate::common::*;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct InputJSON {
    pub bonuses: Vec<BonusInJSON>,
    pub hole: Vec<Vec<i64>>,
    pub figure: FigureJSON,
    pub epsilon: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BonusInJSON {
    pub position: Vec<i64>,
    pub bonus: String,
    pub problem: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FigureJSON {
    pub edges: Vec<Vec<usize>>,
    pub vertices: Vec<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoseJSON {
    pub vertices: Vec<Vec<i64>>,
    pub bonuses: Vec<BonusOutJSON>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BonusOutJSON {
    pub bonus: String,
    pub problem: i64,
}

pub fn parse_input(data: &str) -> Input {
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
    let bonuses : Vec<Bonus> = input_json.bonuses.iter().map(|b|
    Bonus {
        position: Point::new(b.position[0] as f64, b.position[1] as f64),
            bonus: b.bonus.clone(),
            problem: b.problem,
        })
        .collect();

    Input {
        hole: Polygon::new(geo::LineString::from(hole), vec![]),
        figure: Figure { edges, vertices },
        epsilon: input_json.epsilon,
        bonuses: bonuses,
    }
}

#[allow(dead_code)]
pub fn load_input(path: &Path) -> Input {
    let file = std::fs::File::open(path).expect(&format!("can't open {}", path.display()));
    let mut buf_reader = std::io::BufReader::new(file);
    let mut data = String::new();
    buf_reader
        .read_to_string(&mut data)
        .expect(&format!("can't load {}", path.display()));
    parse_input(&data)
}

pub fn read_input() -> Input {
    let mut data = String::new();
    std::io::stdin().read_to_string(&mut data).unwrap();
    parse_input(&data)
}

#[allow(dead_code)]
pub fn figure_to_pose_json(figure: &Figure, bonus_strs: &Vec<String>, bonus_problems: &Vec<i64>) -> String {
    vertices_to_pose_json(&figure.vertices, bonus_strs, bonus_problems)
}

pub fn vertices_to_pose_json(vertices: &[Point], bonus_strs: &Vec<String>, bonus_problems: &Vec<i64>) -> String {
    let vs: Vec<Vec<i64>> = vertices
        .iter()
        .map(|p| vec![p.x() as i64, p.y() as i64])
        .collect();
    assert!(bonus_strs.len() == bonus_problems.len());
    let mut bonuses = vec![];
    for i in 0..bonus_strs.len() {
        bonuses.push(BonusOutJSON {
            bonus: bonus_strs[i].clone(),
            problem: bonus_problems[i],
        });
    }
    let pose_json = PoseJSON {
        vertices: vs,
        bonuses: bonuses,
    };
    serde_json::to_string(&pose_json).unwrap()
}
