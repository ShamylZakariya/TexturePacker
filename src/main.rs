use macroquad::prelude::*;
use std::fmt::Display;

/////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
struct Patch {
    id: i32,
    center: Vec2,
    extent: Vec2,
    rotation: f32,
}

impl Patch {
    fn width(&self) -> f32 {
        self.extent.x
    }

    fn height(&self) -> f32 {
        self.extent.y
    }

    fn left(&self) -> f32 {
        self.center.x - self.extent.x / 2.
    }

    fn right(&self) -> f32 {
        self.center.x + self.extent.x / 2.
    }

    fn top(&self) -> f32 {
        self.center.y - self.extent.y / 2.
    }

    fn bottom(&self) -> f32 {
        self.center.y + self.extent.y / 2.
    }

    fn uprighted(&self) -> Self {
        if self.width() > self.height() {
            Self {
                id: self.id,
                center: self.center,
                extent: Vec2::new(self.extent.y, self.extent.x),
                rotation: std::f32::consts::FRAC_PI_2,
            }
        } else {
            *self
        }
    }

    fn with_left_and_top(&self, left: f32, top: f32) -> Self {
        Self {
            id: self.id,
            center: Vec2::new(left + self.extent.x / 2., top + self.extent.y / 2.),
            extent: self.extent,
            rotation: self.rotation,
        }
    }

    fn overlaps(&self, other: &Patch) -> bool {
        let (x_overlap, y_overlap) = {
            (
                self.left() <= other.left() + other.width()
                    && self.left() + self.width() >= other.left(),
                self.top() <= other.top() + other.height()
                    && self.top() + self.height() >= other.top(),
            )
        };

        x_overlap && y_overlap
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
                    center: Vec2::new(center_x, center_y),
                    extent: Vec2::new(width, height),
                    rotation: 0.,
                };
                patches.push(patch);
            }
        }

        InitialState { patches }
    }
}

#[derive(Clone)]
struct UprightedState {
    patches: Vec<Patch>,
}

impl UprightedState {
    fn from(state: InitialState) -> Self {
        Self {
            patches: state.patches.iter().map(|r| r.uprighted()).collect(),
        }
    }
}

#[derive(Clone)]
struct SortedByHeightState {
    patches: Vec<Patch>,
}

impl SortedByHeightState {
    fn from(state: UprightedState, padding: f32) -> Self {
        let mut sorted_by_height = state.patches;
        sorted_by_height.sort_by(|a, b| b.height().partial_cmp(&a.height()).unwrap());

        let mut arranged_by_height: Vec<Patch> = Vec::new();
        for patch in sorted_by_height {
            arranged_by_height.push(if let Some(last) = arranged_by_height.last() {
                patch.with_left_and_top(last.right() + padding, padding)
            } else {
                patch.with_left_and_top(padding, padding)
            });
        }
        Self {
            patches: arranged_by_height,
        }
    }
}

#[derive(Clone)]
struct FlowedState {
    patches: Vec<Patch>,
}

impl FlowedState {
    fn from(state: SortedByHeightState, padding: f32) -> Self {
        let mut current_y = padding;
        let mut current_x = padding;
        let mut row_height = 0f32;
        let mut result: Vec<Patch> = Vec::new();

        for patch in &state.patches {
            if current_x + patch.width() > screen_width() {
                current_x = padding;
                current_y += row_height;
                row_height = 0f32;
            }

            result.push(patch.with_left_and_top(current_x, current_y));
            current_x += patch.width() + padding;
            row_height = row_height.max(patch.height() + padding);
        }

        Self { patches: result }
    }
}

#[derive(Clone)]
struct PackedUpwardsState {
    patches: Vec<Patch>,
}

impl PackedUpwardsState {
    fn from(state: FlowedState, padding: f32) -> Self {
        let mut result = Vec::new();

        for patch in &state.patches {
            // define a rect going from top of this rect to top of screen
            let test_height = patch.top() - 1.;
            let test = Patch {
                id: -1,
                center: Vec2::new(patch.center.x, test_height / 2.),
                extent: Vec2::new(patch.width(), test_height),
                rotation: 0.,
            };

            let mut bottom: f32 = 0.;
            for candidate in Self::find_intersections(test, &result) {
                bottom = bottom.max(candidate.bottom());
            }
            result.push(patch.with_left_and_top(patch.left(), bottom + padding));
        }

        Self { patches: result }
    }

