use crate::common::*;

pub fn solve(input: &Input, solution: Vec<Point>) -> (Vec<Point>, f64) {
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
    for i in 0..n {
        if on_hole_vertex[i] { continue; }
        let temp = solution[i];
        for j in 0..m {
            if satisfied[j] { continue; }
            solution[i] = hole_points[j];
            let dislike = calculate_dislike(&solution, &input.hole);
            if does_valid_pose(&solution, &input.figure, &input.hole, input.epsilon) && dislike <= best_dislike {
                best_dislike = dislike;
                satisfied[j] = true;
                break;
            }
            solution[i] = temp;
        }
    }
    return (solution, best_dislike);
}