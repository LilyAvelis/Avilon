use eframe::egui::{
    self, Align2, Color32, FontId, Key, Mesh, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
    WidgetInfo, WidgetType, pos2, vec2,
};

use crate::graphics::{INK, MUTED, PAPER, RULE, vertical_gradient};
use crate::{SERIF, consume_key, install_fonts};

mod fabric;
mod magnetic;
mod moire;
mod pixel;
mod unity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Experiment {
    Engraving,
    Prism,
    Moire,
    Fabric,
    Pixel,
    Theatre,
    Magnetic,
}

const EXPERIMENTS: [Experiment; 7] = [
    Experiment::Engraving,
    Experiment::Prism,
    Experiment::Moire,
    Experiment::Fabric,
    Experiment::Pixel,
    Experiment::Theatre,
    Experiment::Magnetic,
];
const COMPLEXITY_LEVELS: usize = 6;

impl Experiment {
    fn title(self) -> &'static str {
        match self {
            Self::Engraving => "Живая гравюра",
            Self::Prism => "Объёмная призма",
            Self::Moire => "Муар",
            Self::Fabric => "Полотно на скобах",
            Self::Pixel => "Пиксельный прыжок",
            Self::Theatre => "Театр света",
            Self::Magnetic => "Магнитное поле",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::Engraving => "Проведи мышью по штрихам.",
            Self::Prism => "Перетащи мышью, чтобы повернуть призму.",
            Self::Moire => {
                "Перетащи тёплый слой. Колесо меняет шаг колец. Двойной клик сбрасывает."
            }
            Self::Fabric => "Возьми полотно мышью, потяни и отпусти.",
            Self::Pixel => "Щёлкни по площадке, чтобы прыгнуть к отметке.",
            Self::Theatre => {
                "Enter открывает сцену. В Bevy удерживай ЛКМ, чтобы двигать свет, и нажми пробел для паузы."
            }
            Self::Magnetic => "Перетаскивай магниты. Правый клик меняет полюса.",
        }
    }

    fn complexity(self) -> usize {
        match self {
            Self::Engraving => 2,
            Self::Prism => 3,
            Self::Moire => 3,
            Self::Fabric => 4,
            Self::Pixel => 2,
            Self::Theatre => 4,
            Self::Magnetic => 4,
        }
    }
}

#[derive(Default)]
pub(super) struct Animation {
    index: usize,
    engraving: Engraving,
    prism: Prism,
    moire: moire::Moire,
    fabric: fabric::Fabric,
    pixel: pixel::Pixel,
    magnetic: magnetic::Magnetic,
    theatre_error: Option<String>,
    unity: unity::Gallery,
}

