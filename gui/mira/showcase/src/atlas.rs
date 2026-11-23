use std::{io, path::PathBuf, process::Command};

use eframe::egui::{
    self, Align, Align2, Color32, CursorIcon, FontFamily, FontId, Key, Painter, Pos2, Rect, Sense,
    Stroke, StrokeKind, TextureHandle, UiBuilder, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::animation::Animation;
use crate::graphics::Sheet;
use crate::infographic::Infographic;
use crate::{SERIF, consume_key, normalize};

const SHEET: Color32 = Color32::from_rgb(248, 245, 235);
const INK: Color32 = Color32::from_rgb(39, 49, 43);
const QUIET: Color32 = Color32::from_rgb(90, 102, 90);
const RULE: Color32 = Color32::from_rgb(208, 211, 194);
const ACCENT: Color32 = Color32::from_rgb(73, 99, 74);
const MAT: Color32 = Color32::from_rgb(236, 234, 222);

struct Style {
    slug: &'static str,
    title: &'static str,
    rating: Option<(&'static str, u8)>,
    cue: &'static str,
    preview: &'static [u8],
}

const STYLES: [Style; 11] = [
    Style {
        slug: "nolox",
        title: "Nolox",
        rating: None,
        cue: "Очередь задач со световыми фазами",
        preview: include_bytes!("../assets/atlas/nolox.jpg"),
    },
    Style {
        slug: "kalka",
        title: "Калька",
        rating: Some(("S+", 3)),
        cue: "Световой стол и сгиб между снимками",
        preview: include_bytes!("../assets/atlas/kalka.jpg"),
    },
    Style {
        slug: "semaphore",
        title: "Семафор",
        rating: Some(("S+", 3)),
        cue: "Действия как маршрут",
        preview: include_bytes!("../assets/atlas/semaphore.jpg"),
    },
    Style {
        slug: "azimuth",
        title: "Азимут",
        rating: Some(("S", 3)),
        cue: "Две оси на кобальтовой сетке",
        preview: include_bytes!("../assets/atlas/azimuth.jpg"),
    },
    Style {
        slug: "orbit",
        title: "Орбита",
        rating: Some(("S", 3)),
        cue: "Круговое меню у указателя",
        preview: include_bytes!("../assets/atlas/orbit.jpg"),
    },
    Style {
        slug: "lozha",
        title: "Ложа",
        rating: Some(("S", 2)),
        cue: "Театральный свет и фактура",
        preview: include_bytes!("../assets/atlas/lozha.jpg"),
    },
    Style {
        slug: "pult",
        title: "Пульт",
        rating: Some(("S", 2)),
        cue: "Приборные ручки и ряд клавиш",
        preview: include_bytes!("../assets/atlas/pult.jpg"),
    },
    Style {
        slug: "verstka",
        title: "Вёрстка",
        rating: Some(("A", 6)),
        cue: "Редакционная сетка и крупный набор",
        preview: include_bytes!("../assets/atlas/verstka.jpg"),
    },
    Style {
        slug: "lumen",
        title: "Люмен",
        rating: Some(("B", 5)),
        cue: "Компактный лаунчер с поиском",
        preview: include_bytes!("../assets/atlas/lumen.jpg"),
    },
    Style {
        slug: "herbarium",
        title: "Гербарий",
        rating: Some(("B", 2)),
        cue: "Холст с карточками и связями",
        preview: include_bytes!("../assets/atlas/herbarium.jpg"),
    },
    Style {
        slug: "contact",
        title: "Контакт",
        rating: Some(("C", 3)),
        cue: "Кинолента для сравнения кадров",
        preview: include_bytes!("../assets/atlas/contact.jpg"),
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum AtlasView {
    Styles,
    Graphics,
    Kinetics,
    Animation,
}

impl AtlasView {
    fn restart_value(self) -> &'static str {
        match self {
            Self::Styles => "styles",
            Self::Graphics => "graphics",
            Self::Kinetics => "kinetics",
            Self::Animation => "animation",
        }
    }

    fn restart_view() -> Self {
        match std::env::var("MIRA_ATLAS_RESTART_VIEW").as_deref() {
            Ok("graphics") => Self::Graphics,
            Ok("kinetics") => Self::Kinetics,
            Ok("animation") => Self::Animation,
            _ => Self::Styles,
        }
    }
}

struct Atlas {
    query: String,
    selected: Option<usize>,
    textures: Vec<TextureHandle>,
    error: Option<String>,
    restart_error: Option<String>,
    view: AtlasView,
    graphics: Sheet,
    kinetics: Animation,
    infographic: Infographic,
}

impl Atlas {
    fn new(ctx: &egui::Context) -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.override_text_color = Some(INK);
        visuals.weak_text_color = Some(QUIET);
        ctx.set_visuals(visuals);
        let textures = STYLES
            .iter()
            .map(|style| {
                let pixels =
                    image::load_from_memory_with_format(style.preview, image::ImageFormat::Jpeg)
                        .expect("embedded atlas preview must be a valid JPEG")
                        .into_rgba8();
                ctx.load_texture(
                    style.slug,
                    egui::ColorImage::from_rgba_unmultiplied(
                        [pixels.width() as usize, pixels.height() as usize],
                        pixels.as_raw(),
                    ),
                    egui::TextureOptions::LINEAR,
                )
            })
            .collect();
        Self {
            query: String::new(),
            selected: Some(0),
            textures,
            error: None,
            restart_error: None,
            view: AtlasView::Styles,
            graphics: Sheet::new(ctx),
            kinetics: Animation::default(),
            infographic: Infographic::default(),
        }
    }

    fn reconcile_selection(&mut self, visible: &[usize]) {
        if self.selected.is_none_or(|index| !visible.contains(&index)) {
            self.selected = visible.first().copied();
        }
    }

    fn launch(&mut self, index: usize) {
        let style = &STYLES[index];
        self.spawn_window(&["--style", style.slug], style.title);
    }

    fn spawn_window(&mut self, args: &[&str], title: &str) {
        let launch = std::env::current_exe().and_then(|path| Command::new(path).args(args).spawn());
        self.error = launch
            .err()
            .map(|error| format!("Не удалось открыть «{title}» · {error}"));
    }

    fn restart(&mut self, ctx: &egui::Context) {
        let result = atlas_executable().and_then(|path| {
            Command::new(path)
                .env("MIRA_ATLAS_RESTART_VIEW", self.view.restart_value())
                .spawn()
        });
        match result {
            Ok(_) => {
                self.infographic.stop();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(error) => {
                self.restart_error = Some(format!("Не удалось перезапустить МИРУ · {error}"))
            }
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let window = ui.available_rect_before_wrap();
        let margin = (window.width() * 0.031).clamp(23.0, 42.0);
        let left = window.left() + margin;
        let right = window.right() - margin;
        let top = window.top();
        let painter = ui.painter_at(window);
        painter.rect_filled(window, 0, SHEET);

        let wide = window.width() >= 920.0;
        painter.text(
            pos2(left - 2.0, top + 53.0),
            Align2::LEFT_CENTER,
            "МИРА",
            serif(if wide { 65.0 } else { 51.0 }),
            INK,
        );
        let label_left = left + if wide { 238.0 } else { 184.0 };
        painter.vline(
            label_left - 12.0,
            top + 35.0..=top + 72.0,
            Stroke::new(1.0, RULE),
        );
        painter.text(
            pos2(label_left, top + 53.0),
            Align2::LEFT_CENTER,
            "Атлас интерфейсов",
            serif(if wide { 24.0 } else { 18.0 }),
            INK,
        );
        let tab_y = top + 92.0;
        let restart = ui
            .put(
                Rect::from_min_size(pos2(right - 209.0, tab_y - 2.0), vec2(209.0, 28.0)),
                egui::Button::new("Перезапустить МИРУ · Ctrl+R"),
            )
            .on_hover_text("Открыть актуальную сборку на той же вкладке");
        if restart.clicked() || consume_key(ui.ctx(), Key::R, true) > 0 {
            self.restart(ui.ctx());
        }
        if let Some(error) = &self.restart_error {
            painter.text(
                pos2(right, tab_y - 10.0),
                Align2::RIGHT_BOTTOM,
                error,
                FontId::proportional(12.0),
                Color32::from_rgb(146, 56, 47),
            );
        }
        painter.hline(left..=right, tab_y + 30.0, Stroke::new(1.0, RULE));
        let mut focus_in_tabs = false;
        for (offset, width, label, target) in [
            (0.0, 79.0, "Стили", AtlasView::Styles),
            (88.0, 94.0, "Графика", AtlasView::Graphics),
            (191.0, 104.0, "Кинетика", AtlasView::Kinetics),
            (304.0, 104.0, "Анимация", AtlasView::Animation),
        ] {
            let rect = Rect::from_min_size(pos2(left + offset, tab_y), vec2(width, 30.0));
            let response = ui
                .interact(
                    rect,
                    ui.make_persistent_id(("atlas-view", target)),
                    Sense::click(),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            if response.clicked() {
                if target == AtlasView::Animation && self.view != target {
                    self.infographic.restart(ui.input(|input| input.time));
                } else if self.view == AtlasView::Animation && target != self.view {
                    self.infographic.stop();
                }
                self.view = target;
                ui.ctx().request_repaint();
            }
            let active = self.view == target;
            response.widget_info(|| {
                WidgetInfo::selected(WidgetType::SelectableLabel, true, active, label)
            });
            focus_in_tabs |= response.has_focus();
            let highlight = active || response.hovered() || response.has_focus();
            let ink = if highlight { ACCENT } else { QUIET };
            if highlight {
                painter.hline(
                    rect.x_range(),
                    rect.bottom(),
                    Stroke::new(if active { 2.5 } else { 1.0 }, ink),
                );
            }
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(16.0),
                ink,
            );
        }
        if self.view != AtlasView::Styles {
            let content =
                Rect::from_min_max(pos2(window.left(), tab_y + 31.0), window.right_bottom());
            ui.scope_builder(
                UiBuilder::new().id_salt(self.view).max_rect(content),
                |ui| {
                    ui.set_clip_rect(content);
                    match self.view {
                        AtlasView::Graphics => self.graphics.show_in_atlas(ui, margin),
                        AtlasView::Kinetics => self.kinetics.show(ui, serif(31.0)),
                        AtlasView::Animation => self.infographic.show(ui, margin),
                        AtlasView::Styles => unreachable!(),
                    }
                },
            );
            return;
        }

        let search_id = egui::Id::new("atlas-style-search");
        if consume_key(ui.ctx(), Key::F, true) > 0 {
            ui.memory_mut(|memory| memory.request_focus(search_id));
        }
        let enter = !focus_in_tabs && consume_key(ui.ctx(), Key::Enter, false) > 0;
        let search_width = if wide { 253.0 } else { 204.0 };
        let search = Rect::from_min_size(
            pos2(right - search_width, top + 39.0),
            vec2(search_width, 29.0),
        );
        ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
            ui.spacing_mut().interact_size.y = 28.0;
            let visuals = ui.visuals_mut();
            visuals.text_edit_bg_color = Some(SHEET);
            visuals.widgets.inactive.bg_stroke = Stroke::NONE;
            visuals.widgets.hovered.bg_stroke = Stroke::NONE;
            visuals.widgets.active.bg_stroke = Stroke::NONE;
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .id(search_id)
                    .desired_width(search.width() - 9.0)
                    .hint_text("Поиск стиля · Ctrl+F"),
            );
            painter.hline(
                search.x_range(),
                search.bottom(),
                Stroke::new(1.0, if response.has_focus() { ACCENT } else { RULE }),
            );
        });

        let mut visible = matching(&self.query);
        if consume_key(ui.ctx(), Key::Escape, false) > 0 && !self.query.is_empty() {
            self.query.clear();
            visible = matching(&self.query);
        }
        self.reconcile_selection(&visible);
        let gallery_top = top + 151.0;
        let count = count_label(visible.len());
        painter.text(
            pos2(left, top + 137.0),
            Align2::LEFT_CENTER,
            count,
            FontId::proportional(12.0),
            QUIET,
        );
        if let Some(error) = &self.error {
            painter.text(
                pos2(right, top + 137.0),
                Align2::RIGHT_CENTER,
                error,
                FontId::proportional(12.0),
                Color32::from_rgb(146, 56, 47),
            );
        } else if !visible.is_empty() {
            painter.text(
                pos2(right, top + 137.0),
                Align2::RIGHT_CENTER,
                "Стрелки — выбор  ·  Enter — открыть",
                FontId::proportional(12.0),
                QUIET,
            );
        }

        let view = Rect::from_min_max(
            pos2(left, gallery_top),
            pos2(right + 8.0, window.bottom() - 12.0),
        );
        let columns = column_count(right - left);
        let dx = if ui.memory(|memory| memory.has_focus(search_id)) {
            0
        } else {
            key_delta(ui.ctx(), Key::ArrowRight, Key::ArrowLeft)
        };
        let dy = key_delta(ui.ctx(), Key::ArrowDown, Key::ArrowUp);
        let moved = dx != 0 || dy != 0;
        if moved
            && let Some(selected) = self.selected
            && let Some(row_index) = visible.iter().position(|&index| index == selected)
        {
            let next = row_index
                .saturating_add_signed(dx + dy * columns as isize)
                .min(visible.len() - 1);
            self.selected = Some(visible[next]);
        }
        if visible.is_empty() {
            painter.text(
                view.center() - vec2(0.0, 30.0),
                Align2::CENTER_CENTER,
                "Стили не найдены",
                serif(27.0),
                QUIET,
            );
            return;
        }

        let move_focus = moved
            && ui.memory(|memory| memory.focused().is_some_and(|focused| focused != search_id));
        let clicked = ui
            .scope_builder(UiBuilder::new().max_rect(view), |ui| {
                ui.set_clip_rect(view);
                egui::ScrollArea::vertical()
                    .id_salt("atlas-gallery")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.gallery(ui, &visible, columns, moved, move_focus)
                    })
                    .inner
            })
            .inner;
        if let Some(index) = clicked.or_else(|| self.selected.filter(|_| enter)) {
            self.selected = Some(index);
            self.launch(index);
        }
    }

    fn gallery(
        &mut self,
        ui: &mut egui::Ui,
        visible: &[usize],
        columns: usize,
        moved: bool,
        move_focus: bool,
    ) -> Option<usize> {
        let gap = 14.0;
        let width = (ui.available_width() - gap * (columns - 1) as f32) / columns as f32;
        let height = width * 0.61 + 68.0;
        let mut clicked = None;
        for row in visible.chunks(columns) {
            let row_rect = ui.allocate_space(vec2(ui.available_width(), height)).1;
            for (slot, &index) in row.iter().enumerate() {
                let card = Rect::from_min_size(
                    row_rect.min + vec2(slot as f32 * (width + gap), 0.0),
                    vec2(width, height),
                );
                let style = &STYLES[index];
                let response = ui
                    .interact(
                        card,
                        ui.make_persistent_id(("atlas-card", style.slug)),
                        Sense::click(),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);
                response.widget_info(|| {
                    let mut label = format!("Открыть «{}»", style.title);
                    if let Some((grade, admission)) = style.rating {
                        label.push_str(&format!(". Тир {grade}, допуск {admission} из 6"));
                    }
                    WidgetInfo::labeled(WidgetType::Button, true, label)
                });
                if response.has_focus() && !moved {
                    self.selected = Some(index);
                }
                let active = self.selected == Some(index);
                if moved && active {
                    response.scroll_to_me(Some(Align::Center));
                    if move_focus {
                        response.request_focus();
                    }
                }
                paint_card(
                    ui,
                    card,
                    style,
                    &self.textures[index],
                    active,
                    response.hovered() || response.has_focus(),
                );
                if response.clicked() {
                    clicked = Some(index);
                }
            }
            ui.add_space(gap);
        }
        clicked
    }
}

