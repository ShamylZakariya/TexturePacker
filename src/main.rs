use macroquad::prelude::*;

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

trait State {
    fn name(&self) -> &'static str;
    fn next(&self) -> Option<Box<dyn State>>;
    fn patches(&self) -> &Vec<Patch>;
}

#[derive(Clone, Copy)]
struct PackingConfig {
    width: f32,
    height: f32,
    padding: f32,
}

struct InitialState {
    patches: Vec<Patch>,
    config: PackingConfig,
}

impl InitialState {
    fn new(config: PackingConfig, cols: i32, rows: i32) -> InitialState {
        let mut patches: Vec<Patch> = Vec::new();
        let cell_width = config.width / (cols as f32);
        let cell_height = config.height / (rows as f32);
        let max_width = cell_width * 1.1;
        let max_height = cell_height * 1.1;
        let min_width = cell_width * 0.5;
        let min_height = cell_height * 0.5;

        for row in 0..rows {
            for col in 0..cols {
                let across_x = (col as f32) / (cols as f32);
                let across_y = (row as f32) / (rows as f32);
                let width = rand::gen_range(min_width, max_width);
                let height = rand::gen_range(min_height, max_height);
                let center_x = (config.width * across_x) + (cell_width / 2.);
                let center_y = (config.height * across_y) + (cell_height / 2.);
                let patch = Patch {
                    id: patches.len() as i32,
                    center: Vec2::new(center_x, center_y),
                    extent: Vec2::new(width, height),
                    rotation: 0.,
                };
                patches.push(patch);
            }
        }

        InitialState { patches, config }
    }
}

impl State for InitialState {
    fn name(&self) -> &'static str {
        "Initial"
    }

    fn next(&self) -> Option<Box<dyn State>> {
        Some(Box::new(UprightedState::from(self)))
    }

    fn patches(&self) -> &Vec<Patch> {
        &self.patches
    }
}

#[derive(Clone)]
struct UprightedState {
    patches: Vec<Patch>,
    config: PackingConfig,
}

impl UprightedState {
    fn from(state: &InitialState) -> Self {
        Self {
            patches: state.patches.iter().map(|r| r.uprighted()).collect(),
            config: state.config,
        }
    }
}

impl State for UprightedState {
    fn name(&self) -> &'static str {
        "Uprighted"
    }

    fn next(&self) -> Option<Box<dyn State>> {
        Some(Box::new(SortedByHeightState::from(self)))
    }

    fn patches(&self) -> &Vec<Patch> {
        &self.patches
    }
}

#[derive(Clone)]
struct SortedByHeightState {
    patches: Vec<Patch>,
    config: PackingConfig,
}

impl SortedByHeightState {
    fn from(state: &UprightedState) -> Self {
        let mut sorted_by_height = state.patches.clone();
        sorted_by_height.sort_by(|a, b| b.height().partial_cmp(&a.height()).unwrap());

        let mut arranged_by_height: Vec<Patch> = Vec::new();
        for patch in sorted_by_height {
            arranged_by_height.push(if let Some(last) = arranged_by_height.last() {
                patch.with_left_and_top(last.right() + state.config.padding, state.config.padding)
            } else {
                patch.with_left_and_top(state.config.padding, state.config.padding)
            });
        }
        Self {
            patches: arranged_by_height,
            config: state.config,
        }
    }
}

impl State for SortedByHeightState {
    fn name(&self) -> &'static str {
        "Sorted by Height"
    }

    fn next(&self) -> Option<Box<dyn State>> {
        Some(Box::new(FlowedState::from(self)))
    }

    fn patches(&self) -> &Vec<Patch> {
        &self.patches
    }
}

#[derive(Clone)]
struct FlowedState {
    patches: Vec<Patch>,
    config: PackingConfig,
}

impl FlowedState {
    fn from(state: &SortedByHeightState) -> Self {
        let padding = state.config.padding;
        let mut current_y = padding;
        let mut current_x = padding;
        let mut row_height = 0f32;
        let mut result: Vec<Patch> = Vec::new();
        let mut row = 0;

        for patch in &state.patches {
            if row % 2 == 0 {
                if current_x + patch.width() > state.config.width {
                    current_x = state.config.width - padding - patch.width();
                    current_y += row_height;
                    row_height = 0f32;
                    row += 1;
                }
            } else {
                current_x -= patch.width() + padding;
                if current_x < padding {
                    current_x = padding;
                    current_y += row_height;
                    row_height = 0.;
                    row += 1;
                }
            }

            result.push(patch.with_left_and_top(current_x, current_y));
            row_height = row_height.max(patch.height() + padding);

            if row % 2 == 0 {
                current_x += patch.width() + padding;
            }
        }

        Self {
            patches: result,
            config: state.config,
        }
    }
}

impl State for FlowedState {
    fn name(&self) -> &'static str {
        "Flowed"
    }

    fn next(&self) -> Option<Box<dyn State>> {
        Some(Box::new(PackedUpwardsState::from(self)))
    }

    fn patches(&self) -> &Vec<Patch> {
        &self.patches
    }
}

#[derive(Clone)]
struct PackedUpwardsState {
    patches: Vec<Patch>,
    config: PackingConfig,
}

impl PackedUpwardsState {
    fn from(state: &FlowedState) -> Self {
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
            result.push(patch.with_left_and_top(patch.left(), bottom + state.config.padding));
        }

        Self {
            patches: result,
            config: state.config,
        }
    }

    fn find_intersections(test: Patch, among: &[Patch]) -> Vec<Patch> {
        among.iter().filter(|p| test.overlaps(p)).copied().collect()
    }
}

impl State for PackedUpwardsState {
    fn name(&self) -> &'static str {
        "Packed Upwards"
    }

    fn next(&self) -> Option<Box<dyn State>> {
        None
    }

    fn patches(&self) -> &Vec<Patch> {
        &self.patches
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
    let config = PackingConfig {
        width: screen_width(),
        height: screen_height(),
        padding: 4.,
    };
    let mut previous_state: Option<Box<dyn State>> = None;
    let mut state: Box<dyn State> = Box::new(InitialState::new(config, cols, rows));
    let mut last_step_time = None;
    let patch_color: Color = [60, 60, 60, 128].into();

    loop {
        if is_key_pressed(KeyCode::Space) {
            if let Some(new_state) = state.next() {
                previous_state = Some(state);
                state = new_state;
                last_step_time = Some(get_time());
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        clear_background(WHITE);

        if let Some(last_step_time) = last_step_time {
            if let Some(previous_state) = &previous_state {
                let now = get_time();
                let elapsed = now - last_step_time;
                draw_interpolated_patches(
                    previous_state.patches(),
                    state.patches(),
                    elapsed as f32,
                    patch_color,
                );
            }
        } else {
            draw_patches(state.patches(), patch_color);
        }

        draw_text(state.name(), 20.0, screen_height() - 20., 30.0, DARKGRAY);

        next_frame().await
    }
}