impl Animation {
    fn advance(&mut self, delta: isize) {
        self.index = (self.index as isize + delta).rem_euclid(EXPERIMENTS.len() as isize) as usize;
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, heading: FontId) {
        let album_height = (ui.available_height() - 49.0).max(300.0);
        egui::ScrollArea::vertical()
            .id_salt("animation-page")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (album, _) = ui.allocate_exact_size(
                    vec2(ui.available_width(), album_height),
                    egui::Sense::hover(),
                );
                if ui.is_rect_visible(album) {
                    self.show_album(ui, album.shrink2(vec2(30.0, 14.0)), heading.clone());
                }

                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(30, 0))
                    .show(ui, |ui| {
                        let jump = ui
                            .allocate_ui_with_layout(
                                vec2(ui.available_width(), 24.0),
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| ui.link("Проекты Unity ↓").clicked(),
                            )
                            .inner;
                        ui.add_space(20.0);
                        self.unity.show(ui, heading, jump);
                        ui.add_space(26.0);
                    });
            });
    }

    fn show_album(&mut self, ui: &mut egui::Ui, outer: Rect, heading: FontId) {
        let ctx = ui.ctx();
        let adjusting = match EXPERIMENTS[self.index] {
            Experiment::Magnetic => self.magnetic.density_has_focus(ui),
            Experiment::Prism => self.prism.control_has_focus(ui),
            _ => false,
        };
        let delta = if adjusting {
            0
        } else {
            consume_key(ctx, Key::ArrowRight, false) as isize
                - consume_key(ctx, Key::ArrowLeft, false) as isize
        };
        if delta != 0 {
            self.advance(delta);
            ui.memory_mut(|memory| {
                if let Some(focused) = memory.focused() {
                    memory.surrender_focus(focused);
                }
            });
        }

        let footer = Rect::from_min_max(
            pos2(outer.left(), outer.bottom() - 38.0),
            outer.right_bottom(),
        );
        for (sign, text, x) in [
            (-1, "Назад", footer.left()),
            (1, "Дальше", footer.right() - 100.0),
        ] {
            let rect = Rect::from_min_size(pos2(x, footer.top() + 3.0), vec2(100.0, 33.0));
            if button(ui, rect, text, text) {
                self.advance(sign);
            }
            let dir = sign as f32;
            let middle = pos2(x + if sign < 0 { 16.0 } else { 84.0 }, rect.center().y);
            let tip = middle + vec2(dir * 4.0, 0.0);
            let stroke = Stroke::new(1.2, INK);
            let p = ui.painter_at(rect);
            p.line_segment([middle - vec2(dir * 4.0, 0.0), tip], stroke);
            p.add(Shape::line(
                vec![
                    tip + vec2(-dir * 3.6, -3.6),
                    tip,
                    tip + vec2(-dir * 3.6, 3.6),
                ],
                stroke,
            ));
        }
        let experiment = EXPERIMENTS[self.index];
        let p = ui.painter_at(outer);
        p.text(
            outer.left_top() + vec2(0.0, 2.0),
            Align2::LEFT_TOP,
            experiment.title(),
            heading,
            INK,
        );
        p.text(
            outer.left_top() + vec2(0.0, 45.0),
            Align2::LEFT_CENTER,
            experiment.hint(),
            FontId::proportional(14.0),
            MUTED,
        );
        let center = footer.center();
        p.text(
            center - vec2(104.0, 0.0),
            Align2::CENTER_CENTER,
            format!("{} / {}", self.index + 1, EXPERIMENTS.len()),
            FontId::monospace(14.0),
            MUTED,
        );
        p.vline(
            center.x - 74.0,
            center.y - 9.0..=center.y + 9.0,
            Stroke::new(1.0, RULE),
        );
        let rating = Rect::from_min_size(center - vec2(58.0, 12.0), vec2(165.0, 24.0));
        complexity_badge(ui, rating, experiment.complexity());
        let mut canvas = Rect::from_min_max(
            outer.left_top() + vec2(0.0, 66.0),
            footer.right_top() - vec2(0.0, 9.0),
        );
        let dt = ui.input(|input| input.stable_dt).clamp(0.0, 0.06);
        match experiment {
            Experiment::Engraving => self.engraving.show(ui, canvas, dt),
            Experiment::Moire => self.moire.show(ui, canvas, dt),
            Experiment::Fabric => self.fabric.show(ui, canvas, dt),
            Experiment::Pixel => self.pixel.show(ui, canvas, dt),
            Experiment::Theatre => self.show_theatre(ui, canvas),
            Experiment::Magnetic => {
                let controls = Rect::from_min_size(
                    pos2(outer.right() - 245.0, outer.top() + 10.0),
                    vec2(245.0, 28.0),
                );
                self.magnetic.show_density(ui, controls);
                self.magnetic.show(ui, canvas, dt);
            }
            Experiment::Prism => {
                let top = outer.top() + 10.0;
                let toggle =
                    Rect::from_min_size(pos2(outer.right() - 228.0, top), vec2(100.0, 31.0));
                let reset =
                    Rect::from_min_size(pos2(outer.right() - 119.0, top), vec2(119.0, 31.0));
                if button(
                    ui,
                    toggle,
                    "prism-spin",
                    if self.prism.spinning {
                        "Пауза"
                    } else {
                        "Вращать"
                    },
                ) {
                    self.prism.spinning = !self.prism.spinning;
                    ui.ctx().request_repaint();
                }
                if button(ui, reset, "prism-reset", "Исходный вид") {
                    self.prism.yaw = Prism::YAW;
                    self.prism.pitch = Prism::PITCH;
                    ui.ctx().request_repaint();
                }
                let controls = Rect::from_min_size(canvas.min, vec2(canvas.width(), 34.0));
                self.prism.controls(ui, controls);
                canvas.min.y += 39.0;
                self.prism.show(ui, canvas, dt);
            }
        }
    }

    fn show_theatre(&mut self, ui: &mut egui::Ui, canvas: Rect) {
        let p = ui.painter_at(canvas);
        vertical_gradient(
            &p,
            canvas,
            Color32::from_rgb(10, 16, 30),
            Color32::from_rgb(22, 43, 57),
        );
        let scale = canvas.height().min(canvas.width());
        let center = canvas.center() - vec2(0.0, scale * 0.08);
        halo(
            &p,
            center,
            vec2(scale * 0.58, scale * 0.42),
            Color32::from_rgba_unmultiplied(40, 108, 117, 148),
        );
        let brass = Color32::from_rgb(225, 173, 102);
        for (width, height, stroke) in [(0.31, 0.13, 0.65), (0.26, 0.20, 0.9), (0.19, 0.25, 1.3)] {
            p.add(Shape::ellipse_stroke(
                center,
                vec2(scale * width, scale * height),
                Stroke::new(stroke, brass.gamma_multiply(0.56)),
            ));
        }
        let lamp = center + vec2(scale * 0.205, -scale * 0.11);
        halo(
            &p,
            lamp,
            Vec2::splat(scale * 0.11),
            brass.gamma_multiply(0.55),
        );
        p.circle_filled(lamp, scale * 0.016, Color32::from_rgb(255, 220, 159));
        p.circle_filled(center, scale * 0.078, Color32::from_rgb(113, 155, 160));
        halo(
            &p,
            center + vec2(-scale * 0.024, -scale * 0.026),
            Vec2::splat(scale * 0.066),
            Color32::from_rgb(232, 223, 166),
        );

        let open = Rect::from_center_size(
            pos2(canvas.center().x, canvas.bottom() - 65.0),
            vec2(170.0, 38.0),
        );
        p.rect_filled(open, 0, PAPER);
        let clicked = button(ui, open, "theatre-open", "Запустить в Bevy");
        let enter = ui.memory(|memory| memory.focused().is_none())
            && consume_key(ui.ctx(), Key::Enter, false) > 0;
        if clicked || enter {
            self.theatre_error = launch_theatre()
                .err()
                .map(|error| format!("Окно Bevy не открылось. {error}"));
        }
        if let Some(error) = &self.theatre_error {
            p.text(
                open.center_bottom() + vec2(0.0, 17.0),
                Align2::CENTER_CENTER,
                error,
                FontId::proportional(12.0),
                Color32::from_rgb(255, 203, 155),
            );
        }
    }
}

