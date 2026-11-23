use super::{Form, Light, Point3, add, cross, dot, normalized, scale, sub};
use egui::{
    Color32, CursorIcon, Id, Mesh, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
    vec2,
};

/// Interactive rotation and lighting for a validated form. Keep one instance per preview.
pub struct View {
    form: Form,
    finish: Light,
    tint: Color32,
    yaw: f32,
    pitch: f32,
    elapsed: f32,
    spin_speed: f32,
    spinning: bool,
}

impl View {
    const DEFAULT_ANGLES: [f32; 2] = [0.36, 0.28];
    pub const DEFAULT_TINT: Color32 = Color32::from_rgb(39, 216, 208);

    pub fn new(form: Form) -> Self {
        let [yaw, pitch] = Self::DEFAULT_ANGLES;
        Self {
            form,
            finish: Light::Studio,
            tint: Self::DEFAULT_TINT,
            yaw,
            pitch,
            elapsed: 0.0,
            spin_speed: 1.0,
            spinning: true,
        }
    }

    pub fn with_light(mut self, light: Light) -> Self {
        self.set_light(light);
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.set_color(color);
        self
    }

    pub fn with_spin_speed(mut self, speed: f32) -> Self {
        self.set_spin_speed(speed);
        self
    }

    pub fn set_form(&mut self, form: Form) {
        self.form = form;
    }
    pub fn set_light(&mut self, light: Light) {
        self.finish = light;
    }
    pub fn set_color(&mut self, tint: Color32) {
        self.tint = tint;
    }

    /// A signed multiplier of the default angular speed. Zero stops automatic rotation.
    pub fn set_spin_speed(&mut self, speed: f32) {
        if speed.is_finite() {
            self.spin_speed = speed;
        }
    }

    /// Pauses rotation while leaving drag and animated lighting available.
    pub fn set_spinning(&mut self, spinning: bool) {
        self.spinning = spinning;
    }
    pub fn is_spinning(&self) -> bool {
        self.spinning
    }
    pub fn angles(&self) -> [f32; 2] {
        [self.yaw, self.pitch]
    }
    pub fn reset_angles(&mut self) {
        [self.yaw, self.pitch] = Self::DEFAULT_ANGLES;
    }

    /// Allocates a transparent preview with its own drag target.
    pub fn show(&mut self, ui: &mut Ui, size: Vec2) -> Response {
        let (id, rect) = ui.allocate_space(size);
        self.show_in(ui, rect, id)
    }

