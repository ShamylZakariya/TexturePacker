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
                self.center.x <= other.center.x + other.extent.x
                    && self.center.x + self.extent.x >= other.center.x,
                self.center.y <= other.center.y + other.extent.y
                    && self.center.y + self.extent.y >= other.center.y,
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
        sorted_by_height.sort_by(|a, b| b.height().partial_cmp(&a.height()).unwrap());

        let mut arranged_by_height: Vec<Patch> = Vec::new();
        for patch in sorted_by_height {
            arranged_by_height.push(if let Some(last) = arranged_by_height.last() {
                patch.with_left_and_top(last.right() + padding, padding)
            } else {
                patch.with_left_and_top(padding, padding)
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
            if current_x + patch.width() > screen_width() {
                current_x = padding;
                current_y += row_height;
                row_height = 0f32;
            }

            result.push(patch.with_left_and_top(current_x, current_y));
            current_x += patch.width() + padding;
            row_height = row_height.max(patch.height() + padding);
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
            // define a rect going from top of this rect to top of screen
            let test = Patch {
                id: -1,
                center: Vec2::new(patch.center.x, (patch.top() - 1.) / 2.),
                extent: Vec2::new(patch.width(), patch.top() - 1.),
                rotation: 0.,
            };

            let mut bottom: f32 = 0.;
            for candidate in Self::find_intersections(test, &result) {
                bottom = bottom.max(candidate.bottom());
            }
            result.push(patch.with_left_and_top(patch.left(), bottom + padding));
        }

        PackedUpwardsState { patches: result }
    }

    fn find_intersections(test: Patch, among: &[Patch]) -> Vec<Patch> {
        let mut result = Vec::new();
        for p in among {
            if p.overlaps(&test) {
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
            patch.left(),
            patch.top(),
            patch.width(),
            patch.height(),
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
