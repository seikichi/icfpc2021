use crate::common::*;

#[allow(dead_code)]
pub fn solve(input: &Input) -> Option<(Vec<Point>, f64)> {
    return try_all_translations_rotations_and_mirrors(&input.figure, &input.hole);
}

fn translate(src: &Figure, dx: f64, dy: f64, dest: &mut Figure) {
    for i in 0..src.vertices.len() {
        let v = src.vertices[i];
        dest.vertices[i] = Point::new(v.x() + dx, v.y() + dy);
    }
}

fn rotate_90_in_place(figure: &mut Figure) {
    for v in figure.vertices.iter_mut() {
        *v = Point::new(-v.y(), v.x());
    }
}

fn mirror_x_in_place(figure: &mut Figure) {
    for v in figure.vertices.iter_mut() {
        *v = Point::new(-v.x(), v.y());
    }
}

fn try_all_translations(
    original_figure: &Figure,
    hole: &Polygon,
    best_dislike: f64,
) -> Option<(Vec<Point>, f64)> {
    let mut figure = original_figure.clone();
    let mut best_vertices = None;
    let mut best_dislike = best_dislike;

    let hole_points: Vec<Point> = hole.exterior().points_iter().skip(1).collect();
    let hole_bound_box = calc_bound_box(&hole_points);
    let figure_bound_box = calc_bound_box(&figure.vertices);
    let l = (hole_bound_box.0.x() - figure_bound_box.0.x()) as i64;
    let r = (hole_bound_box.1.x() - figure_bound_box.1.x()) as i64;
    let u = (hole_bound_box.0.y() - figure_bound_box.0.y()) as i64;
    let b = (hole_bound_box.1.y() - figure_bound_box.1.y()) as i64;
    // eprintln!("hole bound box: {:?}", hole_bound_box);
    // eprintln!("figure bound box: {:?}", figure_bound_box);
    // eprintln!("lrub: {} {} {} {}", l, r, u, b);
    let mut y_step = 1;
    let mut x_step = 1;
    if best_dislike >= 1e+20 {
        y_step = std::cmp::max(1, (b - u) / 50);
        x_step = std::cmp::max(1, (r - l) / 50);
        // if y_step != 1 && x_step != 1 {
        //     eprintln!("orthgonal step: {} {}", y_step, x_step);
        // }
    }

    for dy in l..=r {
        if dy.abs() % y_step != 0 {
            continue;
        }
        for dx in u..=b {
            if dx.abs() % x_step != 0 {
                continue;
            }
            translate(original_figure, dx as f64, dy as f64, &mut figure);
            let dislike = calculate_dislike(&figure.vertices, hole);
            if dislike >= best_dislike {
                continue;
            }
            if does_figure_fit_in_hole(&figure, hole, false) {
                best_vertices = Some(figure.vertices.clone());
                best_dislike = dislike;
            }
        }
    }
    best_vertices.map(|v| (v, best_dislike))
}

fn try_all_translations_rotations_and_mirrors(
    original_figure: &Figure,
    hole: &Polygon,
) -> Option<(Vec<Point>, f64)> {
    let mut figure = original_figure.clone();
    let mut best_vertices = None;
    let mut best_dislike = 1e20;
    if does_figure_fit_in_hole(&figure, hole, false) {
        best_vertices = Some(figure.vertices.clone());
        best_dislike = calculate_dislike(&figure.vertices, &hole);
    }
    for _i in 0..2 {
        for _j in 0..4 {
            if let Some((vs, dislike)) = try_all_translations(&figure, hole, best_dislike) {
                if dislike < best_dislike {
                    best_vertices = Some(vs);
                    best_dislike = dislike;
                }
            }
            rotate_90_in_place(&mut figure);
        }
        mirror_x_in_place(&mut figure);
    }
    best_vertices.map(|v| (v, best_dislike))
}