fn launch_theatre() -> std::io::Result<()> {
    use std::path::Path;
    use std::process::Command;
    let name = format!("mira-bevy-light{}", std::env::consts::EXE_SUFFIX);
    let packaged = std::env::current_exe()?.with_file_name(&name);
    let path = if packaged.is_file() {
        packaged
    } else {
        demo_executable(&Path::new("target/bevy-light/debug").join(name))?
    };
    Command::new(path).spawn().map(drop)
}

fn demo_executable(relative: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    let current = std::env::current_exe()?;
    current
        .ancestors()
        .skip(1)
        .chain(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors())
        .map(|parent| parent.join(relative))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Сборка не найдена: {}", relative.display()),
            )
        })
}

pub fn run_animation() -> eframe::Result {
    eframe::run_native(
        "Анимация · Мира",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1240.0, 700.0])
                .with_min_inner_size([700.0, 450.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Ok(Box::new(Animation::default()))
        }),
    )
}

impl eframe::App for Animation {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.painter().rect_filled(ui.max_rect(), 0, PAPER);
        self.show(ui, FontId::new(31.0, egui::FontFamily::Name(SERIF.into())));
    }
}

fn complexity_badge(ui: &mut egui::Ui, rect: Rect, complexity: usize) {
    let info = format!("Сложность реализации · {complexity} из {COMPLEXITY_LEVELS}");
    let response = ui.interact(
        rect,
        ui.make_persistent_id("animation-complexity"),
        egui::Sense::hover(),
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &info));
    response.on_hover_text(info);
    let painter = ui.painter_at(rect);
    painter.text(
        rect.left_center(),
        Align2::LEFT_CENTER,
        "Сложность",
        FontId::proportional(13.0),
        MUTED,
    );
    for index in 0..COMPLEXITY_LEVELS {
        draw_star(
            &painter,
            rect.left_center() + vec2(84.0 + 15.0 * index as f32, 0.0),
            index < complexity,
        );
    }
}

