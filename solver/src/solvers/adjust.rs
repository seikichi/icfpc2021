use crate::common::*;

pub fn solve(
    input: &Input,
    used_bonus_types: &Vec<BonusType>,
    solution: Vec<Point>,
) -> (Vec<Point>, f64) {
    let mut solution = solution;
    let hole_points: Vec<Point> = input.hole.exterior().points_iter().skip(1).collect();
    let n = solution.len();
    let m = hole_points.len();
    let mut on_hole_vertex = vec![false; n];
    let mut satisfied = vec![false; m];
    for i in 0..n {
        for j in 0..m {
            if solution[i] == hole_points[j] {
                on_hole_vertex[i] = true;
                satisfied[j] = true;
            }
        }
    }
    let mut best_dislike = calculate_dislike(&solution, &input.hole);
    for i in 0..m {
        if satisfied[i] {
            continue;
        }
        for j in 0..n {
            if on_hole_vertex[j] {
                continue;
            }
            let temp = solution[j];
            solution[j] = hole_points[i];
            let next_solution = fix_allowed_distance_violation(j, &solution, &input);
            solution[j] = temp;
            if next_solution.is_none() {
                continue;
            }
            let next_solution = next_solution.unwrap();

            let dislike = calculate_dislike(&solution, &input.hole);
            if does_valid_pose(
                &next_solution,
                &input.figure,
                &input.hole,
                input.epsilon,
                used_bonus_types,
                None,
            ) && dislike <= best_dislike
            {
                solution = next_solution;
                best_dislike = dislike;
                satisfied[i] = true;
                on_hole_vertex[j] = true;
                break;
            }
        }
    }
    return (solution, best_dislike);
}
