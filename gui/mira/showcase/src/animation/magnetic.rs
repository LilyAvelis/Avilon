use std::f32::consts::TAU;

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Painter, Pos2, Rect, Sense, Shape, Stroke,
    StrokeKind, Vec2, vec2,
};

use super::{halo, vertical_gradient};

const NORTH: Color32 = Color32::from_rgb(248, 158, 98);
const SOUTH: Color32 = Color32::from_rgb(105, 193, 244);
const POLE_OFFSET: f32 = 0.056;

struct Magnet {
    position: Vec2,
    north_on_right: bool,
}

pub(super) struct Magnetic {
    magnets: [Magnet; 2],
    time: f32,
    density: usize,
    density_slider: Option<egui::Id>,
}

impl Default for Magnetic {
    fn default() -> Self {
        Self {
            magnets: [
                Magnet {
                    position: vec2(0.36, 0.46),
                    north_on_right: true,
                },
                Magnet {
                    position: vec2(0.64, 0.54),
                    north_on_right: true,
                },
            ],
            time: 0.0,
            density: 14,
            density_slider: None,
        }
    }
}

impl Magnetic {
    pub(super) fn density_has_focus(&self, ui: &egui::Ui) -> bool {
        self.density_slider
            .is_some_and(|id| ui.memory(|memory| memory.has_focus(id)))
    }

    pub(super) fn show_density(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.spacing_mut().slider_width = 118.0;
                let label = ui.label("Густота линий");
                let slider = ui
                    .add(egui::Slider::new(&mut self.density, 6..=42).show_value(false))
                    .labelled_by(label.id);
                self.density_slider = Some(slider.id);
                slider.on_hover_text("Количество линий на магнит");
                ui.monospace(format!("{:02}", self.density));
            },
        );
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, canvas: Rect, dt: f32) {
        let size = canvas.size();
        let scale = size.y.min(size.x);
        if scale <= 0.0 {
            return;
        }
        let extent = size / scale;
        let screen = |point: Vec2| canvas.min + point * scale;
        let magnet_size = scale * vec2(0.164, 0.065);
        for (index, magnet) in self.magnets.iter_mut().enumerate() {
            let center = canvas.min + size * magnet.position;
            let hit = Rect::from_center_size(center, magnet_size).expand(9.0);
            let response = ui
                .interact(
                    hit,
                    ui.make_persistent_id(("magnet", index)),
                    Sense::click_and_drag(),
                )
                .on_hover_cursor(CursorIcon::Grab);
            if response.dragged_by(egui::PointerButton::Primary) {
                let margin = (magnet_size * 0.5 + Vec2::splat(8.0)) / size;
                magnet.position = (magnet.position + response.drag_delta() / size)
                    .clamp(margin, Vec2::splat(1.0) - margin);
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
            if response.secondary_clicked() {
                magnet.north_on_right = !magnet.north_on_right;
            }
        }
        self.time = (self.time + dt * 24.0) % 4096.0;

        let p = ui.painter_at(canvas);
        vertical_gradient(
            &p,
            canvas,
            Color32::from_rgb(14, 25, 35),
            Color32::from_rgb(11, 36, 44),
        );
        let border = canvas.shrink(12.0);
        p.rect_stroke(
            border,
            0,
            Stroke::new(1.0, Color32::from_rgb(46, 70, 80)),
            StrokeKind::Inside,
        );
        let bounds = Rect::from_min_size(Pos2::ZERO, extent);
        let centers = self
            .magnets
            .each_ref()
            .map(|magnet| magnet.position * extent);
        let poles = std::array::from_fn(|index| {
            let magnet = &self.magnets[index / 2];
            let sign = if magnet.north_on_right == (index % 2 == 0) {
                1.0
            } else {
                -1.0
            };
            (
                centers[index / 2] + vec2(sign * POLE_OFFSET, 0.0),
                if index % 2 == 0 { 1.0 } else { -1.0 },
            )
        });

        let step = 12.0 * (14.0 / self.density as f32).sqrt();
        for row in 1..=(size.y / step) as usize {
            for column in 1..=(size.x / step) as usize {
                let x = column as f32;
                let y = row as f32;
                let pixel = vec2(
                    x * step + (y * 3.1 + x * 2.37).sin() * 2.1,
                    y * step + (x * 2.9 - y * 3.2).cos() * 2.1,
                );
                let position = canvas.min + pixel;
                if !border.contains(position) {
                    continue;
                }
                let vector = field(pixel / scale, &poles);
                let strength = (vector.length() / 48.0).min(1.0).sqrt();
                let half = vector.normalized() * (1.25 + strength * 3.35);
                let ink = Color32::from_rgb(104, 192, 186).gamma_multiply(0.24 + strength * 0.52);
                p.line_segment([position - half, position + half], Stroke::new(0.85, ink));
            }
        }
        for north in [0, 2] {
            for ray in 0..self.density {
                let angle = (ray as f32 + 0.5) * TAU / self.density as f32;
                let (sin, cos) = angle.sin_cos();
                let mut point = poles[north].0 + vec2(cos, sin) * 0.021;
                let mut line = Vec::with_capacity(170);
                for _ in 0..170 {
                    if !bounds.contains(Pos2::ZERO + point)
                        || [1, 3]
                            .iter()
                            .any(|&south| (poles[south].0 - point).length_sq() < 0.00035)
                    {
                        break;
                    }
                    line.push(screen(point));
                    let direction = field(point, &poles).normalized();
                    let midway = point + direction * 0.0035;
                    point += field(midway, &poles).normalized() * 0.007;
                }
                if line.len() >= 2 {
                    let moving = (self.time + ray as f32 * 9.7 + north as f32 * 23.0) as usize
                        % (line.len() + 20);
                    if let Some(&position) = line.get(moving) {
                        p.circle_filled(position, 1.75, Color32::from_rgb(240, 212, 142));
                    }
                    p.add(Shape::line(
                        line,
                        Stroke::new(0.8, Color32::from_rgba_unmultiplied(121, 225, 210, 72)),
                    ));
                }
            }
        }
        for (center, magnet) in centers.into_iter().zip(&self.magnets) {
            draw_magnet(&p, screen(center), magnet_size, magnet.north_on_right);
        }
        ui.ctx().request_repaint();
    }
}