fn draw_star(p: &Painter, center: Pos2, filled: bool) {
    let points = (0..10)
        .map(|index| {
            let radius = if index % 2 == 0 { 6.0 } else { 2.8 };
            let angle = -std::f32::consts::FRAC_PI_2 + index as f32 * std::f32::consts::TAU / 10.0;
            let (sin, cos) = angle.sin_cos();
            center + vec2(cos, sin) * radius
        })
        .collect::<Vec<_>>();
    let color = if filled {
        Color32::from_rgb(150, 66, 30)
    } else {
        Color32::from_rgb(145, 143, 120)
    };
    if filled {
        let mut mesh = Mesh::default();
        mesh.colored_vertex(center, color);
        for (index, &point) in points.iter().enumerate() {
            mesh.colored_vertex(point, color);
            mesh.add_triangle(0, index as u32 + 1, ((index + 1) % points.len()) as u32 + 1);
        }
        p.add(Shape::mesh(mesh));
    }
    p.add(Shape::closed_line(points, Stroke::new(0.85, color)));
}

fn button(ui: &mut egui::Ui, rect: Rect, id: &str, label: &str) -> bool {
    let response = ui
        .interact(
            rect,
            ui.make_persistent_id(("animation-button", id)),
            egui::Sense::click(),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    let hot = response.hovered() || response.has_focus();
    let p = ui.painter_at(rect);
    if hot {
        p.rect_filled(rect, 0, Color32::from_rgb(233, 228, 211));
    }
    p.hline(
        rect.x_range(),
        rect.bottom() - 1.0,
        Stroke::new(if hot { 2.0 } else { 1.0 }, if hot { INK } else { RULE }),
    );
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(14.0),
        INK,
    );
    response.clicked()
}

#[derive(Default)]
struct Engraving {
    center: Pos2,
    strength: f32,
}

impl Engraving {
    fn show(&mut self, ui: &mut egui::Ui, canvas: Rect, dt: f32) {
        let response = ui
            .interact(
                canvas,
                ui.make_persistent_id("engraving-canvas"),
                egui::Sense::hover(),
            )
            .on_hover_cursor(egui::CursorIcon::Crosshair);
        let pointer = response.hover_pos();
        let follow = 1.0 - (-18.0 * dt).exp();
        if let Some(pointer) = pointer {
            self.center = if self.strength < 0.001 {
                pointer
            } else {
                self.center.lerp(pointer, follow)
            };
        }
        let target = if pointer.is_some() { 1.0 } else { 0.0 };
        self.strength += (target - self.strength) * (1.0 - (-11.0 * dt).exp());
        if (target - self.strength).abs() < 0.002 {
            self.strength = target;
        }
        if self.strength != target || pointer.is_some_and(|pos| pos.distance(self.center) > 0.15) {
            ui.ctx().request_repaint_after_secs(1.0 / 60.0);
        }

        let p = ui.painter_at(canvas);
        let ivory = Color32::from_rgb(248, 244, 230);
        let charcoal = Color32::from_rgb(37, 38, 31);
        vertical_gradient(&p, canvas, ivory, Color32::from_rgb(231, 222, 201));
        let print = canvas.shrink(18.0);
        let plate = p.with_clip_rect(print);
        let rows = (print.height() / 7.2).round().clamp(22.0, 100.0) as usize;
        let columns = (print.width() / 8.0).round().clamp(40.0, 230.0) as usize;
        let radius_sq = (print.height() * 0.225).powi(2);
        for row in 0..=rows {
            let depth = row as f32 / rows as f32;
            let points = (0..=columns)
                .map(|column| {
                    let x = column as f32 / columns as f32;
                    let envelope = (std::f32::consts::PI * x).sin().max(0.0).powf(1.25);
                    let wave = (x * 13.0 + depth * 4.0).sin() * 0.092
                        + (x * 22.0 - depth * 5.0).sin() * 0.030;
                    let base_y = 0.045 + 0.91 * depth + wave * envelope;
                    let point = print.min + print.size() * vec2(x, base_y);
                    let distance = point - self.center;
                    let pressure = 2.05 * self.strength * (-distance.length_sq() / radius_sq).exp();
                    point + distance * pressure
                })
                .collect();
            let stroke = if row % 5 == 0 {
                Stroke::new(1.18, charcoal)
            } else {
                Stroke::new(0.66, Color32::from_rgb(105, 101, 86))
            };
            plate.add(Shape::line(points, stroke));
        }
        p.rect_stroke(
            print,
            0,
            Stroke::new(1.0, Color32::from_rgb(151, 141, 119)),
            StrokeKind::Inside,
        );
        p.rect_stroke(canvas, 0, Stroke::new(1.0, RULE), StrokeKind::Inside);
    }
}

