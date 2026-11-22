use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontFamily, FontId, Painter, Pos2, Rect, Sense, Stroke,
    UiBuilder, pos2, vec2,
};

use crate::{App, SERIF, WORKS};

const PAPER: Color32 = Color32::from_rgb(246, 242, 232);
const PANEL: Color32 = Color32::from_rgb(237, 232, 219);
const INK: Color32 = Color32::from_rgb(34, 38, 36);
const SECONDARY: Color32 = Color32::from_rgb(94, 93, 81);
const RULE: Color32 = Color32::from_rgb(202, 195, 178);
const ACCENT: Color32 = Color32::from_rgb(151, 55, 35);
const SELECTION: Color32 = Color32::from_rgb(232, 223, 206);
const HOVER: Color32 = Color32::from_rgb(239, 233, 220);

pub(crate) fn show(ui: &mut egui::Ui, app: &mut App) {
    let window = ui.available_rect_before_wrap();
    let painter = ui.painter_at(window);
    painter.rect_filled(window, 0, PAPER);

    let margin = (window.width() * 0.042).clamp(25.0, 52.0);
    let left = window.left() + margin;
    let right = window.right() - margin;
    let top = window.top();
    let width = right - left;
    let rail_width = (width * 0.29).clamp(208.0, 350.0);
    let rail_left = right - rail_width;
    let plate_right = rail_left - 29.0;
    let shelf = window.bottom() - 45.0;
    let hero_y = top + 140.0;

    let masthead_y = top + 29.0;
    painter.circle_filled(pos2(left + 7.0, masthead_y), 6.5, ACCENT);
    painter.circle_filled(pos2(left + 9.5, masthead_y - 2.0), 2.5, PAPER);
    text(&painter, pos2(left + 21.0, masthead_y), "МИРА", 13.0, INK);
    text(
        &painter,
        pos2(left + 76.0, masthead_y),
        "/  ВЁРСТКА",
        12.0,
        SECONDARY,
    );
    let settings_button =
        Rect::from_min_size(pos2(right - 130.0, masthead_y - 14.0), vec2(130.0, 28.0));
    if text_button(ui, &painter, settings_button, "Настройки", "Ctrl+,") {
        app.open_settings();
    }
    rule(&painter, left, right, top + 51.0);

    painter.text(
        pos2(left - 2.0, top + 79.0),
        Align2::LEFT_CENTER,
        "После дождя",
        serif(if window.height() < 630.0 { 43.0 } else { 54.0 }),
        INK,
    );
    text(
        &painter,
        pos2(left + 1.0, top + 119.0),
        "ФОТОГРАФИЧЕСКАЯ КОЛЛЕКЦИЯ",
        11.0,
        SECONDARY,
    );

    let search_width = rail_width - 1.0;
    let search = Rect::from_min_size(pos2(rail_left, top + 101.0), vec2(search_width, 26.0));
    ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
        ui.set_clip_rect(search.expand(1.0));
        ui.spacing_mut().interact_size.y = 26.0;
        let visuals = ui.visuals_mut();
        visuals.text_edit_bg_color = Some(PAPER);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, RULE);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, SECONDARY);
        app.search(ui, search_width - 2.0);
    });

    let matches = app.matches();
    let rail_count = matches.len();
    let hero = Rect::from_min_max(pos2(left, hero_y), pos2(plate_right, shelf - 100.0));
    let selected = app.selected;
    if let Some(index) = selected {
        let work = &WORKS[index];
        cover(&painter, hero, &app.textures[index]);
        let open = ui
            .interact(
                hero,
                ui.make_persistent_id("editorial_preview"),
                Sense::click(),
            )
            .on_hover_cursor(CursorIcon::PointingHand);
        if open.clicked() {
            app.open_preview();
        }
        let number = Rect::from_min_size(hero.left_top() + vec2(12.0, 12.0), vec2(42.0, 24.0));
        painter.rect_filled(number, 0, PAPER);
        painter.text(
            number.center(),
            Align2::CENTER_CENTER,
            format!("{:02}", index + 1),
            mono(12.0),
            INK,
        );

        let title_y = hero.bottom() + 25.0;
        painter.text(
            pos2(left, title_y),
            Align2::LEFT_CENTER,
            work.title,
            serif(25.0),
            INK,
        );
        painter.text(
            pos2(plate_right, title_y),
            Align2::RIGHT_CENTER,
            work.year.to_string(),
            mono(13.0),
            SECONDARY,
        );
        let details_y = title_y + 23.0;
        text(
            &painter,
            pos2(left, details_y),
            &format!("{}  /  {}", work.category, work.place),
            12.0,
            ACCENT,
        );
        let description = painter.layout(
            work.description.to_owned(),
            FontId::proportional(12.0),
            SECONDARY,
            hero.width(),
        );
        painter
            .with_clip_rect(Rect::from_min_max(
                pos2(left, details_y + 11.0),
                pos2(plate_right, shelf - 3.0),
            ))
            .galley(pos2(left, details_y + 12.0), description, SECONDARY);
    } else {
        painter.rect_filled(hero, 0, PANEL);
        painter.text(
            hero.center(),
            Align2::CENTER_CENTER,
            "Ничего не найдено",
            serif(28.0),
            SECONDARY,
        );
    }

    painter.line_segment(
        [
            pos2(rail_left - 15.0, hero_y),
            pos2(rail_left - 15.0, shelf - 17.0),
        ],
        Stroke::new(1.0, RULE),
    );
    text(
        &painter,
        pos2(rail_left, hero_y + 12.0),
        "СОДЕРЖАНИЕ",
        11.0,
        INK,
    );
    painter.text(
        pos2(right, hero_y + 12.0),
        Align2::RIGHT_CENTER,
        format!("{rail_count:02} / {:02}", WORKS.len()),
        mono(11.0),
        SECONDARY,
    );

    let rows_top = hero_y + 29.0;
    let natural_height = if app.compact {
        36.0
    } else {
        ((shelf - rows_top - 20.0) / WORKS.len() as f32).clamp(40.0, 63.0)
    };
    let row_height = natural_height.min(((shelf - rows_top - 8.0) / WORKS.len() as f32).max(26.0));
    rule(&painter, rail_left, right, rows_top);
    for (slot, index) in matches.into_iter().enumerate() {
        let row = Rect::from_min_size(
            pos2(rail_left, rows_top + slot as f32 * row_height),
            vec2(rail_width, row_height),
        );
        let response = ui
            .interact(
                row,
                ui.make_persistent_id(("editorial_row", index)),
                Sense::click(),
            )
            .on_hover_cursor(CursorIcon::PointingHand);
        let work = &WORKS[index];
        let active = Some(index) == selected;
        if active {
            painter.rect_filled(row, 0, SELECTION);
            painter.rect_filled(
                Rect::from_min_size(row.min, vec2(2.0, row.height())),
                0,
                ACCENT,
            );
        } else if response.hovered() {
            painter.rect_filled(row, 0, HOVER);
        }
        rule(&painter, row.left(), row.right(), row.bottom());
        let thumb_size = (row_height - 8.0).min((rail_width - 170.0).clamp(30.0, 47.0));
        let thumb = Rect::from_min_size(
            pos2(row.left() + 27.0, row.center().y - thumb_size * 0.5),
            vec2(thumb_size, thumb_size),
        );
        painter.text(
            pos2(row.left() + 7.0, row.center().y),
            Align2::LEFT_CENTER,
            format!("{:02}", index + 1),
            mono(10.0),
            SECONDARY,
        );
        cover(&painter, thumb, &app.textures[index]);
        let label_x = thumb.right() + 9.0;
        let label_painter = painter.with_clip_rect(Rect::from_min_max(
            pos2(label_x, row.top()),
            pos2(right - 12.0, row.bottom()),
        ));
        let title_y = if app.compact {
            row.center().y
        } else {
            row.center().y - 7.0
        };
        text(
            &label_painter,
            pos2(label_x, title_y),
            work.title,
            if rail_width < 245.0 { 12.0 } else { 13.0 },
            if active { ACCENT } else { INK },
        );
        if !app.compact {
            text(
                &label_painter,
                pos2(label_x, row.center().y + 9.0),
                work.place,
                11.0,
                SECONDARY,
            );
        }
        if app.marked[index] {
            painter.circle_filled(pos2(right - 6.0, row.center().y), 3.0, ACCENT);
        }
        if response.clicked() {
            app.select(index);
        }
        if response.double_clicked() {
            app.open_preview();
        }
    }

    rule(&painter, left, right, shelf);
    let controls_y = shelf + 8.0;
    text(
        &painter,
        pos2(left, controls_y + 14.0),
        "↑ ↓  Выбор",
        12.0,
        SECONDARY,
    );
    let preview = Rect::from_min_size(pos2(right - 267.0, controls_y), vec2(128.0, 28.0));
    if selected.is_some() && text_button(ui, &painter, preview, "Просмотр", "Enter") {
        app.open_preview();
    }
    let actions = Rect::from_min_size(pos2(right - 130.0, controls_y), vec2(130.0, 28.0));
    if text_button(ui, &painter, actions, "Действия", "Ctrl+K") {
        if app.overlay == Some(crate::Overlay::Actions) {
            app.close_overlay();
        } else {
            app.open_actions();
        }
    }
}