// A planar slice of two softened pole pairs. Falloff and direction come from the summed field.
fn field(point: Vec2, poles: &[(Vec2, f32); 4]) -> Vec2 {
    poles.iter().fold(Vec2::ZERO, |field, &(pole, sign)| {
        let delta = point - pole;
        let r_squared = delta.length_sq() + 0.0005;
        field + delta * (sign / (r_squared * r_squared.sqrt()))
    })
}

fn draw_magnet(p: &Painter, center: Pos2, size: Vec2, north_on_right: bool) {
    let frame = Rect::from_center_size(center, size);
    let p_left = Rect::from_min_max(frame.min, frame.center_bottom());
    let p_right = Rect::from_min_max(frame.center_top(), frame.max);
    let faces = if north_on_right {
        [(SOUTH, "S"), (NORTH, "N")]
    } else {
        [(NORTH, "N"), (SOUTH, "S")]
    };
    p.rect_filled(
        frame.translate(vec2(0.0, 4.0)).expand(2.0),
        3,
        Color32::from_black_alpha(140),
    );
    for (surface, (ink, label)) in [p_left, p_right].into_iter().zip(faces) {
        halo(
            p,
            surface.center(),
            size * vec2(0.77, 1.45),
            ink.gamma_multiply(0.21),
        );
        vertical_gradient(
            p,
            surface,
            ink.gamma_multiply(0.88),
            ink.gamma_multiply(0.51),
        );
        p.text(
            surface.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::monospace((size.y * 0.55).max(10.0)),
            Color32::from_rgb(17, 29, 34),
        );
    }
    p.rect_stroke(
        frame,
        2,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(231, 245, 240, 116)),
        StrokeKind::Inside,
    );
    p.line_segment(
        [frame.center_top(), frame.center_bottom()],
        Stroke::new(1.2, Color32::from_rgb(40, 55, 60)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_slider_changes_the_number_of_field_lines() {
        let ctx = egui::Context::default();
        let mut magnetic = Magnetic::default();
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(245.0, 28.0));
        ctx.run_ui(egui::RawInput::default(), |ui| {
            magnetic.show_density(ui, rect);
        })
        .textures_delta
        .clear();
        let id = magnetic.density_slider.expect("slider was registered");
        let track = ctx.read_response(id).expect("slider was drawn").rect;
        let pointer = track.right_center() - vec2(1.0, 0.0);
        let click = |pressed| egui::Event::PointerButton {
            pos: pointer,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        for events in [
            vec![egui::Event::PointerMoved(pointer)],
            vec![click(true)],
            vec![click(false)],
        ] {
            ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| magnetic.show_density(ui, rect),
            )
            .textures_delta
            .clear();
        }
        assert_eq!(magnetic.density, 42);
    }

    #[test]
    fn secondary_click_flips_and_primary_drag_moves_only_the_selected_magnet() {
        let ctx = egui::Context::default();
        let mut magnetic = Magnetic::default();
        let canvas = Rect::from_min_size(Pos2::ZERO, vec2(840.0, 420.0));
        let center = canvas.min + canvas.size() * magnetic.magnets[0].position;
        let target = center + vec2(84.0, -35.0);
        let button = |pos, button, pressed| egui::Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        use egui::PointerButton::{Primary, Secondary};
        let frames = [
            vec![egui::Event::PointerMoved(center)],
            vec![button(center, Secondary, true)],
            vec![button(center, Secondary, false)],
            vec![button(center, Primary, true)],
            vec![egui::Event::PointerMoved(target)],
            vec![button(target, Primary, false)],
        ];
        for (index, events) in frames.into_iter().enumerate() {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(canvas),
                    time: Some(index as f64 * 0.04),
                    events,
                    ..Default::default()
                },
                |ui| magnetic.show(ui, canvas, 0.016),
            )
            .textures_delta
            .clear();
        }
        assert!(!magnetic.magnets[0].north_on_right);
        assert!(
            (magnetic.magnets[0].position - (target - canvas.min) / canvas.size()).length() < 0.001
        );
        assert!(magnetic.magnets[1].north_on_right);
        assert_eq!(magnetic.magnets[1].position, vec2(0.64, 0.54));
    }
}