struct Prism {
    yaw: f32,
    pitch: f32,
    spinning: bool,
    speed: f32,
    elapsed: f32,
    form: PrismForm,
    finish: PrismFinish,
    tint: Color32,
    speed_id: Option<egui::Id>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PrismForm {
    Prism,
    Shard,
    Crystal,
    Seal,
    Trigon,
    Lens,
    Obelisk,
}

impl PrismForm {
    const ALL: [Self; 7] = [
        Self::Prism,
        Self::Shard,
        Self::Crystal,
        Self::Seal,
        Self::Trigon,
        Self::Lens,
        Self::Obelisk,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Prism => "Призма",
            Self::Shard => "Осколок",
            Self::Crystal => "Кристалл",
            Self::Seal => "Печать",
            Self::Trigon => "Тригранник",
            Self::Lens => "Линза",
            Self::Obelisk => "Обелиск",
        }
    }

    fn profile(self) -> (usize, &'static [(f32, f32)]) {
        match self {
            Self::Prism => (
                6,
                &[(-1.18, 0.54), (-0.79, 0.94), (0.68, 1.0), (1.10, 0.63)],
            ),
            Self::Crystal => (5, &[(-1.30, 0.0), (0.0, 0.99), (1.30, 0.0)]),
            Self::Seal => (
                8,
                &[(-0.40, 0.81), (-0.24, 1.18), (0.24, 1.18), (0.40, 0.81)],
            ),
            Self::Trigon => (3, &[(-0.91, 1.03), (0.91, 1.03)]),
            Self::Lens => (
                10,
                &[
                    (-0.34, 0.38),
                    (-0.19, 0.91),
                    (0.0, 1.16),
                    (0.19, 0.91),
                    (0.34, 0.38),
                ],
            ),
            Self::Obelisk => (
                4,
                &[(-1.21, 0.59), (-0.90, 0.76), (0.67, 0.80), (1.30, 0.0)],
            ),
            Self::Shard => unreachable!("the tetrahedron has its own vertices"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PrismFinish {
    Studio,
    Sweep,
    Pearl,
    Neon,
    Wave,
}

impl PrismFinish {
    const ALL: [Self; 5] = [
        Self::Studio,
        Self::Sweep,
        Self::Pearl,
        Self::Neon,
        Self::Wave,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Studio => "Студийный",
            Self::Sweep => "Бегущий блик",
            Self::Pearl => "Перламутр",
            Self::Neon => "Неоновые рёбра",
            Self::Wave => "Волна света",
        }
    }
}

impl Default for Prism {
    fn default() -> Self {
        Self {
            yaw: Self::YAW,
            pitch: Self::PITCH,
            spinning: true,
            speed: 1.0,
            elapsed: 0.0,
            form: PrismForm::Prism,
            finish: PrismFinish::Studio,
            tint: Self::DEFAULT_TINT,
            speed_id: None,
        }
    }
}

impl Prism {
    const YAW: f32 = 0.36;
    const PITCH: f32 = 0.28;
    const DEFAULT_TINT: Color32 = Color32::from_rgb(39, 216, 208);

    fn control_has_focus(&self, ui: &egui::Ui) -> bool {
        self.speed_id
            .is_some_and(|id| ui.memory(|memory| memory.has_focus(id)))
            || egui::Popup::is_any_open(ui.ctx())
    }

    fn controls(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.horizontal(|ui| {
                ui.label("Форма");
                egui::ComboBox::from_id_salt("prism-form")
                    .width(108.0)
                    .selected_text(self.form.name())
                    .show_ui(ui, |ui| {
                        for form in PrismForm::ALL {
                            ui.selectable_value(&mut self.form, form, form.name());
                        }
                    });
                ui.separator();
                ui.label("Свет");
                egui::ComboBox::from_id_salt("prism-finish")
                    .width(129.0)
                    .selected_text(self.finish.name())
                    .show_ui(ui, |ui| {
                        for finish in PrismFinish::ALL {
                            ui.selectable_value(&mut self.finish, finish, finish.name());
                        }
                    });
                ui.separator();
                ui.label("Цвет");
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut self.tint,
                    egui::color_picker::Alpha::Opaque,
                );
                if ui
                    .small_button("↺")
                    .on_hover_text("Исходная палитра")
                    .clicked()
                {
                    self.tint = Self::DEFAULT_TINT;
                }
                ui.separator();
                ui.label("Скорость");
                self.speed_id = Some(
                    ui.add(
                        egui::Slider::new(&mut self.speed, 0.15..=3.0)
                            .suffix("×")
                            .fixed_decimals(2),
                    )
                    .id,
                );
            });
        });
    }

