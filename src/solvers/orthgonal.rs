use crate::common::*;

pub fn solve(input: &Input) -> Option<(Figure, f64)> {
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

fn try_all_translations(original_figure: &Figure, hole: &Polygon) -> Option<(Figure, f64)> {
    let mut figure = original_figure.clone();
    let mut best_figure = None;
    let mut best_dislike = 1e20;
    for dy in -100..=100 {
        for dx in -100..=100 {
            translate(original_figure, dx as f64, dy as f64, &mut figure);
            if does_figure_fit_in_hole(&figure, hole) {
                let dislike = calculate_dislike(&figure, hole);
                if dislike < best_dislike {
                    best_figure = Some(figure.clone());
                    best_dislike = dislike;
                }
            }
        }
    }
    best_figure.map(|f| (f, best_dislike))
}

fn try_all_translations_rotations_and_mirrors(
    original_figure: &Figure,
    hole: &Polygon,
) -> Option<(Figure, f64)> {
    let mut figure = original_figure.clone();
    let mut best_figure = None;
    let mut best_dislike = 1e20;
    for _i in 0..2 {
        for _j in 0..4 {
            if let Some((f, dislike)) = try_all_translations(&figure, hole) {
                if dislike < best_dislike {
                    best_figure = Some(f);
                    best_dislike = dislike;
                }
            }
            rotate_90_in_place(&mut figure);
        }
        mirror_x_in_place(&mut figure);
    }
    best_figure.map(|f| (f, best_dislike))
}