fn matching(query: &str) -> Vec<usize> {
    let query = normalize(query);
    STYLES
        .iter()
        .enumerate()
        .filter_map(|(index, style)| {
            let haystack = normalize(&format!(
                "{} {} {} {}",
                style.slug,
                style.title,
                style.rating.map_or("", |(grade, _)| grade),
                style.cue
            ));
            query
                .split_whitespace()
                .all(|word| haystack.contains(word))
                .then_some(index)
        })
        .collect()
}

fn key_delta(ctx: &egui::Context, forward: Key, backward: Key) -> isize {
    consume_key(ctx, forward, false) as isize - consume_key(ctx, backward, false) as isize
}

fn column_count(width: f32) -> usize {
    (((width + 14.0) / 261.0).floor() as usize).clamp(2, 4)
}

fn count_label(count: usize) -> String {
    match count {
        1 => "1 стиль".to_owned(),
        2..=4 => format!("{count} стиля"),
        _ => format!("{count} стилей"),
    }
}

fn paint_card(
    ui: &egui::Ui,
    card: Rect,
    style: &Style,
    texture: &TextureHandle,
    active: bool,
    hot: bool,
) {
    let p = ui.painter_at(card);
    let fill = if hot {
        Color32::from_rgb(239, 241, 231)
    } else {
        Color32::from_rgb(255, 253, 245)
    };
    p.rect_filled(card, 0, fill);
    let picture = Rect::from_min_max(
        card.min + vec2(6.0, 6.0),
        pos2(card.right() - 6.0, card.bottom() - 66.0),
    );
    p.rect_filled(picture, 0, MAT);
    draw_preview(&p, picture, texture);

    let title_y = card.bottom() - 42.0;
    p.text(
        pos2(card.left() + 12.0, title_y),
        Align2::LEFT_CENTER,
        style.title,
        serif(23.0),
        INK,
    );
    if let Some((grade, admission)) = style.rating {
        let (mark, ink) = grade_ink(grade);
        let mark_rect =
            Rect::from_center_size(pos2(card.right() - 42.0, title_y), vec2(62.0, 22.0));
        p.rect_filled(mark_rect, 0, mark);
        p.text(
            mark_rect.center(),
            Align2::CENTER_CENTER,
            format!("{grade} · {admission}/6"),
            FontId::monospace(11.0),
            ink,
        );
    }

    let cue = p.layout(
        style.cue.to_owned(),
        FontId::proportional(11.0),
        QUIET,
        card.width() - 24.0,
    );
    p.galley(pos2(card.left() + 12.0, card.bottom() - 23.0), cue, QUIET);
    p.rect_stroke(
        card,
        0,
        Stroke::new(
            if active || hot { 1.5 } else { 1.0 },
            if active || hot { ACCENT } else { RULE },
        ),
        StrokeKind::Inside,
    );
}