    /// Uses an existing rectangle, for example a scene with a host-drawn background.
    pub fn show_in(&mut self, ui: &mut Ui, rect: Rect, id: Id) -> Response {
        let response = ui
            .interact(rect, id, Sense::drag())
            .on_hover_cursor(CursorIcon::Grab);
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let dt = ui.input(|input| input.stable_dt).clamp(0.0, 0.06);
        let auto_spin = self.spinning && self.spin_speed != 0.0;
        if response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            let delta = response.drag_delta();
            self.yaw += delta.x * 0.012;
            self.pitch = (self.pitch + delta.y * 0.012).clamp(-1.45, 1.45);
        } else if auto_spin && !response.is_pointer_button_down_on() {
            self.yaw += dt * 0.62 * self.spin_speed;
        }
        if auto_spin || matches!(self.finish, Light::Sweep | Light::Wave) {
            self.elapsed += dt;
            ui.ctx().request_repaint_after_secs(1.0 / 60.0);
        }
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);
        self.paint(
            &ui.painter_at(rect),
            rect.center(),
            rect.width().min(rect.height()) * 1.23,
        );
        response
    }

    /// Paints the solid at the current angles. The caller provides the background and clipping.
    pub fn paint(&self, p: &Painter, center: Pos2, focal: f32) {
        if !focal.is_finite() || focal <= 0.0 {
            return;
        }
        let eye = [0.0, 0.0, 4.8];
        let (sy, cy) = self.yaw.sin_cos();
        let (sx, cx) = self.pitch.sin_cos();
        let rotated = |[x, y, z]: Point3| {
            let x1 = x * cy + z * sy;
            let z1 = z * cy - x * sy;
            [x1, y * cx - z1 * sx, y * sx + z1 * cx]
        };
        let key_position = if self.finish == Light::Sweep {
            let (sin, cos) = (self.elapsed * 2.2).sin_cos();
            [3.5 * cos, 2.1, 3.5 * sin + 1.2]
        } else {
            [-2.4, 3.1, 3.2]
        };
        let surface = |vertices: &[Point3], preset: Color32| {
            let pigment = if self.tint == Self::DEFAULT_TINT {
                preset
            } else {
                preset.lerp_to_gamma(self.tint, 0.78)
            };
            let midpoint = vertices.iter().fold([0.0; 3], |sum, &p| add(sum, p));
            let midpoint = scale(midpoint, 1.0 / vertices.len() as f32);
            let normal = normalized(cross(
                sub(vertices[1], vertices[0]),
                sub(vertices[2], vertices[0]),
            ));
            if dot(normal, sub(eye, midpoint)) <= 0.0 {
                return;
            }
            let mut mesh = Mesh::default();
            let mut outline = Vec::with_capacity(vertices.len());
            for &vertex in vertices {
                let view = normalized(sub(eye, vertex));
                let key = normalized(sub(key_position, vertex));
                let side = normalized(sub([3.2, -0.5, 2.0], vertex));
                let diffuse = dot(normal, key).max(0.0);
                let reflected = dot(normal, normalized(add(key, view))).max(0.0).powi(12);
                let rim = dot(normal, side).max(0.0);
                let facing = dot(normal, view).max(0.0);
                let fresnel = (1.0 - facing).powi(3);
                let base = Color32::from_rgb(10, 20, 43)
                    .lerp_to_gamma(pigment, 0.30 + diffuse * 0.54)
                    .lerp_to_gamma(Color32::from_rgb(136, 119, 249), rim * 0.22)
                    .lerp_to_gamma(
                        Color32::from_rgb(220, 255, 247),
                        (reflected * 0.84 + fresnel * 0.25).min(1.0),
                    );
                let color = match self.finish {
                    Light::Pearl => {
                        let phase = 7.0 * (1.0 - facing);
                        let channel = |offset: f32| (142.0 + 108.0 * (phase + offset).cos()) as u8;
                        base.lerp_to_gamma(
                            Color32::from_rgb(channel(0.0), channel(2.1), channel(4.2)),
                            0.56,
                        )
                    }
                    Light::Neon => base.gamma_multiply(0.48),
                    Light::Wave => {
                        let height = (self.elapsed * 2.0).sin() * 1.08;
                        let distance = (vertex[1] + 0.32 * vertex[0] - height) / 0.48;
                        base.lerp_to_gamma(
                            Color32::from_rgb(231, 250, 255),
                            0.79 * (-distance * distance).exp(),
                        )
                    }
                    Light::Studio | Light::Sweep => base,
                };
                let perspective = focal / (eye[2] - vertex[2]);
                let point = center + vec2(vertex[0], -vertex[1]) * perspective;
                mesh.colored_vertex(point, color);
                outline.push(point);
            }
            for corner in 1..vertices.len() as u32 - 1 {
                mesh.add_triangle(0, corner, corner + 1);
            }
            p.add(Shape::mesh(mesh));
            let neon = self.finish == Light::Neon;
            let edge = if neon {
                Stroke::new(1.45, self.tint.lerp_to_gamma(Color32::WHITE, 0.30))
            } else {
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(155, 236, 239, 102))
            };
            for (index, &point) in outline.iter().enumerate() {
                let segment = [point, outline[(index + 1) % outline.len()]];
                if neon {
                    p.line_segment(segment, Stroke::new(5.5, self.tint.gamma_multiply(0.22)));
                }
                p.line_segment(segment, edge);
            }
        };
        for (index, face) in self.form.faces.iter().enumerate() {
            let vertices: Vec<_> = face
                .iter()
                .map(|&corner| rotated(self.form.vertices[corner]))
                .collect();
            surface(&vertices, self.form.pigments[index]);
        }
    }
}