    fn show(&mut self, ui: &mut egui::Ui, canvas: Rect, dt: f32) {
        let response = ui
            .interact(
                canvas,
                ui.make_persistent_id("prism-canvas"),
                egui::Sense::drag(),
            )
            .on_hover_cursor(egui::CursorIcon::Grab);
        if response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            let delta = response.drag_delta();
            self.yaw += delta.x * 0.012;
            self.pitch = (self.pitch + delta.y * 0.012).clamp(-1.45, 1.45);
        } else if self.spinning && !response.is_pointer_button_down_on() {
            self.yaw += dt * 0.62 * self.speed;
        }
        let moving_light = matches!(self.finish, PrismFinish::Sweep | PrismFinish::Wave);
        if self.spinning || moving_light {
            self.elapsed += dt;
            ui.ctx().request_repaint_after_secs(1.0 / 60.0);
        }
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);
        let p = ui.painter_at(canvas);
        let unit = canvas.width().min(canvas.height());
        let center = canvas.center() - vec2(0.0, 8.0);
        vertical_gradient(
            &p,
            canvas,
            Color32::from_rgb(9, 13, 31),
            Color32::from_rgb(18, 35, 57),
        );
        halo(
            &p,
            center,
            vec2(unit * 0.76, unit * 0.57),
            Color32::from_rgba_unmultiplied(44, 63, 161, 165),
        );
        let ground = center + vec2(0.0, unit * 0.365);
        halo(
            &p,
            ground,
            vec2(unit * 0.35, unit * 0.062),
            Color32::from_black_alpha(160),
        );
        for factor in [0.25, 0.33, 0.43] {
            p.add(Shape::ellipse_stroke(
                ground,
                vec2(unit * factor, unit * factor * 0.17),
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(112, 191, 224, 57)),
            ));
        }
        self.paint_solid(&p, center, unit * 1.23);
        p.rect_stroke(
            canvas,
            0,
            Stroke::new(1.0, Color32::from_rgb(51, 66, 98)),
            StrokeKind::Inside,
        );
    }

    fn paint_solid(&self, p: &Painter, center: Pos2, focal: f32) {
        let eye = [0.0, 0.0, 4.8];
        let (sy, cy) = self.yaw.sin_cos();
        let (sx, cx) = self.pitch.sin_cos();
        let rotated = |[x, y, z]: Point3| {
            let x1 = x * cy + z * sy;
            let z1 = z * cy - x * sy;
            [x1, y * cx - z1 * sx, y * sx + z1 * cx]
        };
        let key_position = if self.finish == PrismFinish::Sweep {
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
            let mut normal = normalized(cross(
                sub(vertices[1], vertices[0]),
                sub(vertices[2], vertices[0]),
            ));
            if dot(normal, midpoint) < 0.0 {
                normal = scale(normal, -1.0);
            }
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
                    PrismFinish::Pearl => {
                        let phase = 7.0 * (1.0 - facing);
                        let channel = |offset: f32| (142.0 + 108.0 * (phase + offset).cos()) as u8;
                        base.lerp_to_gamma(
                            Color32::from_rgb(channel(0.0), channel(2.1), channel(4.2)),
                            0.56,
                        )
                    }
                    PrismFinish::Neon => base.gamma_multiply(0.48),
                    PrismFinish::Wave => {
                        let height = (self.elapsed * 2.0).sin() * 1.08;
                        let distance = (vertex[1] + 0.32 * vertex[0] - height) / 0.48;
                        base.lerp_to_gamma(
                            Color32::from_rgb(231, 250, 255),
                            0.79 * (-distance * distance).exp(),
                        )
                    }
                    PrismFinish::Studio | PrismFinish::Sweep => base,
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
            let neon = self.finish == PrismFinish::Neon;
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
        let pigments = [
            Color32::from_rgb(39, 216, 208),
            Color32::from_rgb(67, 116, 247),
            Color32::from_rgb(142, 114, 245),
            Color32::from_rgb(49, 162, 228),
            Color32::from_rgb(87, 236, 205),
            Color32::from_rgb(85, 94, 201),
        ];
        if self.form == PrismForm::Shard {
            let vertices = [
                [0.92, 0.92, 0.92],
                [-0.92, -0.92, 0.92],
                [-0.92, 0.92, -0.92],
                [0.92, -0.92, -0.92],
            ]
            .map(rotated);
            for (index, face) in [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]]
                .into_iter()
                .enumerate()
            {
                surface(&face.map(|corner| vertices[corner]), pigments[index]);
            }
            return;
        }

        let (sides, profile) = self.form.profile();
        let rings: Vec<Vec<_>> = profile
            .iter()
            .map(|&(height, radius)| {
                (0..sides)
                    .map(|side| {
                        let angle = side as f32 * std::f32::consts::TAU / sides as f32;
                        let (sin, cos) = angle.sin_cos();
                        rotated([cos * radius, height, sin * radius])
                    })
                    .collect()
            })
            .collect();
        for (band, pair) in rings.windows(2).enumerate() {
            for side in 0..sides {
                let next = (side + 1) % sides;
                let lower = &pair[0];
                let upper = &pair[1];
                let pigment = pigments[side % pigments.len()];
                if profile[band].1 == 0.0 {
                    surface(&[lower[side], upper[side], upper[next]], pigment);
                } else if profile[band + 1].1 == 0.0 {
                    surface(&[lower[side], upper[side], lower[next]], pigment);
                } else {
                    surface(
                        &[lower[side], upper[side], upper[next], lower[next]],
                        pigment,
                    );
                }
            }
        }
        if profile[0].1 > 0.0 {
            surface(&rings[0], Color32::from_rgb(43, 89, 155));
        }
        let top = profile.len() - 1;
        if profile[top].1 > 0.0 {
            surface(&rings[top], Color32::from_rgb(109, 226, 228));
        }
    }
}

