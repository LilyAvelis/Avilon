use avilon_egui::engraving::{Settings as EngravingSettings, View as EngravingView};
use avilon_egui::facets::{Form, Light, Preset, View};
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
                    let mut album_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt("animation-album")
                            .max_rect(album),
                    );
                    album_ui.set_clip_rect(ui.clip_rect().intersect(album));
                    self.show_album(
                        &mut album_ui,
                        album.shrink2(vec2(30.0, 14.0)),
                        heading.clone(),
                    );
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
            Experiment::Engraving => self.engraving.control_has_focus(ui),
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
            Experiment::Engraving => {
                let controls = Rect::from_min_size(canvas.min, vec2(canvas.width(), 34.0));
                canvas.min.y += self.engraving.controls(ui, controls);
                self.engraving.show(ui, canvas);
            }
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
                    if self.prism.view.is_spinning() {
                        "Пауза"
                    } else {
                        "Вращать"
                    },
                ) {
                    self.prism.view.set_spinning(!self.prism.view.is_spinning());
                    ui.ctx().request_repaint();
                }
                if button(ui, reset, "prism-reset", "Исходный вид") {
                    self.prism.view.reset_angles();
                    ui.ctx().request_repaint();
                }
                let controls = Rect::from_min_size(canvas.min, vec2(canvas.width(), 34.0));
                self.prism.controls(ui, controls);
                canvas.min.y += 39.0;
                self.prism.show(ui, canvas);
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

pub(super) fn demo_executable(relative: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
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

pub(super) fn complexity_badge(ui: &mut egui::Ui, rect: Rect, complexity: usize) {
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
    view: EngravingView,
    settings: EngravingSettings,
    controls: [Option<egui::Id>; 4],
}

impl Engraving {
    fn control_has_focus(&self, ui: &egui::Ui) -> bool {
        ui.memory(|memory| {
            self.controls
                .iter()
                .flatten()
                .any(|&id| memory.has_focus(id))
        })
    }

    fn controls(&mut self, ui: &mut egui::Ui, rect: Rect) -> f32 {
        let controls = ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.spacing_mut().slider_width = 71.0;
            ui.horizontal_wrapped(|ui| {
                ui.label("Сид");
                self.controls[0] = Some(ui.add(egui::DragValue::new(&mut self.settings.seed)).id);
                if ui.small_button("Другой рисунок").clicked() {
                    self.settings.seed = self
                        .settings
                        .seed
                        .wrapping_mul(1_664_525)
                        .wrapping_add(1_013_904_223);
                }
                ui.separator();
                ui.label("Радиус");
                self.controls[1] = Some(
                    ui.add(
                        egui::Slider::new(&mut self.settings.radius, 0.04..=0.45).fixed_decimals(2),
                    )
                    .id,
                );
                ui.label("Зум");
                self.controls[2] = Some(
                    ui.add(
                        egui::Slider::new(&mut self.settings.zoom, 0.32..=5.5)
                            .logarithmic(true)
                            .fixed_decimals(2)
                            .suffix("×"),
                    )
                    .id,
                );
                ui.label("Плотность");
                self.controls[3] = Some(
                    ui.add(
                        egui::Slider::new(&mut self.settings.density, 0.5..=2.0).fixed_decimals(2),
                    )
                    .id,
                );
                ui.toggle_value(&mut self.settings.live, "Оживить");
            });
        });
        self.view.set_settings(self.settings);
        controls.response.rect.height().max(30.0) + 6.0
    }

    fn show(&mut self, ui: &mut egui::Ui, canvas: Rect) {
        self.view
            .show_in(ui, canvas, ui.make_persistent_id("engraving-canvas"));
    }
}

struct Prism {
    view: View,
    speed: f32,
    form: PrismForm,
    finish: Light,
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

    fn geometry(self) -> Form {
        Form::preset(match self {
            Self::Prism => Preset::Prism,
            Self::Shard => Preset::Shard,
            Self::Crystal => Preset::Crystal,
            Self::Seal => Preset::Seal,
            Self::Trigon => Preset::Trigon,
            Self::Lens => Preset::Lens,
            Self::Obelisk => Preset::Obelisk,
        })
    }
}

const LIGHTS: [Light; 5] = [
    Light::Studio,
    Light::Sweep,
    Light::Pearl,
    Light::Neon,
    Light::Wave,
];

fn light_name(light: Light) -> &'static str {
    match light {
        Light::Studio => "Студийный",
        Light::Sweep => "Бегущий блик",
        Light::Pearl => "Перламутр",
        Light::Neon => "Неоновые рёбра",
        Light::Wave => "Волна света",
    }
}

impl Default for Prism {
    fn default() -> Self {
        Self {
            view: View::new(Form::preset(Preset::Prism)),
            speed: 1.0,
            form: PrismForm::Prism,
            finish: Light::Studio,
            tint: Self::DEFAULT_TINT,
            speed_id: None,
        }
    }
}

impl Prism {
    const DEFAULT_TINT: Color32 = View::DEFAULT_TINT;

