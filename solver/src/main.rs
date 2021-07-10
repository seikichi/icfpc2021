mod common;
mod inout;
mod solvers;

use inout::*;
use std::time::Duration;

fn main() {
    let input = read_input();
    if let Some((solution1, dislike1)) = solvers::dfs::solve(&input) {
        eprintln!("dfs: dislike = {}", dislike1);
        let (solution2, dislike2) = solvers::hill_climbing::solve(&input, solution1, Duration::from_millis(10000));
        let j = vertices_to_pose_json(&solution2);
        println!("{}", j);
        if !common::does_valid_pose(&solution2, &input.figure, &input.hole, input.epsilon) {
            eprintln!("Pose is invalid");
            std::process::exit(1);
        }
        eprintln!("hill_climbing: dislike = {}", dislike2);
    } else {
        eprintln!("No solutions");
        std::process::exit(1);
    }
}
