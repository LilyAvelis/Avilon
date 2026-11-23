use eframe::egui::{
    self, Color32, CursorIcon, Mesh, Painter, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2,
    vec2,
};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 280.0;
const COLUMNS: usize = 22;
const ROWS: usize = 13;
const HORIZONTAL: f32 = 21.0;
const VERTICAL: f32 = 13.5;
const STEP: f32 = 1.0 / 120.0;
const PINS: [usize; 4] = [0, 7, 14, 21];

#[derive(Clone, Copy)]
struct Point {
    xy: Vec2,
    depth: f32,
}

impl Point {
    fn projected(self) -> Vec2 {
        self.xy + vec2(0.11, -0.23) * self.depth
    }
}

struct Particle {
    point: Point,
    previous: Point,
    pinned: bool,
}

struct Link {
    from: usize,
    to: usize,
    rest: f32,
    stiffness: f32,
}

struct Pull {
    index: usize,
    offset: Vec2,
    origin: Vec2,
    target: Vec2,
}

#[derive(Default)]
pub(super) struct Fabric {
    particles: Vec<Particle>,
    links: Vec<Link>,
    pull: Option<Pull>,
    accumulated: f32,
    settling: f32,
}

impl Fabric {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, canvas: Rect, dt: f32) {
        if self.particles.is_empty() {
            self.initialize();
        }
        let scale = (canvas.width() / WIDTH).min(canvas.height() / HEIGHT);
        let origin = canvas.center() - vec2(WIDTH, HEIGHT) * (0.5 * scale);
        let response = ui.interact(canvas, ui.make_persistent_id("fabric-pull"), Sense::drag());
        let hovered = response
            .hover_pos()
            .and_then(|position| self.nearest((position - origin) / scale));

