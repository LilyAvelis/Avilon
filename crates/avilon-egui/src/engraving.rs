//! Seeded engraved lines and an interactive radial lens for egui.

use egui::{
    self, Color32, CursorIcon, Id, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Vec2,
    pos2, vec2,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub seed: u32,
    /// Radius as a fraction of the engraving's height.
    pub radius: f32,
    /// Scale at the center. Above one enlarges, below one shrinks.
    pub zoom: f32,
    pub density: f32,
    pub live: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            seed: 0,
            radius: 0.225,
            zoom: 3.05,
            density: 1.0,
            live: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Wave {
    frequency: f32,
    depth: f32,
    offset: f32,
    amplitude: f32,
}

/// A repeatable field of coherent curves. Neighboring strokes share the same waves.
#[derive(Clone, Debug)]
pub struct Pattern([Wave; 3]);

impl Pattern {
    pub fn from_seed(seed: u32) -> Self {
        if seed == 0 {
            return Self([
                Wave {
                    frequency: 13.0,
                    depth: 4.0,
                    offset: 0.0,
                    amplitude: 0.092,
                },
                Wave {
                    frequency: 22.0,
                    depth: -5.0,
                    offset: 0.0,
                    amplitude: 0.030,
                },
                Wave {
                    frequency: 0.0,
                    depth: 0.0,
                    offset: 0.0,
                    amplitude: 0.0,
                },
            ]);
        }
        let mut state = seed;
        let mut next = || {
            // Xorshift32: an identical seed produces the same wave parameters.
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state >> 8) as f32 / 16_777_216.0
        };
        Self(std::array::from_fn(|i| Wave {
            frequency: [7.0, 16.0, 26.0][i] + 8.0 * next(),
            depth: (4.0 + next() * 4.0) * if i == 1 { -1.0 } else { 1.0 },
            offset: next() * std::f32::consts::TAU,
            amplitude: [0.068, 0.023, 0.008][i] * (0.72 + 0.54 * next()),
        }))
    }

    /// Vertical coordinate of a stroke at normalized coordinates x and depth.
    /// A phase of zero gives the still engraving. The phase repeats every full turn.
    pub fn height(&self, x: f32, depth: f32, phase: f32) -> f32 {
        self.at_phase(phase).sample(x, depth)
    }

    fn at_phase(&self, phase: f32) -> Self {
        Self(std::array::from_fn(|i| {
            let mut wave = self.0[i];
            wave.offset += (phase * (i + 1) as f32).sin() * 0.78;
            wave
        }))
    }

    fn sample(&self, x: f32, depth: f32) -> f32 {
        let envelope = (std::f32::consts::PI * x).sin().max(0.0).powf(1.25);
        let wave: f32 = self
            .0
            .iter()
            .map(|wave| {
                (x * wave.frequency + depth * wave.depth + wave.offset).sin() * wave.amplitude
            })
            .sum();
        0.045 + 0.91 * depth + wave * envelope
    }
}

/// Gaussian lens. Strong zooms use two bounded passes so the radial map stays monotone.
pub fn lens_offset(offset: Vec2, radius: f32, zoom: f32) -> Vec2 {
    if offset == Vec2::ZERO
        || radius <= 0.0
        || !radius.is_finite()
        || zoom <= 0.0
        || !zoom.is_finite()
    {
        return offset;
    }
    let zoom = zoom.clamp(0.32, 5.5);
    const CLASSIC_PEAK: f32 = 3.05;
    let primary = zoom.min(CLASSIC_PEAK);
    let pass = |offset: Vec2, gain: f32| {
        let distance_sq = (offset / radius).length_sq();
        offset * (1.0 + (gain - 1.0) * (-distance_sq).exp())
    };
    let warped = pass(offset, primary);
    if zoom <= CLASSIC_PEAK {
        warped
    } else {
        pass(warped, zoom / CLASSIC_PEAK)
    }
}

