use macroquad::prelude::*;
use std::fmt::Display;

/////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
struct Patch {
    id: i32,
    bounds: Rect,
    uprighted: bool,
}

impl Patch {
    fn uprighted(&self) -> Self {
        if self.bounds.w > self.bounds.h {
            let half_width = self.bounds.w / 2.;
            let half_height = self.bounds.h / 2.;
            let center_x = self.bounds.x + half_width;
            let center_y = self.bounds.y + half_height;
            let new_rect = Rect::new(
                center_x - half_height,
                center_y - half_width,
                self.bounds.h,
                self.bounds.w,
            );
            Self {
                id: self.id,
                bounds: new_rect,
                uprighted: true,
            }
        } else {
            *self
        }
    }

    fn with_new_bounds(&self, bounds: Rect) -> Self {
        Self {
            id: self.id,
            bounds,
            uprighted: self.uprighted,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct InitialState {
    patches: Vec<Patch>,
}

impl InitialState {
    fn new(cols: i32, rows: i32) -> InitialState {
        let mut patches: Vec<Patch> = Vec::new();
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
                let patch = Patch {
                    id: patches.len() as i32,
                    bounds: Rect::new(center_x - width / 2., center_y - height / 2., width, height),
                    uprighted: false,
                };
                patches.push(patch);
            }
        }

        InitialState { patches }
    }

    fn next(&self) -> UprightedState {
        UprightedState {
            patches: self.patches.iter().map(|r| r.uprighted()).collect(),
        }
    }
}

#[derive(Clone)]
struct UprightedState {
    patches: Vec<Patch>,
}

impl UprightedState {
    fn next(&self, padding: f32) -> SortedByHeightState {
        let mut sorted_by_height = self.patches.clone();
        sorted_by_height.sort_by(|a, b| b.bounds.h.partial_cmp(&a.bounds.h).unwrap());

        let mut arranged_by_height: Vec<Patch> = Vec::new();
        for patch in sorted_by_height {
            arranged_by_height.push(if let Some(last) = arranged_by_height.last() {
                patch.with_new_bounds(Rect::new(
                    last.bounds.right() + padding,
                    last.bounds.y,
                    patch.bounds.w,
                    patch.bounds.h,
                ))
            } else {
                patch.with_new_bounds(Rect::new(padding, padding, patch.bounds.w, patch.bounds.h))
            });
        }
        SortedByHeightState {
            patches: arranged_by_height,
        }
    }
}

#[derive(Clone)]
struct SortedByHeightState {
    patches: Vec<Patch>,
}

impl SortedByHeightState {
    fn next(&self, padding: f32) -> FlowedState {
        let mut current_y = padding;
        let mut current_x = padding;
        let mut row_height = 0f32;
        let mut result: Vec<Patch> = Vec::new();

        for patch in &self.patches {
            if current_x + patch.bounds.w > screen_width() {
                current_x = padding;
                current_y += row_height;
                row_height = 0f32;
            }

            result.push(patch.with_new_bounds(Rect::new(
                current_x,
                current_y,
                patch.bounds.w,
                patch.bounds.h,
            )));
            current_x += patch.bounds.w + padding;
            row_height = row_height.max(patch.bounds.h + padding);
        }

        FlowedState { patches: result }
    }
}

#[derive(Clone)]
struct FlowedState {
    patches: Vec<Patch>,
}

impl FlowedState {
    fn next(&self, padding: f32) -> PackedUpwardsState {
        let mut result = Vec::new();

        for patch in &self.patches {
            // define a rect going from top of tjis rect to top of screen
            let test = Rect::new(patch.bounds.x, 0., patch.bounds.w, patch.bounds.y - 1.);
            let mut bottom: f32 = 0.;
            for candidate in Self::find_intersections(test, &result) {
                bottom = bottom.max(candidate.bounds.bottom());
            }
            result.push(patch.with_new_bounds(Rect::new(
                patch.bounds.x,
                bottom + padding,
                patch.bounds.w,
                patch.bounds.h,
            )));
        }

        PackedUpwardsState { patches: result }
    }

    fn find_intersections(bounds: Rect, among: &[Patch]) -> Vec<Patch> {
        let mut result = Vec::new();
        for p in among {
            if p.bounds.overlaps(&bounds) {
                result.push(*p);
            }
        }
        result
    }
}

#[derive(Clone)]
struct PackedUpwardsState {
    patches: Vec<Patch>,
}

#[derive(Clone)]
enum State {
    Initial(InitialState),
    Upright(UprightedState),
    Sorted(SortedByHeightState),
    Flowed(FlowedState),
    PackedUpwards(PackedUpwardsState),
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            State::Initial(_) => write!(f, "Initial"),
            State::Upright(_) => write!(f, "Upright"),
            State::Sorted(_) => write!(f, "Sorted"),
            State::Flowed(_) => write!(f, "Flowed"),
            State::PackedUpwards(_) => write!(f, "PackedUpwards"),
        }
    }
}

impl State {
    fn new(cols: i32, rows: i32) -> State {
        State::Initial(InitialState::new(cols, rows))
    }

    fn is_done(&self) -> bool {
        matches!(*self, State::PackedUpwards(_))
    }

    fn next(self) -> State {
        match self {
            State::Initial(state) => State::Upright(state.next()),
            State::Upright(state) => State::Sorted(state.next(Self::padding())),
            State::Sorted(state) => State::Flowed(state.next(Self::padding())),
            State::Flowed(state) => State::PackedUpwards(state.next(Self::padding())),
            State::PackedUpwards(_) => self.clone(),
        }
    }

    fn patches(&self) -> &Vec<Patch> {
        match self {
            State::Initial(state) => &state.patches,
            State::Upright(state) => &state.patches,
            State::Sorted(state) => &state.patches,
            State::Flowed(state) => &state.patches,
            State::PackedUpwards(state) => &state.patches,
        }
    }

    fn padding() -> f32 {
        4.
    }
}

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

fn draw(patches: &[Patch], color: Color) {
    for patch in patches {
        draw_rectangle(
            patch.bounds.x,
            patch.bounds.y,
            patch.bounds.w,
            patch.bounds.h,
            color,
        );
    }
}

#[macroquad::main(conf)]
async fn main() {
    let rows = 6;
    let cols = 3;
    let mut step = State::new(cols, rows);

    loop {
        if is_key_pressed(KeyCode::Space) && !step.is_done() {
            step = step.next();
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        clear_background(WHITE);

        match &step {
            State::Initial(_) | State::Upright(_) => {
                draw_screen_grid(cols, rows, LIGHTGRAY);
            }
            _ => {}
        }

        draw(step.patches(), DARKGRAY);
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