        if response.drag_started()
            && let Some(position) = response.interact_pointer_pos()
        {
            let target = (position - origin) / scale;
            self.pull = self.nearest(target).map(|index| {
                let origin = self.particles[index].point.projected();
                Pull {
                    index,
                    offset: origin - target,
                    origin,
                    target: origin,
                }
            });
        }
        if response.dragged()
            && let Some(pull) = &mut self.pull
        {
            if let Some(position) = response.interact_pointer_pos() {
                let target = (position - origin) / scale + pull.offset;
                pull.target = target.clamp(vec2(36.0, 28.0), vec2(WIDTH - 36.0, HEIGHT - 16.0));
            }
            self.settling = 1.8;
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        } else {
            self.pull = None;
            if hovered.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }
        }

        if self.settling > 0.0 {
            let time = dt.clamp(0.0, 0.045);
            self.accumulated += time;
            while self.accumulated >= STEP {
                self.step();
                self.accumulated -= STEP;
            }
            self.settling -= time;
            ui.ctx().request_repaint_after_secs(1.0 / 60.0);
        }
        self.paint(&ui.painter_at(canvas), canvas, origin, scale, hovered);
    }

    fn initialize(&mut self) {
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let v = row as f32 / (ROWS - 1) as f32;
                let point = Point {
                    xy: vec2(
                        99.0 + column as f32 * HORIZONTAL,
                        49.0 + row as f32 * VERTICAL,
                    ),
                    depth: (column as f32 * 0.72).sin() * v * 9.0,
                };
                self.particles.push(Particle {
                    point,
                    previous: point,
                    pinned: row == 0 && PINS.contains(&column),
                });

                for (dx, dy, stiffness) in [
                    (1, 0, 0.90),
                    (0, 1, 0.90),
                    (1, 1, 0.54),
                    (-1, 1, 0.54),
                    (2, 0, 0.14),
                    (0, 2, 0.14),
                ] {
                    let next_column = column as isize + dx;
                    let next_row = row + dy;
                    if !(0..COLUMNS as isize).contains(&next_column) || next_row >= ROWS {
                        continue;
                    }
                    self.links.push(Link {
                        from: row * COLUMNS + column,
                        to: next_row * COLUMNS + next_column as usize,
                        rest: vec2(dx as f32 * HORIZONTAL, dy as f32 * VERTICAL).length(),
                        stiffness,
                    });
                }
            }
        }
        self.settling = 1.3;
    }

    fn nearest(&self, at: Vec2) -> Option<usize> {
        self.particles
            .iter()
            .enumerate()
            .filter(|(_, particle)| !particle.pinned)
            .map(|(index, particle)| (index, (particle.point.projected() - at).length_sq()))
            .filter(|(_, distance)| *distance < 28.0 * 28.0)
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
    }

    fn step(&mut self) {
        let grabbed = self.pull.as_ref().map(|pull| pull.index);
        for (index, particle) in self.particles.iter_mut().enumerate() {
            if particle.pinned {
                continue;
            }
            let old = particle.point;
            if let Some(pull) = self.pull.as_ref().filter(|pull| pull.index == index) {
                let stretch = (pull.target - pull.origin).length();
                let desired_depth = (20.0 + stretch * 0.21).min(72.0);
                particle.point.depth += (desired_depth - particle.point.depth) * 0.105;
                let target = pull.target - vec2(0.11, -0.23) * particle.point.depth;
                particle.point.xy += limited(target - particle.point.xy, 19.0);
            } else {
                let delta = limited(old.xy - particle.previous.xy, 15.0);
                particle.point.xy += delta * 0.985 + vec2(0.0, 540.0) * STEP * STEP;
                particle.point.depth += (old.depth - particle.previous.depth).clamp(-13.0, 13.0)
                    * 0.978
                    - old.depth * 16.0 * STEP * STEP;
            }
            particle.previous = old;
        }

        for iteration in 0..5 {
            for cursor in 0..self.links.len() {
                let link = &self.links[if iteration % 2 == 0 {
                    cursor
                } else {
                    self.links.len() - cursor - 1
                }];
                let from = &self.particles[link.from];
                let to = &self.particles[link.to];
                let from_weight = if from.pinned || grabbed == Some(link.from) {
                    0.0
                } else {
                    1.0
                };
                let to_weight = if to.pinned || grabbed == Some(link.to) {
                    0.0
                } else {
                    1.0
                };
                let total = from_weight + to_weight;
                if total == 0.0 {
                    continue;
                }
                let xy = to.point.xy - from.point.xy;
                let depth = to.point.depth - from.point.depth;
                let distance = (xy.length_sq() + depth * depth).sqrt();
                let correction =
                    link.stiffness * (distance - link.rest) / (distance.max(0.01) * total);
                let a = &mut self.particles[link.from].point;
                a.xy += xy * (correction * from_weight);
                a.depth += depth * (correction * from_weight);
                let b = &mut self.particles[link.to].point;
                b.xy -= xy * (correction * to_weight);
                b.depth -= depth * (correction * to_weight);
            }
        }
    }

    fn paint(&self, p: &Painter, canvas: Rect, origin: Pos2, scale: f32, hovered: Option<usize>) {
        let board = Color32::from_rgb(220, 222, 208);
        let rule = Color32::from_rgb(172, 184, 169);
        let metal = Color32::from_rgb(84, 103, 89);
        let warm = Color32::from_rgb(167, 91, 57);
        let at = |x, y| origin + vec2(x, y) * scale;
        let point = |index: usize| origin + self.particles[index].point.projected() * scale;
        p.rect_filled(canvas, 2, board);
        p.rect_stroke(canvas, 2, Stroke::new(1.0, rule), StrokeKind::Inside);

        for y in (22..HEIGHT as usize).step_by(15) {
            p.line_segment(
                [at(24.0, y as f32), at(WIDTH - 24.0, y as f32)],
                Stroke::new(0.65 * scale, rule.gamma_multiply(0.44)),
            );
        }
        for x in [59.0, WIDTH - 59.0] {
            p.line_segment(
                [at(x, 31.0), at(x, HEIGHT - 24.0)],
                Stroke::new(scale, rule),
            );
        }
        let rail = Rect::from_two_pos(at(76.0, 40.0), at(564.0, 49.0));
        p.rect_filled(
            rail.translate(vec2(0.0, 3.0) * scale),
            3,
            Color32::from_black_alpha(24),
        );
        p.rect_filled(rail, 3, metal);
        p.line_segment(
            [rail.left_top(), rail.right_top()],
            Stroke::new(scale, Color32::from_rgb(237, 231, 200)),
        );

        let mut shadow = Mesh::default();
        let mut cloth = Mesh::default();
        for index in 0..self.particles.len() {
            let pos = point(index);
            let depth = self.particles[index].point.depth.max(0.0);
            let shade_offset = vec2(6.0 + depth * 0.19, 10.0 + depth * 0.27) * scale;
            shadow.colored_vertex(pos + shade_offset, Color32::from_black_alpha(32));
            cloth.colored_vertex(pos, self.material(index));
        }
        for row in 0..ROWS - 1 {
            for col in 0..COLUMNS - 1 {
                let a = (row * COLUMNS + col) as u32;
                let b = a + 1;
                let c = a + COLUMNS as u32;
                let d = c + 1;
                shadow.add_triangle(a, b, c);
                shadow.add_triangle(b, d, c);
                cloth.add_triangle(a, b, c);
                cloth.add_triangle(b, d, c);
            }
        }
        p.add(Shape::mesh(shadow));
        p.add(Shape::mesh(cloth));

        let thread = Color32::from_rgba_unmultiplied(67, 87, 79, 72);
        for column in [2, 5, 8, 11, 14, 17, 19] {
            let warp = (0..ROWS).map(|row| point(row * COLUMNS + column)).collect();
            p.add(Shape::line(warp, Stroke::new(0.65 * scale, thread)));
        }
        for row in [2, 5, 8, 11] {
            let weft = (0..COLUMNS)
                .map(|column| point(row * COLUMNS + column))
                .collect();
            p.add(Shape::line(weft, Stroke::new(0.65 * scale, thread)));
        }
        let edge = (0..COLUMNS)
            .chain((1..ROWS).map(|row| row * COLUMNS + COLUMNS - 1))
            .chain(((ROWS - 1) * COLUMNS..ROWS * COLUMNS - 1).rev())
            .chain((1..ROWS - 1).rev().map(|row| row * COLUMNS))
            .map(point)
            .collect();
        p.add(Shape::closed_line(
            edge,
            Stroke::new(1.45 * scale, Color32::from_rgb(93, 105, 79)),
        ));
        for index in PINS {
            let pin = Rect::from_center_size(
                point(index) - vec2(0.0, 4.0) * scale,
                vec2(10.0, 15.0) * scale,
            );
            p.rect_filled(pin, 2, warm);
            p.rect_filled(
                pin.shrink2(vec2(2.0, 4.0) * scale),
                1,
                Color32::from_rgb(233, 174, 114),
            );
        }

        if let Some(index) = self.pull.as_ref().map(|pull| pull.index).or(hovered) {
            let pos = point(index);
            p.circle_stroke(
                pos,
                8.5 * scale,
                Stroke::new(1.6 * scale, Color32::from_rgb(246, 236, 195)),
            );
            p.circle_stroke(pos, 10.5 * scale, Stroke::new(scale, warm));
        }
    }

    fn material(&self, index: usize) -> Color32 {
        let row = index / COLUMNS;
        let col = index % COLUMNS;
        let at = |y: usize, x: usize| self.particles[y * COLUMNS + x].point;
        let left = at(row, col.saturating_sub(1));
        let right = at(row, (col + 1).min(COLUMNS - 1));
        let top = at(row.saturating_sub(1), col);
        let bottom = at((row + 1).min(ROWS - 1), col);
        let dx = right.xy - left.xy;
        let dy = bottom.xy - top.xy;
        let zx = right.depth - left.depth;
        let zy = bottom.depth - top.depth;
        let nx = dx.y * zy - zx * dy.y;
        let ny = zx * dy.x - dx.x * zy;
        let nz = dx.x * dy.y - dx.y * dy.x;
        let length = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
        let light = ((0.57 * nx - 0.43 * ny + 0.70 * nz) / length * 0.62 + 0.37).clamp(0.0, 1.0);

        let warp = (col + 2) % 7;
        let (dark, bright) = if row == 0 || row == ROWS - 1 || col == 0 || col == COLUMNS - 1 {
            ([89, 62, 46], [194, 113, 71])
        } else if [1, 4].contains(&warp) || [1, ROWS - 2].contains(&row) {
            ([60, 79, 80], [153, 173, 157])
        } else {
            ([100, 107, 84], [228, 224, 176])
        };
        Color32::from_rgb(dark[0], dark[1], dark[2])
            .lerp_to_gamma(Color32::from_rgb(bright[0], bright[1], bright[2]), light)
    }
}

fn limited(vector: Vec2, max: f32) -> Vec2 {
    vector * (max / vector.length().max(max))
}