pub struct View {
    settings: Settings,
    pattern: Pattern,
    center: Pos2,
    strength: f32,
    phase: f32,
}

impl Default for View {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}

impl View {
    pub fn new(settings: Settings) -> Self {
        let mut view = Self {
            settings: Settings::default(),
            pattern: Pattern::from_seed(0),
            center: Pos2::ZERO,
            strength: 0.0,
            phase: 0.0,
        };
        view.set_settings(settings);
        view
    }

    pub fn set_settings(&mut self, settings: Settings) {
        let previous = self.settings;
        if previous.seed != settings.seed {
            self.pattern = Pattern::from_seed(settings.seed);
        }
        let bounded = |value: f32, fallback, min, max| {
            if value.is_finite() {
                value.clamp(min, max)
            } else {
                fallback
            }
        };
        self.settings = Settings {
            radius: bounded(settings.radius, previous.radius, 0.04, 0.55),
            zoom: bounded(settings.zoom, previous.zoom, 0.32, 5.5),
            density: bounded(settings.density, previous.density, 0.5, 2.0),
            ..settings
        };
    }

    pub fn lens_state(&self) -> (Pos2, f32) {
        (self.center, self.strength)
    }

    pub fn show(&mut self, ui: &mut egui::Ui, size: Vec2) -> Response {
        let (id, rect) = ui.allocate_space(size);
        self.show_in(ui, rect, id)
    }

    pub fn show_in(&mut self, ui: &mut egui::Ui, canvas: Rect, id: Id) -> Response {
        let response = ui
            .interact(canvas, id, Sense::hover())
            .on_hover_cursor(CursorIcon::Crosshair);
        if !ui.is_rect_visible(canvas) || canvas.width() <= 36.0 || canvas.height() <= 36.0 {
            return response;
        }
        let dt = ui.input(|input| input.stable_dt).clamp(0.0, 0.06);
        if self.settings.live {
            self.phase = (self.phase + dt * 0.58).rem_euclid(std::f32::consts::TAU);
        }
        let pointer = response.hover_pos();
        let destination = pointer.or_else(|| {
            self.settings.live.then(|| {
                canvas.center()
                    + canvas.size()
                        * vec2(
                            self.phase.sin() * 0.31,
                            (self.phase * 2.0 + 0.45).sin() * 0.24,
                        )
            })
        });
        if let Some(point) = destination {
            self.center = if self.strength < 0.001 {
                point
            } else {
                self.center.lerp(point, 1.0 - (-18.0 * dt).exp())
            };
        }
        let target = if destination.is_some() { 1.0 } else { 0.0 };
        self.strength += (target - self.strength) * (1.0 - (-11.0 * dt).exp());
        if (target - self.strength).abs() < 0.002 {
            self.strength = target;
        }
        if self.settings.live
            || self.strength != target
            || destination.is_some_and(|pos| pos.distance(self.center) > 0.15)
        {
            ui.ctx().request_repaint_after_secs(1.0 / 60.0);
        }
        self.paint(ui, canvas);
        response
    }

