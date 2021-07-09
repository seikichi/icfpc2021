mod common;
mod inout;
mod solvers;

use inout::*;

fn main() {
    let input = read_input();
    if let Some((solution, dislike)) = solvers::orthgonal::solve(&input) {
        let j = figure_to_pose_json(&solution);
        println!("{}", j);
        eprintln!("dislike = {}", dislike);
    } else {
        eprintln!("No solutions");
        std::process::exit(1);
    }
}
