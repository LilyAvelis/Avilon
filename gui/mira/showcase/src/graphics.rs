use std::collections::HashSet;

use eframe::egui::{
    self, Align2, Color32, FontData, FontFamily, FontId, Mesh, Painter, Rect, Shape, Stroke,
    StrokeKind, TextureHandle, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};

mod print;
mod typography;
mod web;

pub(crate) const PAPER: Color32 = Color32::from_rgb(243, 241, 232);
pub(crate) const INK: Color32 = Color32::from_rgb(29, 34, 29);
pub(crate) const MUTED: Color32 = Color32::from_rgb(82, 86, 71);
pub(crate) const RULE: Color32 = Color32::from_rgb(179, 179, 158);
const TITLE: &str = "После дождя";
const FIRST: &str = "После";
const SECOND: &str = "дождя";
const LINE: &str = "Свет остаётся на мокром камне.";

struct Specimen {
    name: &'static str,
    rating: (&'static str, u8),
    face: &'static str,
    file: &'static str,
}

const SPECIMENS: [Specimen; 6] = [
    Specimen {
        name: "Оттиск",
        rating: ("S", 5),
        face: "Georgia Bold",
        file: "georgiab.ttf",
    },
    Specimen {
        name: "Чертёж",
        rating: ("B", 2),
        face: "Bahnschrift",
        file: "bahnschrift.ttf",
    },
    Specimen {
        name: "Афиша",
        rating: ("C", 1),
        face: "Arial Narrow Bold",
        file: "ARIALNB.TTF",
    },
    Specimen {
        name: "Эфир",
        rating: ("B", 2),
        face: "Palatino Linotype Italic",
        file: "palai.ttf",
    },
    Specimen {
        name: "Кинокадр",
        rating: ("A", 5),
        face: "Segoe UI Light",
        file: "segoeuil.ttf",
    },
    Specimen {
        name: "Ризограф",
        rating: ("S", 5),
        face: "Trebuchet MS Bold",
        file: "trebucbd.ttf",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Group {
    Typography,
    Print,
    Digital,
    FirstSheet,
}

const GROUPS: [Group; 4] = [
    Group::Typography,
    Group::Print,
    Group::Digital,
    Group::FirstSheet,
];

impl Group {
    fn title(self) -> &'static str {
        match self {
            Self::Typography => "Типографика",
            Self::Print => "Печать",
            Self::Digital => "Цифровая графика",
            Self::FirstSheet => "Первый лист",
        }
    }

    fn samples(self) -> &'static [Specimen] {
        match self {
            Self::Typography => &typography::SAMPLES,
            Self::Print => &print::SAMPLES,
            Self::Digital => &web::SAMPLES,
            Self::FirstSheet => &SPECIMENS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum View {
    All,
    Group(Group),
    Tabs,
}

pub(super) struct Sheet {
    fonts: HashSet<&'static str>,
    photo: TextureHandle,
    view: View,
    active_tab: usize,
}

pub(super) fn section_tabs<T: Copy + Eq + std::hash::Hash + std::fmt::Debug>(
    ui: &mut egui::Ui,
    views: &[(T, &str)],
    current: T,
    margin: f32,
    font_size: f32,
) -> T {
    let mut selected = current;
    let width = ui.available_width();
    let mut x = 0.0;
    let mut y = 0.0;
    let mut tabs = Vec::with_capacity(views.len());
    for &(view, label) in views {
        let font = FontId::proportional(font_size);
        let galley = ui.painter().layout_no_wrap(label.to_owned(), font, INK);
        let tab_width = galley.size().x + 20.0;
        if x + tab_width > width - 2.0 * margin {
            x = 0.0;
            y += 33.0;
        }
        tabs.push((view, label, galley, vec2(x, y), tab_width));
        x += tab_width + 6.0;
    }
    let row = ui
        .allocate_space(vec2(width, y + 33.0))
        .1
        .shrink2(vec2(margin, 0.0));
    ui.painter()
        .hline(row.x_range(), row.bottom(), Stroke::new(1.0, RULE));
    for (view, label, galley, offset, tab_width) in tabs {
        let rect = Rect::from_min_size(row.min + offset, vec2(tab_width, 33.0));
        let response = ui
            .interact(rect, ui.make_persistent_id(view), egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        let active = current == view;
        response
            .widget_info(|| WidgetInfo::selected(WidgetType::SelectableLabel, true, active, label));
        let accent = Color32::from_rgb(142, 52, 32);
        let hot = response.hovered() || response.has_focus();
        if active || hot {
            ui.painter().hline(
                rect.x_range(),
                rect.bottom(),
                Stroke::new(if active { 2.5 } else { 1.0 }, accent),
            );
        }
        let origin = rect.center() - galley.size() * 0.5;
        ui.painter().galley_with_override_text_color(
            origin,
            galley,
            if active || hot { accent } else { INK },
        );
        if response.clicked() {
            selected = view;
            ui.ctx().request_repaint();
        }
    }
    selected
}

impl Sheet {
    pub(super) fn new(ctx: &egui::Context) -> Self {
        let mut fonts = crate::font_definitions();
        let windows_fonts =
            std::env::var_os("WINDIR").map(|root| std::path::PathBuf::from(root).join("Fonts"));
        let mut loaded = HashSet::new();
        for specimen in GROUPS.into_iter().flat_map(Group::samples) {
            if loaded.contains(specimen.face) {
                continue;
            }
            let Some(bytes) = windows_fonts
                .as_ref()
                .and_then(|folder| std::fs::read(folder.join(specimen.file)).ok())
            else {
                continue;
            };
            let mut fallbacks = fonts.families[&FontFamily::Proportional].clone();
            fonts
                .font_data
                .insert(specimen.face.into(), FontData::from_owned(bytes).into());
            fallbacks.insert(0, specimen.face.into());
            fonts
                .families
                .insert(FontFamily::Name(specimen.face.into()), fallbacks);
            loaded.insert(specimen.face);
        }
        ctx.set_fonts(fonts);
        let photo = image::load_from_memory_with_format(crate::PHOTOS[2], image::ImageFormat::Jpeg)
            .expect("embedded collection image must be a valid JPEG")
            .into_rgba8();
        let photo = ctx.load_texture(
            "graphics-tram",
            egui::ColorImage::from_rgba_unmultiplied(
                [photo.width() as usize, photo.height() as usize],
                photo.as_raw(),
            ),
            egui::TextureOptions::LINEAR,
        );
        Self {
            fonts: loaded,
            photo,
            view: View::All,
            active_tab: 1,
        }
    }

    fn font(&self, specimen: &Specimen, size: f32) -> FontId {
        if self.fonts.contains(specimen.face) {
            FontId::new(size, FontFamily::Name(specimen.face.into()))
        } else {
            FontId::proportional(size)
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.painter().rect_filled(ui.max_rect(), 0, PAPER);
        self.title(ui);
        self.show_samples(ui, 30.0, 15.0);
    }

    pub(super) fn show_in_atlas(&mut self, ui: &mut egui::Ui, margin: f32) {
        ui.add_space(8.0);
        self.show_samples(ui, margin, 13.0);
    }

    fn show_samples(&mut self, ui: &mut egui::Ui, margin: f32, tab_font_size: f32) {
        self.filters(ui, margin, tab_font_size);
        egui::ScrollArea::vertical()
            .id_salt(self.view)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let content = ui.available_rect_before_wrap().shrink2(vec2(margin, 0.0));
                ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
                    ui.add_space(12.0);
                    match self.view {
                        View::All => {
                            for group in GROUPS {
                                self.section(ui, group);
                                ui.add_space(10.0);
                            }
                            self.tabs(ui);
                        }
                        View::Group(group) => self.section(ui, group),
                        View::Tabs => self.tabs(ui),
                    }
                    ui.add_space(24.0);
                });
            });
    }

    fn title(&self, ui: &mut egui::Ui) {
        ui.add_space(15.0);
        let header = ui
            .allocate_space(vec2(ui.available_width(), 77.0))
            .1
            .shrink2(vec2(30.0, 0.0));
        let p = ui.painter();
        p.text(
            header.left_top(),
            Align2::LEFT_TOP,
            "Графика",
            self.font(&SPECIMENS[0], 46.0),
            INK,
        );
        let count: usize = GROUPS.into_iter().map(|group| group.samples().len()).sum();
        p.text(
            header.left_top() + vec2(2.0, 60.0),
            Align2::LEFT_CENTER,
            format!("«После дождя» · композиций — {count}"),
            FontId::proportional(15.0),
            MUTED,
        );
    }

    fn filters(&mut self, ui: &mut egui::Ui, margin: f32, font_size: f32) {
        let views = [
            (View::All, "Все"),
            (View::Group(Group::Typography), Group::Typography.title()),
            (View::Group(Group::Print), Group::Print.title()),
            (View::Group(Group::Digital), Group::Digital.title()),
            (View::Group(Group::FirstSheet), Group::FirstSheet.title()),
            (View::Tabs, "Вкладки"),
        ];
        self.view = section_tabs(ui, &views, self.view, margin, font_size);
    }

    fn section(&self, ui: &mut egui::Ui, group: Group) {
        let width = ui.available_width();
        let heading = ui.allocate_space(vec2(width, 42.0)).1;
        ui.painter().text(
            heading.left_center(),
            Align2::LEFT_CENTER,
            group.title(),
            self.font(&SPECIMENS[0], 22.0),
            INK,
        );
        let samples = group.samples();
        let columns = if width >= 1050.0 { 3 } else { 2 };
        let gap = 16.0;
        let w = (width - gap * (columns - 1) as f32) / columns as f32;
        let h = w / 1.84;
        for (row_index, row_samples) in samples.chunks(columns).enumerate() {
            let row = ui.allocate_space(vec2(width, h + 47.0)).1;
            if !ui.is_rect_visible(row) {
                continue;
            }
            for (column, sample) in row_samples.iter().enumerate() {
                let rect =
                    Rect::from_min_size(row.min + vec2(column as f32 * (w + gap), 0.0), vec2(w, h));
                let p = ui.painter_at(rect);
                let font = |size| self.font(sample, size * rect.width() / 417.0);
                let index = row_index * columns + column;
                match group {
                    Group::Typography => typography::paint(&p, rect, index, font),
                    Group::Print => print::paint(&p, rect, index, font),
                    Group::Digital => web::paint(&p, rect, index, font),
                    Group::FirstSheet => self.specimen(&p, rect, index, font),
                }
                p.rect_stroke(
                    rect,
                    0,
                    Stroke::new(1.0, Color32::from_black_alpha(30)),
                    StrokeKind::Inside,
                );

                let p = ui.painter();
                p.text(
                    rect.left_bottom() + vec2(0.0, 13.0),
                    Align2::LEFT_CENTER,
                    sample.name,
                    FontId::proportional(14.0),
                    INK,
                );
                let (tier, allowance) = sample.rating;
                p.text(
                    rect.right_bottom() + vec2(0.0, 13.0),
                    Align2::RIGHT_CENTER,
                    format!("Тир {tier} · допуск {allowance}/6"),
                    FontId::proportional(12.0),
                    INK,
                );
                p.text(
                    rect.left_bottom() + vec2(0.0, 30.0),
                    Align2::LEFT_CENTER,
                    if self.fonts.contains(sample.face) {
                        sample.face
                    } else {
                        "Запасной шрифт egui"
                    },
                    FontId::monospace(11.0),
                    MUTED,
                );
            }
        }
    }

    fn specimen(&self, p: &Painter, rect: Rect, index: usize, face: impl Fn(f32) -> FontId) {
        let at = |x, y| rect.min + rect.size() * vec2(x, y);
        let size = rect.width() / 417.0;
        let caption = FontId::proportional(14.0 * size);
        match index {
            0 => {
                let ink = Color32::from_rgb(48, 26, 19);
                let red = Color32::from_rgb(203, 62, 35);
                p.rect_filled(rect, 0, Color32::from_rgb(248, 227, 194));
                for radius in [80.0, 99.0, 118.0, 137.0] {
                    p.circle_stroke(at(0.90, 0.50), radius * size, Stroke::new(1.65 * size, red));
                }
                p.circle_filled(at(1.015, 0.53), 67.0 * size, red);
                p.vline(
                    at(0.056, 0.0).x,
                    at(0.0, 0.12).y..=at(0.0, 0.87).y,
                    Stroke::new(2.0, red),
                );
                p.text(at(0.093, 0.30), Align2::LEFT_CENTER, FIRST, face(56.0), ink);
                p.text(
                    at(0.087, 0.57),
                    Align2::LEFT_CENTER,
                    SECOND,
                    face(71.0),
                    ink,
                );
                p.text(at(0.096, 0.825), Align2::LEFT_CENTER, LINE, caption, ink);
            }
            1 => {
                let base = Color32::from_rgb(16, 33, 44);
                let grid = Color32::from_rgb(43, 80, 87);
                let mint = Color32::from_rgb(195, 255, 123);
                p.rect_filled(rect, 0, base);
                for i in 1..20 {
                    let x = i as f32 / 20.0;
                    p.line_segment([at(x, 0.0), at(x, 1.0)], Stroke::new(1.0, grid));
                }
                for i in 1..11 {
                    let y = i as f32 / 11.0;
                    p.line_segment([at(0.0, y), at(1.0, y)], Stroke::new(1.0, grid));
                }
                p.circle_stroke(
                    at(0.836, 0.407),
                    47.0 * size,
                    Stroke::new(1.0, mint.gamma_multiply(0.52)),
                );
                p.circle_stroke(at(0.836, 0.407), 32.0 * size, Stroke::new(1.0, mint));
                p.circle_filled(at(0.836, 0.407), 3.0 * size, mint);
                p.add(Shape::line(
                    vec![
                        at(0.064, 0.13),
                        at(0.064, 0.70),
                        at(0.62, 0.70),
                        at(0.716, 0.53),
                    ],
                    Stroke::new(1.5, mint),
                ));
                p.text(
                    at(0.095, 0.29),
                    Align2::LEFT_CENTER,
                    FIRST,
                    face(47.0),
                    Color32::WHITE,
                );
                p.text(
                    at(0.095, 0.51),
                    Align2::LEFT_CENTER,
                    SECOND,
                    face(52.0),
                    mint,
                );
                p.text(
                    at(0.095, 0.81),
                    Align2::LEFT_CENTER,
                    LINE,
                    caption,
                    Color32::from_rgb(201, 226, 221),
                );
            }
            2 => {
                let cobalt = Color32::from_rgb(46, 60, 225);
                let navy = Color32::from_rgb(26, 35, 113);
                let white = Color32::from_rgb(248, 250, 216);
                p.rect_filled(rect, 0, cobalt);
                p.add(Shape::convex_polygon(
                    vec![at(0.49, 0.0), at(1.0, 0.0), at(1.0, 0.86), at(0.74, 0.86)],
                    navy,
                    Stroke::NONE,
                ));
                p.text(
                    at(0.053, 0.24),
                    Align2::LEFT_CENTER,
                    FIRST,
                    face(78.0),
                    white,
                );
                p.text(
                    at(0.951, 0.59),
                    Align2::RIGHT_CENTER,
                    SECOND,
                    face(100.0),
                    white,
                );
                p.line_segment(
                    [at(0.055, 0.843), at(0.944, 0.843)],
                    Stroke::new(1.0, white),
                );
                p.text(at(0.057, 0.92), Align2::LEFT_CENTER, LINE, caption, white);
            }
            3 => {
                let rose = Color32::from_rgb(255, 215, 235);
                vertical_gradient(
                    p,
                    rect,
                    Color32::from_rgb(27, 19, 51),
                    Color32::from_rgb(73, 25, 75),
                );
                for offset in [-0.047, 0.0, 0.047] {
                    p.add(Shape::ellipse_stroke(
                        at(0.5 + offset, 0.40),
                        rect.size() * vec2(0.353, 0.283),
                        Stroke::new(1.0, rose.gamma_multiply(0.38)),
                    ));
                }
                for (x, y) in [(0.185, 0.27), (0.81, 0.22), (0.773, 0.56)] {
                    p.add(Shape::convex_polygon(
                        vec![
                            at(x, y - 0.027),
                            at(x + 0.014, y),
                            at(x, y + 0.027),
                            at(x - 0.014, y),
                        ],
                        rose,
                        Stroke::NONE,
                    ));
                }
                p.text(
                    at(0.49, 0.407),
                    Align2::CENTER_CENTER,
                    TITLE,
                    face(49.0),
                    rose,
                );
                p.text(at(0.50, 0.769), Align2::CENTER_CENTER, LINE, caption, rose);
            }
            4 => {
                let ratio = self.photo.aspect_ratio() / rect.aspect_ratio();
                let crop = vec2(ratio.recip().min(1.0), ratio.min(1.0));
                p.image(
                    self.photo.id(),
                    rect,
                    Rect::from_center_size(pos2(0.5, 0.5), crop),
                    Color32::WHITE,
                );
                vertical_gradient(
                    p,
                    Rect::from_min_max(at(0.0, 0.22), rect.max),
                    Color32::TRANSPARENT,
                    Color32::from_rgb(9, 13, 20),
                );
                let white = Color32::from_rgb(249, 250, 245);
                p.line_segment(
                    [at(0.063, 0.84), at(0.063, 0.52)],
                    Stroke::new(2.0, Color32::from_rgb(247, 185, 110)),
                );
                p.text(
                    at(0.092, 0.59),
                    Align2::LEFT_CENTER,
                    TITLE,
                    face(53.0),
                    white,
                );
                p.text(at(0.095, 0.8), Align2::LEFT_CENTER, LINE, caption, white);
            }
            _ => {
                let red = Color32::from_rgb(212, 55, 55);
                let blue = Color32::from_rgb(29, 39, 96);
                p.rect_filled(rect, 0, Color32::from_rgb(246, 222, 156));
                p.add(Shape::convex_polygon(
                    vec![at(0.54, 0.0), at(1.0, 0.0), at(1.0, 0.79), at(0.69, 0.58)],
                    red,
                    Stroke::NONE,
                ));
                for x in 0..34 {
                    for y in 0..15 {
                        let pos = at(0.016 + x as f32 * 0.029, 0.024 + y as f32 * 0.067);
                        p.circle_filled(
                            pos,
                            (1.0 + ((x + y * 3) % 5) as f32 * 0.16) * size,
                            blue.gamma_multiply(0.32),
                        );
                    }
                }
                for (text, position, size_pt) in [
                    (FIRST, at(0.105, 0.178), 62.0),
                    (SECOND, at(0.227, 0.449), 71.0),
                ] {
                    for (offset, ink) in [
                        (vec2(4.0, 5.0) * size, Color32::from_rgb(251, 161, 113)),
                        (Vec2::ZERO, blue),
                    ] {
                        let galley = p.layout_no_wrap(text.to_owned(), face(size_pt), ink);
                        p.add(
                            egui::epaint::TextShape::new(position + offset, galley, ink)
                                .with_angle(-0.12),
                        );
                    }
                }
                p.text(at(0.932, 0.874), Align2::RIGHT_CENTER, LINE, caption, blue);
            }
        }
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        let styles = [
            (
                "Редакционные",
                Color32::from_rgb(254, 250, 237),
                Color32::from_rgb(173, 58, 31),
            ),
            (
                "Компактные",
                Color32::from_rgb(17, 28, 44),
                Color32::from_rgb(195, 255, 123),
            ),
            (
                "Сегментные",
                Color32::from_rgb(225, 226, 249),
                Color32::from_rgb(58, 42, 155),
            ),
            (
                "Папки",
                Color32::from_rgb(226, 229, 232),
                Color32::from_rgb(91, 112, 131),
            ),
            ("Текстовые", Color32::from_rgb(255, 255, 252), INK),
            (
                "С точкой",
                Color32::from_rgb(233, 240, 230),
                Color32::from_rgb(26, 109, 76),
            ),
        ];
        let width = ui.available_width();
        let header = ui.allocate_space(vec2(width, 43.0)).1;
        let p = ui.painter();
        p.hline(header.x_range(), header.top() + 5.0, Stroke::new(1.0, RULE));
        p.text(
            header.left_center(),
            Align2::LEFT_CENTER,
            "Вкладки",
            self.font(&SPECIMENS[0], 21.0),
            INK,
        );
        p.text(
            header.right_center(),
            Align2::RIGHT_CENTER,
            "Выбор синхронизирован",
            FontId::proportional(13.0),
            MUTED,
        );
        let columns = if width >= 930.0 { 3 } else { 1 };
        let gap = 16.0;
        let section_w = (width - gap * (columns - 1) as f32) / columns as f32;
        let mut selection = None;
        for (row_index, row_styles) in styles.chunks(columns).enumerate() {
            let row = ui.allocate_space(vec2(width, 84.0)).1;
            for (column, &(name, panel, accent)) in row_styles.iter().enumerate() {
                let mode = row_index * columns + column;
                let sample = Rect::from_min_size(
                    row.min + vec2(column as f32 * (section_w + gap), 0.0),
                    vec2(section_w, 74.0),
                );
                let p = ui.painter_at(sample);
                p.rect_filled(sample, 0, panel);
                p.text(
                    sample.left_top() + vec2(10.0, 7.0),
                    Align2::LEFT_TOP,
                    name,
                    FontId::proportional(11.0),
                    if mode == 1 {
                        Color32::from_rgb(170, 194, 222)
                    } else {
                        MUTED
                    },
                );
                let strip = sample.shrink2(vec2(8.0, 0.0));
                let tab_w = strip.width() / 3.0;
                if mode == 3 {
                    p.rect_filled(
                        Rect::from_min_max(
                            pos2(strip.left(), sample.top() + 58.0),
                            pos2(strip.right(), sample.bottom() - 5.0),
                        ),
                        0,
                        Color32::WHITE,
                    );
                    p.hline(
                        strip.x_range(),
                        sample.top() + 58.0,
                        Stroke::new(1.0, accent),
                    );
                }
                for (item, title) in ["Избранное", TITLE, "Архив"].into_iter().enumerate()
                {
                    let rect = Rect::from_min_size(
                        pos2(strip.left() + tab_w * item as f32, sample.top() + 25.0),
                        vec2(tab_w, 34.0),
                    )
                    .shrink2(vec2(2.0, 0.0));
                    let response = ui
                        .interact(
                            rect,
                            ui.make_persistent_id(("type-tabs", mode, item)),
                            egui::Sense::click(),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    let active = self.active_tab == item;
                    response.widget_info(|| {
                        WidgetInfo::selected(WidgetType::SelectableLabel, true, active, title)
                    });
                    if response.clicked() {
                        selection = Some(item);
                    }
                    let hot = response.hovered() || response.has_focus();
                    if mode == 2 && (active || hot) {
                        p.rect_filled(
                            rect.shrink(1.0),
                            16,
                            if active {
                                accent
                            } else {
                                Color32::from_rgb(211, 208, 242)
                            },
                        );
                    }
                    if mode == 0 {
                        p.line_segment(
                            [rect.left_bottom(), rect.right_bottom()],
                            Stroke::new(
                                if active { 2.5 } else { 1.0 },
                                if active || hot { accent } else { RULE },
                            ),
                        );
                    } else if mode == 1 && (active || hot) {
                        let slot = rect.shrink2(vec2(1.0, 4.0));
                        p.rect_filled(slot, 0, Color32::from_rgb(39, 58, 69));
                        p.vline(slot.left() + 1.0, slot.y_range(), Stroke::new(2.0, accent));
                    } else if mode == 3 && active {
                        let tab = rect.shrink2(vec2(1.0, 0.0));
                        let fill = Color32::WHITE;
                        p.rect_filled(tab, 4, fill);
                        p.rect_stroke(tab, 4, Stroke::new(1.0, accent), StrokeKind::Inside);
                        p.rect_filled(
                            Rect::from_min_max(
                                tab.left_bottom() + vec2(1.0, -4.0),
                                tab.right_bottom() + vec2(-1.0, 1.0),
                            ),
                            0,
                            fill,
                        );
                    } else if mode == 3 && hot {
                        let hover = Rect::from_min_max(
                            rect.min + vec2(1.0, 0.0),
                            pos2(rect.right() - 1.0, sample.top() + 57.5),
                        );
                        let radius = egui::CornerRadius {
                            nw: 4,
                            ne: 4,
                            sw: 0,
                            se: 0,
                        };
                        p.rect_filled(hover, radius, panel.lerp_to_gamma(accent, 0.20));
                    } else if mode == 5 && (active || hot) {
                        p.circle_filled(
                            rect.center_bottom() + vec2(0.0, 3.0),
                            if active { 3.0 } else { 2.0 },
                            accent,
                        );
                    }
                    let color = match (mode, active) {
                        (1, true) => accent,
                        (1, false) => Color32::from_rgb(214, 226, 241),
                        (2, true) => Color32::WHITE,
                        (0, true) => accent,
                        (4, false) if !hot => MUTED,
                        (5, true) => accent,
                        _ => INK,
                    };
                    let font = match mode {
                        0 => self.font(&SPECIMENS[0], 13.0),
                        1 => self.font(&SPECIMENS[1], 13.0),
                        2 => self.font(&SPECIMENS[4], 15.0),
                        4 if active => self.font(&typography::SAMPLES[0], 14.0),
                        _ => FontId::proportional(14.0),
                    };
                    p.text(rect.center(), Align2::CENTER_CENTER, title, font, color);
                }
            }
        }
        if let Some(index) = selection {
            self.active_tab = index;
            ui.ctx().request_repaint();
        }
    }
}

pub(crate) fn vertical_gradient(p: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = Mesh::default();
    for (point, color) in [
        (rect.left_top(), top),
        (rect.right_top(), top),
        (rect.right_bottom(), bottom),
        (rect.left_bottom(), bottom),
    ] {
        mesh.colored_vertex(point, color);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(Shape::mesh(mesh));
}

impl eframe::App for Sheet {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

pub fn run_graphics() -> eframe::Result {
    eframe::run_native(
        "Графика · Мира",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1380.0, 820.0])
                .with_min_inner_size([700.0, 500.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Ok(Box::new(Sheet::new(&cc.egui_ctx)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_hover_fills_the_tab_up_to_the_rail_without_a_border() {
        let ctx = egui::Context::default();
        let mut sheet = Sheet::new(&ctx);
        sheet.view = View::Tabs;
        let mut frame = |events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1380.0, 820.0))),
                    events,
                    ..Default::default()
                },
                |ui| sheet.show(ui),
            );
            output.textures_delta.clear();
            output.shapes
        };
        let panel = Color32::from_rgb(226, 229, 232);
        let sample = frame(Vec::new())
            .iter()
            .find_map(|shape| match &shape.shape {
                Shape::Rect(shape) if shape.fill == panel => Some(shape.rect),
                _ => None,
            })
            .expect("folder tab sample is visible");
        let pointer = sample.min + vec2(sample.width() / 6.0, 42.0);
        let shapes = frame(vec![egui::Event::PointerMoved(pointer)]);
        let fill = panel.lerp_to_gamma(Color32::from_rgb(91, 112, 131), 0.20);
        let hover = shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                Shape::Rect(shape) if shape.fill == fill && shape.rect.contains(pointer) => {
                    Some(shape)
                }
                _ => None,
            })
            .expect("hovered folder has a tinted fill");
        assert_eq!(hover.stroke, Stroke::NONE);
        assert_eq!(hover.rect.bottom(), sample.top() + 57.5);
    }
}