    fn paint(&self, ui: &egui::Ui, canvas: Rect) {
        let painter = ui.painter_at(canvas);
        let mut paper = egui::Mesh::default();
        let top = Color32::from_rgb(248, 244, 230);
        let bottom = Color32::from_rgb(231, 222, 201);
        for (pos, color) in [
            (canvas.left_top(), top),
            (canvas.right_top(), top),
            (canvas.right_bottom(), bottom),
            (canvas.left_bottom(), bottom),
        ] {
            paper.colored_vertex(pos, color);
        }
        paper.add_triangle(0, 1, 2);
        paper.add_triangle(0, 2, 3);
        painter.add(Shape::mesh(paper));
        let print = canvas.shrink(18.0);
        let plate = painter.with_clip_rect(print);
        let rows = (print.height() / 7.2 * self.settings.density)
            .round()
            .clamp(16.0, 128.0) as usize;
        let radius = print.height() * self.settings.radius;
        let columns = (print.width() / (radius * 0.17).clamp(2.0, 8.0))
            .round()
            .clamp(40.0, 480.0) as usize;
        let zoom = 1.0 + (self.settings.zoom - 1.0) * self.strength;
        let pattern = self.pattern.at_phase(self.phase);
        for row in 0..=rows {
            let depth = row as f32 / rows as f32;
            let points = (0..=columns)
                .map(|column| {
                    let x = column as f32 / columns as f32;
                    let y = pattern.sample(x, depth);
                    let point = pos2(
                        print.left() + x * print.width(),
                        print.top() + y * print.height(),
                    );
                    if self.strength == 0.0 {
                        point
                    } else {
                        self.center + lens_offset(point - self.center, radius, zoom)
                    }
                })
                .collect();
            let stroke = if row % 5 == 0 {
                Stroke::new(1.18, Color32::from_rgb(37, 38, 31))
            } else {
                Stroke::new(0.66, Color32::from_rgb(105, 101, 86))
            };
            plate.add(Shape::line(points, stroke));
        }
        painter.rect_stroke(
            print,
            0,
            Stroke::new(1.0, Color32::from_rgb(151, 141, 119)),
            StrokeKind::Inside,
        );
        painter.rect_stroke(
            canvas,
            0,
            Stroke::new(1.0, Color32::from_rgb(205, 202, 183)),
            StrokeKind::Inside,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_repeats_and_phase_changes_the_engraving() {
        let heights = |seed, phase| {
            std::array::from_fn::<_, 12, _>(|i| {
                Pattern::from_seed(seed).height(i as f32 / 13.0, 0.4, phase)
            })
        };
        assert_eq!(heights(42, 0.0), heights(42, 0.0));
        assert_ne!(heights(42, 0.0), heights(43, 0.0));
        assert_ne!(heights(42, 0.0), heights(42, 0.5));
    }

    #[test]
    fn default_lens_matches_the_original_gaussian_profile() {
        for distance in [0.0_f32, 2.0, 8.0, 15.0, 26.0, 60.0] {
            let offset = vec2(distance, 0.0);
            let radius = 15.0;
            let original = offset * (1.0 + 2.05 * (-(distance / radius).powi(2)).exp());
            assert!(
                (lens_offset(offset, radius, Settings::default().zoom) - original).length() < 1e-5
            );
        }
    }

    #[test]
    fn lens_preserves_radial_order_even_at_maximum_zoom() {
        for zoom in [0.32, 1.0, 3.05, 4.5, 5.5] {
            let mut previous = 0.0;
            for step in 1..=700 {
                let radius = step as f32 * 0.25;
                let mapped = lens_offset(vec2(radius, 0.0), 15.0, zoom).x;
                assert!(mapped > previous, "{zoom}× at radius {radius}");
                previous = mapped;
            }
        }
        assert!(lens_offset(vec2(3.0, 0.0), 15.0, 4.0).x > 3.0);
        assert!(lens_offset(vec2(3.0, 0.0), 15.0, 0.4).x < 3.0);
        assert!(
            lens_offset(vec2(3.0, 0.0), 10.0, 4.0).x < lens_offset(vec2(3.0, 0.0), 20.0, 4.0).x
        );
    }

    #[test]
    fn live_view_moves_the_lens_without_pointer_input() {
        let mut view = View::new(Settings {
            live: true,
            ..Settings::default()
        });
        let context = egui::Context::default();
        let mut draw = |time| {
            context
                .run_ui(
                    egui::RawInput {
                        time: Some(time),
                        ..Default::default()
                    },
                    |ui| {
                        view.show(ui, vec2(300.0, 220.0));
                    },
                )
                .textures_delta
                .clear();
            view.lens_state()
        };
        let (first, initial_strength) = draw(0.0);
        let (later, strength) = draw(0.06);
        assert!(strength > initial_strength);
        assert!(first.distance(later) > 0.1);
    }
}