    fn control_has_focus(&self, ui: &egui::Ui) -> bool {
        self.speed_id
            .is_some_and(|id| ui.memory(|memory| memory.has_focus(id)))
            || egui::Popup::is_any_open(ui.ctx())
    }

    fn controls(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let previous = self.form;
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
                    .selected_text(light_name(self.finish))
                    .show_ui(ui, |ui| {
                        for finish in LIGHTS {
                            ui.selectable_value(&mut self.finish, finish, light_name(finish));
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
        if self.form != previous {
            self.view.set_form(self.form.geometry());
        }
        self.view.set_light(self.finish);
        self.view.set_color(self.tint);
        self.view.set_spin_speed(self.speed);
    }

    fn show(&mut self, ui: &mut egui::Ui, canvas: Rect) {
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
        self.view
            .show_in(ui, canvas, ui.make_persistent_id("prism-canvas"));
        p.rect_stroke(
            canvas,
            0,
            Stroke::new(1.0, Color32::from_rgb(51, 66, 98)),
            StrokeKind::Inside,
        );
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
        for experiment in [
            Experiment::Engraving,
            Experiment::Prism,
            Experiment::Magnetic,
        ] {
            assert_unity_follows_album(experiment);
        }
    }

    fn assert_unity_follows_album(experiment: Experiment) {
        let ctx = egui::Context::default();
        let mut animation = Animation {
            index: EXPERIMENTS
                .iter()
                .position(|&item| item == experiment)
                .unwrap(),
            ..Animation::default()
        };
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
        assert!(before.iter().any(|text| text == experiment.title()));
        assert!(before.iter().any(|text| text == "Проекты Unity ↓"));
        assert!(
            !before
                .iter()
                .any(|text| text == "Проекты Unity" || text == "Наклонный сад"),
            "Unity overlaps the {experiment:?} album: {before:?}"
        );

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
            "Лаборатория воды",
        ] {
            assert!(after.iter().any(|text| text == title), "{title}: {after:?}");
        }
        assert_eq!(
            after
                .iter()
                .filter(|text| text.as_str() == "Запустить ↗")
                .count(),
            4
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
        let angle = animation.prism.view.angles()[0];
        frame(&ctx, &mut animation, 0.08, vec![]);
        assert!(animation.prism.view.angles()[0] > angle);
        animation.prism.view.set_spinning(false);
        let angle = animation.prism.view.angles()[0];
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
        assert_eq!(animation.prism.view.angles()[0], angle);
        frame(
            &ctx,
            &mut animation,
            0.12,
            vec![egui::Event::PointerMoved(start + vec2(46.0, -25.0))],
        );
        assert!(animation.prism.view.angles()[0] > angle + 0.4);
        assert!(animation.prism.view.angles()[1] < 0.28 - 0.2);
        frame(&ctx, &mut animation, 0.14, arrow(Key::ArrowLeft));
        assert_eq!(animation.index, 0);
    }

    #[test]
    fn the_engraving_live_toggle_works_at_minimum_width() {
        let ctx = egui::Context::default();
        let mut animation = Animation::default();
        let mut initial = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(700.0, 380.0))),
                time: Some(0.0),
                ..Default::default()
            },
            |ui| animation.show(ui, FontId::proportional(31.0)),
        );
        initial.textures_delta.clear();
        let toggle = initial
            .shapes
            .iter()
            .find_map(|item| match &item.shape {
                Shape::Text(text)
                    if text.galley.job.text == "Оживить"
                        && item
                            .clip_rect
                            .contains(text.visual_bounding_rect().center()) =>
                {
                    Some(text.visual_bounding_rect().center())
                }
                _ => None,
            })
            .expect("live toggle is visible");
        frame(
            &ctx,
            &mut animation,
            0.04,
            vec![egui::Event::PointerMoved(toggle)],
        );
        for (pressed, time) in [(true, 0.08), (false, 0.12)] {
            frame(
                &ctx,
                &mut animation,
                time,
                vec![egui::Event::PointerButton {
                    pos: toggle,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
        assert!(animation.engraving.settings.live);
        let center = animation.engraving.view.lens_state().0;
        frame(&ctx, &mut animation, 0.18, vec![egui::Event::PointerGone]);
        assert!(center.distance(animation.engraving.view.lens_state().0) > 0.1);
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
        assert_eq!(animation.engraving.view.lens_state().0, start);
        let strength = animation.engraving.view.lens_state().1;
        assert!(strength > 0.0);
        frame(
            &ctx,
            &mut animation,
            0.04,
            vec![egui::Event::PointerMoved(start + vec2(45.0, 0.0))],
        );
        assert!(animation.engraving.view.lens_state().0.x > start.x);
        assert!(animation.engraving.view.lens_state().0.x < start.x + 45.0);
        assert!(animation.engraving.view.lens_state().1 > strength);
        let strength = animation.engraving.view.lens_state().1;
        frame(&ctx, &mut animation, 0.06, vec![egui::Event::PointerGone]);
        assert!(animation.engraving.view.lens_state().1 < strength);
    }
}