fn text_button(ui: &mut egui::Ui, painter: &Painter, rect: Rect, label: &str, key: &str) -> bool {
    let response = ui
        .interact(
            rect,
            ui.make_persistent_id(("editorial_button", label)),
            Sense::click(),
        )
        .on_hover_cursor(CursorIcon::PointingHand);
    if response.hovered() || response.has_focus() {
        painter.hline(
            rect.left() + 6.0..=rect.right() - 6.0,
            rect.bottom() - 4.0,
            Stroke::new(1.0, ACCENT),
        );
    }
    text(
        painter,
        pos2(rect.left() + 6.0, rect.center().y),
        label,
        12.0,
        INK,
    );
    painter.text(
        pos2(rect.right() - 6.0, rect.center().y),
        Align2::RIGHT_CENTER,
        format!("/ {key}"),
        mono(10.0),
        SECONDARY,
    );
    response.clicked()
}

fn rule(painter: &Painter, left: f32, right: f32, y: f32) {
    painter.hline(left..=right, y, Stroke::new(1.0, RULE));
}

fn text(painter: &Painter, pos: Pos2, value: &str, size: f32, color: Color32) {
    painter.text(
        pos,
        Align2::LEFT_CENTER,
        value,
        FontId::proportional(size),
        color,
    );
}

fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SERIF.into()))
}

fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

fn cover(painter: &Painter, bounds: Rect, texture: &egui::TextureHandle) {
    let [w, h] = texture.size();
    let ratio = bounds.aspect_ratio() / (w as f32 / h as f32);
    let view = vec2(ratio.min(1.0), ratio.recip().min(1.0));
    let uv = Rect::from_center_size(pos2(0.5, 0.5), view);
    painter.image(texture.id(), bounds, uv, Color32::WHITE);
}
