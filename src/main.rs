use std::{borrow::Borrow, fmt::Display};

use macroquad::prelude::*;

/////////////////////////////////////////////////////////////////////////////////

fn conf() -> Conf {
    Conf {
        window_title: String::from("Texture Packer"),
        window_width: 768,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

fn draw_screen_grid(cols: i32, rows: i32, color: Color) {
    for row in 0..rows {
        let y = ((row as f32) / (rows as f32)) * screen_height();
        draw_line(0.0, y, screen_width(), y, 1.0, color);
    }
    for col in 0..cols {
        let x = ((col as f32) / (cols as f32)) * screen_width();
        draw_line(x, 0.0, x, screen_height(), 1.0, color);
    }
}

/////////////////////////////////////////////////////////////////////////////////

fn uprighted(rects: Vec<Rect>) -> Vec<Rect> {
    rects
        .iter()
        .map(|r| {
            if r.w > r.h {
                Rect::new(r.x, r.y, r.h, r.w)
            } else {
                *r
            }
        })
        .collect()
}

fn sorted(rects: Vec<Rect>, padding: f32) -> Vec<Rect> {
    let mut sorted_by_height = rects;
    sorted_by_height.sort_by(|a, b| b.h.partial_cmp(&a.h).unwrap());

    let mut arranged_by_height: Vec<Rect> = Vec::new();
    for rect in sorted_by_height {
        arranged_by_height.push(if let Some(last) = arranged_by_height.last() {
            Rect::new(last.right() + padding, last.y, rect.w, rect.h)
        } else {
            Rect::new(padding, padding, rect.w, rect.h)
        });
    }

    arranged_by_height
}

fn flowed(rects: Vec<Rect>, padding: f32) -> Vec<Rect> {
    let mut current_y = padding;
    let mut current_x = padding;
    let mut row_height = 0f32;
    let mut result: Vec<Rect> = Vec::new();

    for rect in rects {
        if current_x + rect.w > screen_width() {
            current_x = padding;
            current_y += row_height;
            row_height = 0f32;
        }

        result.push(Rect::new(current_x, current_y, rect.w, rect.h));
        current_x += rect.w + padding;
        row_height = row_height.max(rect.h + padding);
    }

    result
}

fn find_intersections(rect: Rect, all:&Vec<Rect>) -> Vec<Rect> {
    let mut result = Vec::new();
    for r in all {
        if r.overlaps(&rect) {
            result.push(*r);
        }
    }
    result
}

fn packed_upwards(rects: Vec<Rect>, padding: f32) -> Vec<Rect> {
    let mut result = Vec::new();

    for rect in &rects {
        // define a rect going from top of tjis rect to top of screen
        let test = Rect::new(rect.x, 0., rect.w, rect.y - 1.);
        let mut bottom:f32 = 0.;
        for candidate in find_intersections(test, &result) {
            bottom = bottom.max(candidate.bottom());
        }
        result.push(Rect::new(rect.x, bottom + padding, rect.w, rect.h));
    }

    result
}

fn draw(rects: &Vec<Rect>, color: Color) {
    for r in rects {
        draw_rectangle(r.x, r.y, r.w, r.h, color);
    }
}

#[derive(Clone)]
enum Step {
    Initial(Vec<Rect>),
    Upright(Vec<Rect>),
    Sorted(Vec<Rect>),
    Flowed(Vec<Rect>),
    PackedUpwards(Vec<Rect>),
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Step::Initial(_) => write!(f, "Initial"),
            Step::Upright(_) => write!(f, "Upright"),
            Step::Sorted(_) => write!(f, "Sorted"),
            Step::Flowed(_) => write!(f, "Flowed"),
            Step::PackedUpwards(_) => write!(f, "PackedUpwards"),
        }
    }
}

impl Step {
    fn new(cols: i32, rows: i32) -> Step {
        let mut rects: Vec<Rect> = Vec::new();
        let cell_width = screen_width() / (cols as f32);
        let cell_height = screen_height() / (rows as f32);
        let min_width = cell_width * 0.1;
        let min_height = cell_height * 0.1;

        for row in 0..rows {
            for col in 0..cols {
                let across_x = (col as f32) / (cols as f32);
                let across_y = (row as f32) / (rows as f32);
                let width = rand::gen_range(min_width, cell_width);
                let height = rand::gen_range(min_height, cell_height);
                let center_x = (screen_width() * across_x) + (cell_width / 2.);
                let center_y = (screen_height() * across_y) + (cell_height / 2.);
                rects.push(Rect::new(
                    center_x - width / 2.,
                    center_y - height / 2.,
                    width,
                    height,
                ));
            }
        }

        Step::Initial(rects)
    }

    fn is_done(&self) -> bool {
        match *self {
            Step::PackedUpwards(_) => true,
            _ => false,
        }
    }

    fn next(self) -> Step {
        match self {
            Step::Initial(rects) => Step::Upright(uprighted(rects)),
            Step::Upright(rects) => Step::Sorted(sorted(rects, Self::padding())),
            Step::Sorted(rects) => Step::Flowed(flowed(rects, Self::padding())),
            Step::Flowed(rects) => Step::PackedUpwards(packed_upwards(rects, Self::padding())),
            Step::PackedUpwards(_) => self.clone(),
        }
    }

    fn rects(&self) -> &Vec<Rect> {
        match self {
            Step::Initial(rects) => rects,
            Step::Upright(rects) => rects,
            Step::Sorted(rects) => rects,
            Step::Flowed(rects) => rects,
            Step::PackedUpwards(rects) => rects,
        }
    }

    fn padding() -> f32 {
        4.
    }
}

/////////////////////////////////////////////////////////////////////////////////

#[macroquad::main(conf)]
async fn main() {
    let rows = 8;
    let cols = 8;
    let mut step = Step::new(cols, rows);

    loop {
        if is_key_pressed(KeyCode::Space) && !step.is_done() {
            step = step.next();
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        clear_background(WHITE);

        match &step {
            Step::Initial(_) | Step::Upright(_) => {
                draw_screen_grid(cols, rows, LIGHTGRAY);
            }
            _ => {}
        }

        draw(step.rects(), DARKGRAY);
        draw_text(
            format!("{}", step).as_str(),
            20.0,
            screen_height() - 20.,
            30.0,
            DARKGRAY,
        );

        next_frame().await
    }
}