    fn find_intersections(test: Patch, among: &[Patch]) -> Vec<Patch> {
        among.iter().filter(|p| test.overlaps(p)).copied().collect()
    }
}

#[derive(Clone)]
enum Sequence {
    Initial(InitialState),
    Upright(UprightedState),
    Sorted(SortedByHeightState),
    Flowed(FlowedState),
    PackedUpwards(PackedUpwardsState),
}

impl Display for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Sequence::Initial(_) => write!(f, "Initial"),
            Sequence::Upright(_) => write!(f, "Upright"),
            Sequence::Sorted(_) => write!(f, "Sorted"),
            Sequence::Flowed(_) => write!(f, "Flowed"),
            Sequence::PackedUpwards(_) => write!(f, "PackedUpwards"),
        }
    }
}

impl Sequence {
    fn new(cols: i32, rows: i32) -> Sequence {
        Sequence::Initial(InitialState::new(cols, rows))
    }

    fn next(self) -> Sequence {
        let padding = 4.;
        match self {
            Sequence::Initial(state) => Sequence::Upright(UprightedState::from(state)),
            Sequence::Upright(state) => Sequence::Sorted(SortedByHeightState::from(state, padding)),
            Sequence::Sorted(state) => Sequence::Flowed(FlowedState::from(state, padding)),
            Sequence::Flowed(state) => Sequence::PackedUpwards(PackedUpwardsState::from(state, padding)),
            Sequence::PackedUpwards(_) => self,
        }
    }

    fn patches(&self) -> &Vec<Patch> {
        match self {
            Sequence::Initial(state) => &state.patches,
            Sequence::Upright(state) => &state.patches,
            Sequence::Sorted(state) => &state.patches,
            Sequence::Flowed(state) => &state.patches,
            Sequence::PackedUpwards(state) => &state.patches,
        }
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

fn ease(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let t = t / (d / 2.);
    if t < 1. {
        c / 2. * t * t * t + b
    } else {
        let t = t - 2.;
        c / 2. * (t * t * t + 2.) + b
    }
}

fn ease_unit(t: f32) -> f32 {
    ease(t.clamp(0., 1.), 0., 1., 1.)
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

fn draw_patches(patches: &[Patch], color: Color) {
    for patch in patches {
        draw_rectangle(
            patch.left(),
            patch.top(),
            patch.width(),
            patch.height(),
            color,
        );
        draw_text(
            format!("{}", patch.id).as_str(),
            patch.center.x,
            patch.center.y,
            16.,
            WHITE,
        );
    }
}

fn draw_interpolated_patches(old_patches: &[Patch], new_patches: &[Patch], t: f32, color: Color) {
    let t = t.clamp(0., 1.);
    let t = ease_unit(t);
    for (old, current) in old_patches.iter().zip(new_patches.iter()) {
        let center = old.center + t * (current.center - old.center);
        let extent = old.extent + t * (current.extent - old.extent);
        draw_rectangle(
            center.x - extent.x / 2.,
            center.y - extent.y / 2.,
            extent.x,
            extent.y,
            color,
        );
        draw_text(
            format!("{}", current.id).as_str(),
            center.x,
            center.y,
            16.,
            WHITE,
        );
    }
}

#[macroquad::main(conf)]
async fn main() {
    let rows = 6;
    let cols = 3;
    let mut sequence = Sequence::new(cols, rows);
    let mut previous_patches = None;
    let mut current_patches = sequence.patches().clone();
    let mut last_step_time = None;

    loop {
        if is_key_pressed(KeyCode::Space) {
            previous_patches = Some(current_patches.clone());
            sequence = sequence.next();
            current_patches = sequence.patches().clone();
            last_step_time = Some(get_time());
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        clear_background(WHITE);

        match &sequence {
            Sequence::Initial(_) | Sequence::Upright(_) => {
                draw_screen_grid(cols, rows, LIGHTGRAY);
            }
            _ => {}
        }

        if let Some(last_step_time) = last_step_time {
            if let Some(previous_patches) = &previous_patches {
                let now = get_time();
                let elapsed = now - last_step_time;
                draw_interpolated_patches(
                    previous_patches,
                    &current_patches,
                    elapsed as f32,
                    DARKGRAY,
                );
            }
        } else {
            draw_patches(&current_patches, DARKGRAY);
        }

        draw_text(
            format!("{}", sequence).as_str(),
            20.0,
            screen_height() - 20.,
            30.0,
            DARKGRAY,
        );

        next_frame().await
    }
}
