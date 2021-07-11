mod common;
mod inout;
mod solvers;

use crate::common::*;
use inout::*;
use std::time::Duration;

fn main() {
    let initial_solver: String = {
        if let Ok(s) = std::env::var("INITIAL_SOLVER") {
            s
        } else {
            "dfs".to_string()
        }
    };
    let used_bonus_types: Vec<BonusType> = {
        if let Ok(ss) = std::env::var("USED_BONUS_TYPES") {
            ss.split(",").map(|s| BonusType::from_str(&s)).collect()
        } else {
            vec![]
        }
    };
    let disable_dfs_centroid = std::env::var("DISABLE_DFS_CENTROID").is_ok();
    let use_hill_climbing = std::env::var("USE_HILL_CLIMBING").is_ok();
    let skip_ortho = std::env::var("SKIP_ORTHO").is_ok();
    let hill_climbing_time_limit = {
        if let Ok(s) = std::env::var("HILL_CLIMBING_TIME_LIMIT_SECONDS") {
            let f: f64 = s.parse().expect("Invalid HILL_CLIMBING_TIME_LIMIT_SECONDS");
            Duration::from_secs_f64(f)
        } else {
            Duration::from_millis(2000)
        }
    };
    eprintln!("initial_solver = {}", initial_solver);
    eprintln!("hill_climbing_time_limit = {:?}", hill_climbing_time_limit);

    let input = read_input();
    let initial_solution = match initial_solver.as_str() {
        "dfs" => solvers::dfs::solve(&input, disable_dfs_centroid),
        "shrink" => solvers::shrink::solve(&input),
        _ => panic!("INITIAL_SOLVER {} is invalid.", initial_solver),
    };
    if let Some((solution1, dislike1)) = initial_solution {
        eprintln!("initial: dislike = {}", dislike1);

        let solution2 = if skip_ortho {
            solution1
        } else {
            // orthgonal1
            let mut input2 = input.clone();
            input2.figure.vertices = solution1;
            let (solution2, dislike2) = solvers::orthgonal::solve(&input2).unwrap();
            eprintln!("orthgonal: dislike = {}", dislike2);
            solution2
        };

        let (solution3, dislike3) = if use_hill_climbing {
            eprintln!("hill climbing...");
            solvers::hill_climbing::solve(&input, solution2, hill_climbing_time_limit)
        } else {
            eprintln!("annealing...");
            solvers::annealing::solve(&input, solution2, hill_climbing_time_limit)
        };
        eprintln!("hill_climbing/annealing: dislike = {}", dislike3);

        let solution4 = if skip_ortho {
            solution3
        } else {
            // orthgonal2
            let mut input3 = input.clone();
            input3.figure.vertices = solution3;
            let (solution4, dislike4) = solvers::orthgonal::solve(&input3).unwrap();
            eprintln!("orthgonal: dislike = {}", dislike4);
            solution4
        };

        // adjust
        let (solution5, dislike5) = solvers::adjust::solve(&input, &used_bonus_types, solution4);
        eprintln!("adjust: dislike = {}", dislike5);

        // output
        let j = vertices_to_pose_json(&solution5, &used_bonus_types, &None);
        println!("{}", j);
        if !common::does_valid_pose(
            &solution5,
            &input.figure,
            &input.hole,
            input.epsilon,
            &used_bonus_types,
            None,
        ) {
            eprintln!("Pose is invalid");
            std::process::exit(1);
        }
    } else {
        eprintln!("No solutions");
        std::process::exit(1);
    }
}