fn draw_preview(p: &Painter, rect: Rect, texture: &TextureHandle) {
    let aspect = texture.aspect_ratio();
    let width = rect.width().min(rect.height() * aspect);
    let height = width / aspect;
    let fitted = Rect::from_center_size(rect.center(), vec2(width, height));
    p.image(
        texture.id(),
        fitted,
        Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn grade_ink(grade: &str) -> (Color32, Color32) {
    if grade.starts_with("S+") {
        (
            Color32::from_rgb(239, 218, 165),
            Color32::from_rgb(97, 67, 26),
        )
    } else {
        match grade.as_bytes()[0] {
            b'S' => (
                Color32::from_rgb(229, 218, 235),
                Color32::from_rgb(81, 48, 97),
            ),
            b'A' => (
                Color32::from_rgb(210, 231, 211),
                Color32::from_rgb(46, 82, 52),
            ),
            b'B' => (
                Color32::from_rgb(220, 230, 235),
                Color32::from_rgb(51, 77, 89),
            ),
            _ => (
                Color32::from_rgb(236, 229, 216),
                Color32::from_rgb(99, 84, 62),
            ),
        }
    }
}

fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SERIF.into()))
}

impl eframe::App for Atlas {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

fn atlas_executable() -> io::Result<PathBuf> {
    let current = std::env::current_exe()?;
    #[cfg(all(windows, not(debug_assertions)))]
    {
        let name = current
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if name == "mira-atlas.exe" || name.starts_with("mira-atlas-running-") {
            let published = current.with_file_name("mira-atlas.exe");
            if published.is_file() {
                return atlas_snapshot(&published);
            }
        }
    }
    Ok(current)
}

#[cfg(all(windows, not(debug_assertions)))]
fn atlas_snapshot(published: &std::path::Path) -> io::Result<PathBuf> {
    use std::{fs, time::UNIX_EPOCH};

    // Run a snapshot so Windows leaves the published executable writable between builds.
    let metadata = published.metadata()?;
    let stamp = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?;
    let snapshot = published.with_file_name(format!(
        "mira-atlas-running-{:x}-{:x}.exe",
        stamp.as_nanos(),
        metadata.len()
    ));
    if snapshot.is_file() {
        return Ok(snapshot);
    }

    let temporary = snapshot.with_extension(format!("{}.tmp", std::process::id()));
    let write = (|| {
        if fs::copy(published, &temporary)? != metadata.len()
            || published.metadata()?.modified()? != metadata.modified()?
        {
            return Err(io::Error::other("Сборка МИРЫ ещё обновляется"));
        }
        fs::rename(&temporary, &snapshot)
    })();
    if let Err(error) = write {
        let _ = fs::remove_file(&temporary);
        if !snapshot.is_file() {
            return Err(error);
        }
    }
    Ok(snapshot)
}

pub fn run_atlas() -> eframe::Result {
    #[cfg(all(windows, not(debug_assertions)))]
    {
        let current =
            std::env::current_exe().map_err(|error| eframe::Error::AppCreation(error.into()))?;
        if current
            .file_name()
            .is_some_and(|name| name == "mira-atlas.exe")
        {
            let result = atlas_executable().and_then(|path| Command::new(path).spawn());
            if result.is_ok() {
                return Ok(());
            }
            // The atlas can still start if its installation directory is read-only.
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 860.0])
            .with_min_inner_size([700.0, 535.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Мира · Атлас интерфейсов",
        options,
        Box::new(|cc| {
            let mut atlas = Atlas::new(&cc.egui_ctx);
            atlas.view = AtlasView::restart_view();
            Ok(Box::new(atlas))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_plays_in_its_own_tab_at_minimum_window_size() {
        let ctx = egui::Context::default();
        let mut atlas = Atlas::new(&ctx);
        let click = pos2(380.0, 106.0);
        let pointer_button = |pressed| egui::Event::PointerButton {
            pos: click,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let mut frame = |seconds, events| {
            let input = egui::RawInput {
                time: Some(seconds),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(700.0, 535.0))),
                events,
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| atlas.show(ui));
            output.textures_delta.clear();
            for primitive in ctx.tessellate(output.shapes, output.pixels_per_point) {
                if let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive
                    && mesh.texture_id != egui::TextureId::default()
                    && atlas.view == AtlasView::Animation
                {
                    let bounds = mesh.calc_bounds();
                    assert!(bounds.bottom() < 503.0);
                    assert!(bounds.top() > 123.0);
                    return atlas.view;
                }
            }
            assert_ne!(
                atlas.view,
                AtlasView::Animation,
                "clip must paint in the tab"
            );
            atlas.view
        };
        assert_eq!(frame(0.0, vec![]), AtlasView::Styles);
        frame(
            0.1,
            vec![egui::Event::PointerMoved(click), pointer_button(true)],
        );
        assert_eq!(
            frame(0.2, vec![pointer_button(false)]),
            AtlasView::Animation
        );
        assert_eq!(frame(2.3, vec![]), AtlasView::Animation);
        assert_eq!(frame(10.0, vec![]), AtlasView::Animation);
    }

    #[test]
    fn a_filtered_style_becomes_the_launch_selection() {
        let ctx = egui::Context::default();
        let mut atlas = Atlas::new(&ctx);
        let visible = matching("КАлЬкА");
        atlas.selected = Some(9);
        atlas.reconcile_selection(&visible);
        assert_eq!(visible.len(), 1);
        assert_eq!(STYLES[atlas.selected.unwrap()].slug, "kalka");
        atlas.reconcile_selection(&matching("несуществующий стиль"));
        assert!(atlas.selected.is_none());
    }
}
