use std::f32::consts::TAU;
use std::sync::LazyLock;

use eframe::egui::{
    self, Color32, CursorIcon, Mesh, Painter, Pos2, Rect, Sense, Shape, Stroke, Vec2, pos2, vec2,
};

const ANGLE_COUNT: usize = 160;
static ANGLES: LazyLock<[(f32, f32); ANGLE_COUNT]> = LazyLock::new(|| {
    std::array::from_fn(|index| (index as f32 * TAU / ANGLE_COUNT as f32).sin_cos())
});

#[derive(Default)]
pub(super) struct Moire {
    elapsed: f32,
    shift: Vec2,
    settled_shift: Vec2,
    spacing_bias: f32,
    settled_spacing: f32,
}

impl Moire {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, canvas: Rect, dt: f32) {
        if canvas.width() <= 0.0 || canvas.height() <= 0.0 {
            return;
        }
        let response = ui
            .interact(
                canvas,
                ui.make_persistent_id("moire-grating"),
                Sense::click_and_drag(),
            )
            .on_hover_cursor(CursorIcon::Grab);
        if response.dragged() {
            let delta = response.drag_delta() / canvas.size();
            self.shift = (self.shift + delta).clamp(vec2(-0.36, -0.36), vec2(0.30, 0.33));
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }
        if response.hovered() {
            let wheel = ui.input_mut(|input| {
                input.smooth_scroll_delta.y = 0.0;
                input
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        egui::Event::MouseWheel { delta, .. } => Some(delta.y),
                        _ => None,
                    })
                    .sum::<f32>()
            });
            self.spacing_bias = (self.spacing_bias + wheel * 0.0017).clamp(-0.26, 0.32);
        }
        if response.double_clicked() {
            self.shift = Vec2::ZERO;
            self.spacing_bias = 0.0;
        }

        let dt = dt.clamp(0.0, 0.05);
        self.elapsed += dt;
        let blend = 1.0 - (-dt * 14.0).exp();
        self.settled_shift += (self.shift - self.settled_shift) * blend;
        self.settled_spacing += (self.spacing_bias - self.settled_spacing) * blend;

        let p = ui.painter_at(canvas);
        stage(&p, canvas);
        let span = canvas.height().min(canvas.width() * 0.56);
        let t = self.elapsed;
        let center = canvas.center();
        let fixed = center + vec2(-0.101 * canvas.width(), 0.022 * canvas.height());
        let drift = vec2((t * 0.59).sin(), (t * 0.83 + 0.82).sin()) * span * 0.024;
        let moving = center
            + canvas.size() * vec2(0.109, -0.045)
            + canvas.size() * self.settled_shift
            + drift;
        let pitch = (span * 0.0195).clamp(5.4, 10.0);
        let radius = span * 1.06;
        rings(
            &p,
            Plate {
                origin: fixed,
                pitch,
                extent: radius,
                phase: -t * 0.19,
                tilt: 0.09,
                inner: Color32::from_rgb(109, 244, 223),
                outer: Color32::from_rgb(54, 104, 236),
            },
        );
        rings(
            &p,
            Plate {
                origin: moving,
                pitch: pitch * (1.056 + self.settled_spacing),
                extent: radius,
                phase: t * 0.24 + 0.37,
                tilt: -0.28 + (t * 0.61).sin() * 0.065,
                inner: Color32::from_rgb(255, 206, 143),
                outer: Color32::from_rgb(209, 107, 252),
            },
        );
        center_mark(&p, fixed, Color32::from_rgb(157, 243, 223));
        center_mark(&p, moving, Color32::from_rgb(255, 221, 175));

        ui.ctx().request_repaint();
    }
}

struct Plate {
    origin: Pos2,
    pitch: f32,
    extent: f32,
    phase: f32,
    tilt: f32,
    inner: Color32,
    outer: Color32,
}

