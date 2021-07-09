use std::io::Read;
use serde::{Deserialize, Serialize};

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

fn main() {
    let mut data = String::new();
    std::io::stdin().read_to_string(&mut data).unwrap();

    let input: InputJSON = serde_json::from_str(&data).expect("failed to parse input as JSON");

    println!("input = {:?}", input);
}