fn halo(p: &Painter, center: Pos2, radius: Vec2, color: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    for i in 0..40 {
        let (sin, cos) = (i as f32 * std::f32::consts::TAU / 40.0).sin_cos();
        mesh.colored_vertex(center + vec2(cos, sin) * radius, Color32::TRANSPARENT);
        mesh.add_triangle(0, i + 1, (i + 1) % 40 + 1);
    }
    p.add(Shape::mesh(mesh));
}

type Point3 = [f32; 3];

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Point3, factor: f32) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn dot(a: Point3, b: Point3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalized(a: Point3) -> Point3 {
    scale(a, dot(a, a).max(f32::EPSILON).sqrt().recip())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &egui::Context, animation: &mut Animation, time: f64, events: Vec<egui::Event>) {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(700.0, 380.0))),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| animation.show(ui, FontId::proportional(31.0)),
        )
        .textures_delta
        .clear();
    }

    fn arrow(key: Key) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            })
            .collect()
    }

    #[test]
    fn footer_button_advances_the_carousel_inside_the_atlas_content() {
        let ctx = egui::Context::default();
        let mut animation = Animation::default();
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 640.0));
        let content = Rect::from_min_max(pos2(0.0, 123.0), screen.right_bottom());
        let next = pos2(820.0, 559.0);
        let click = |pressed| egui::Event::PointerButton {
            pos: next,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        for (index, events) in [
            vec![],
            vec![egui::Event::PointerMoved(next)],
            vec![click(true)],
            vec![click(false)],
        ]
        .into_iter()
        .enumerate()
        {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    time: Some(index as f64 * 0.04),
                    events,
                    ..Default::default()
                },
                |ui| {
                    ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
                        ui.set_clip_rect(content);
                        animation.show(ui, FontId::proportional(31.0));
                    });
                },
            )
            .textures_delta
            .clear();
        }
        assert_eq!(animation.index, 1);
    }

    #[test]
    fn scrolling_reveals_the_unity_launchers_beneath_the_album() {
        let ctx = egui::Context::default();
        let mut animation = Animation::default();
        let mut draw = |time, events| {
            let mut frame = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1130.0, 630.0))),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ui| animation.show(ui, FontId::proportional(31.0)),
            );
            frame.textures_delta.clear();
            frame
                .shapes
                .into_iter()
                .filter_map(|clipped| match clipped.shape {
                    Shape::Text(text)
                        if clipped.clip_rect.intersects(text.visual_bounding_rect()) =>
                    {
                        Some(text.galley.job.text.clone())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let before = draw(0.0, vec![]);
        assert!(before.iter().any(|text| text == "Живая гравюра"));
        assert!(before.iter().any(|text| text == "Проекты Unity ↓"));
        assert!(!before.iter().any(|text| text == "Наклонный сад"));

        draw(
            0.04,
            vec![
                egui::Event::PointerMoved(pos2(1050.0, 515.0)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    phase: egui::TouchPhase::Move,
                    delta: vec2(0.0, -1000.0),
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        let after = draw(0.24, vec![]);
        for title in [
            "Наклонный сад",
            "Площадь от рассвета до ночи",
            "Дорожка домино",
        ] {
            assert!(after.iter().any(|text| text == title), "{title}: {after:?}");
        }
        assert_eq!(
            after
                .iter()
                .filter(|text| text.as_str() == "Запустить ↗")
                .count(),
            3
        );
    }

    #[test]
    fn arrows_wrap_and_the_paused_prism_still_rotates_when_dragged() {
        let ctx = egui::Context::default();
        let mut animation = Animation::default();
        frame(&ctx, &mut animation, 0.0, vec![]);
        frame(&ctx, &mut animation, 0.02, arrow(Key::ArrowLeft));
        assert_eq!(animation.index, EXPERIMENTS.len() - 1);
        frame(&ctx, &mut animation, 0.04, arrow(Key::ArrowRight));
        assert_eq!(animation.index, 0);
        frame(&ctx, &mut animation, 0.06, arrow(Key::ArrowRight));
        assert_eq!(animation.index, 1);
        let angle = animation.prism.yaw;
        frame(&ctx, &mut animation, 0.08, vec![]);
        assert!(animation.prism.yaw > angle);
        animation.prism.spinning = false;
        let angle = animation.prism.yaw;
        let start = pos2(370.0, 160.0);
        frame(
            &ctx,
            &mut animation,
            0.10,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        assert_eq!(animation.prism.yaw, angle);
        frame(
            &ctx,
            &mut animation,
            0.12,
            vec![egui::Event::PointerMoved(start + vec2(46.0, -25.0))],
        );
        assert!(animation.prism.yaw > angle + 0.4);
        assert!(animation.prism.pitch < Prism::PITCH - 0.2);
        frame(&ctx, &mut animation, 0.14, arrow(Key::ArrowLeft));
        assert_eq!(animation.index, 0);
    }

    #[test]
    fn the_engraving_follows_the_pointer_and_relaxes_after_it_leaves() {
        let ctx = egui::Context::default();
        let mut animation = Animation::default();
        frame(&ctx, &mut animation, 0.0, vec![]);
        let start = pos2(360.0, 150.0);
        frame(
            &ctx,
            &mut animation,
            0.02,
            vec![egui::Event::PointerMoved(start)],
        );
        assert_eq!(animation.engraving.center, start);
        let strength = animation.engraving.strength;
        assert!(strength > 0.0);
        frame(
            &ctx,
            &mut animation,
            0.04,
            vec![egui::Event::PointerMoved(start + vec2(45.0, 0.0))],
        );
        assert!(animation.engraving.center.x > start.x);
        assert!(animation.engraving.center.x < start.x + 45.0);
        assert!(animation.engraving.strength > strength);
        let strength = animation.engraving.strength;
        frame(&ctx, &mut animation, 0.06, vec![egui::Event::PointerGone]);
        assert!(animation.engraving.strength < strength);
    }
}