fn rings(p: &Painter, plate: Plate) {
    let (sin_tilt, cos_tilt) = plate.tilt.sin_cos();
    let steps = (plate.extent / plate.pitch).ceil() as usize;
    let fade_in = plate.phase.rem_euclid(1.0);
    for step in 0..steps {
        let radius = (step as f32 + fade_in) * plate.pitch;
        if radius < 0.1 || radius >= plate.extent {
            continue;
        }
        let r = (radius / plate.extent).clamp(0.0, 1.0);
        let intensity = (1.0 - r * r).powf(1.2) * (r * 19.0).min(1.0);
        let width = 0.65 + (1.0 - r) * 0.7;
        let (sin_phase, cos_phase) = (radius * 0.016).sin_cos();
        let mut points = Vec::with_capacity(ANGLE_COUNT);
        for &(sin, cos) in ANGLES.iter() {
            // A slight scallop keeps a stable beat even away from the center.
            let offset = (cos * cos_phase + sin * sin_phase) * 0.022;
            let x = radius * cos * (1.12 + offset);
            let y = radius * sin * (0.84 - offset);
            points.push(
                plate.origin + vec2(x * cos_tilt - y * sin_tilt, x * sin_tilt + y * cos_tilt),
            );
        }
        let tone = shade(plate.inner, plate.outer, r);
        p.add(Shape::closed_line(
            points,
            Stroke::new(width, tone.gamma_multiply(intensity * 0.70)),
        ));
    }
}

fn shade(from: Color32, to: Color32, t: f32) -> Color32 {
    let channel = |a: u8, b: u8| egui::lerp(a as f32..=b as f32, t).round() as u8;
    Color32::from_rgb(
        channel(from.r(), to.r()),
        channel(from.g(), to.g()),
        channel(from.b(), to.b()),
    )
}

fn stage(p: &Painter, rect: Rect) {
    let mut backdrop = Mesh::default();
    let pool = rect.center() + vec2(-0.03 * rect.width(), 0.02 * rect.height());
    for (point, color) in [
        (rect.left_top(), Color32::from_rgb(11, 21, 37)),
        (rect.right_top(), Color32::from_rgb(24, 15, 37)),
        (rect.right_bottom(), Color32::from_rgb(28, 15, 36)),
        (rect.left_bottom(), Color32::from_rgb(9, 16, 31)),
        (pool, Color32::from_rgb(23, 29, 50)),
    ] {
        backdrop.colored_vertex(point, color);
    }
    for (i, j) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
        backdrop.add_triangle(i, j, 4);
    }
    p.add(Shape::mesh(backdrop));

    let line = Color32::from_rgb(58, 64, 85);
    let margin = rect.shrink(15.0);
    for (point, dx, dy) in [
        (margin.left_top(), 1.0, 1.0),
        (margin.right_top(), -1.0, 1.0),
        (margin.right_bottom(), -1.0, -1.0),
        (margin.left_bottom(), 1.0, -1.0),
    ] {
        p.line_segment(
            [point, point + vec2(dx * 17.0, 0.0)],
            Stroke::new(1.0, line),
        );
        p.line_segment(
            [point, point + vec2(0.0, dy * 17.0)],
            Stroke::new(1.0, line),
        );
    }
    p.hline(
        margin.x_range(),
        rect.center().y,
        Stroke::new(1.0, Color32::from_white_alpha(12)),
    );
    p.vline(
        rect.center().x,
        margin.y_range(),
        Stroke::new(1.0, Color32::from_white_alpha(12)),
    );
}

fn center_mark(p: &Painter, center: Pos2, light: Color32) {
    let halo = light.gamma_multiply(0.56);
    p.circle_filled(center, 6.3, Color32::from_rgb(19, 22, 39));
    p.circle_stroke(center, 6.3, Stroke::new(1.0, halo));
    p.line_segment(
        [
            pos2(center.x - 2.5, center.y),
            pos2(center.x + 2.5, center.y),
        ],
        Stroke::new(1.0, light),
    );
    p.line_segment(
        [
            pos2(center.x, center.y - 2.5),
            pos2(center.x, center.y + 2.5),
        ],
        Stroke::new(1.0, light),
    );
}
