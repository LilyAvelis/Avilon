use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind,
    UiBuilder, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{App, WORKS};

pub(super) const SECTIONS: [Option<&str>; 5] = [
    None,
    Some("Пейзаж"),
    Some("Архитектура"),
    Some("Город"),
    Some("Ботаника"),
];

const BLUE: Color32 = Color32::from_rgb(14, 28, 86);
const DEEP: Color32 = Color32::from_rgb(8, 18, 58);
const GRID: Color32 = Color32::from_rgb(50, 69, 131);
const WHITE: Color32 = Color32::from_rgb(231, 236, 255);
const LIGHT: Color32 = Color32::from_rgb(160, 185, 255);
const SIGNAL: Color32 = Color32::from_rgb(226, 250, 128);

pub(super) fn show(ui: &mut egui::Ui, app: &mut App) {
    let window = ui.available_rect_before_wrap();
    let p = ui.painter_at(window);
    let left = window.left() + 24.0;
    let right = window.right() - 24.0;
    let top = window.top();
    let bottom = window.bottom();
    p.rect_filled(window, 0, BLUE);
    text(
        &p,
        pos2(left, top + 33.0),
        Align2::LEFT_CENTER,
        "АЗИМУТ",
        24.0,
        SIGNAL,
    );
    text(
        &p,
        pos2(left + 128.0, top + 32.0),
        Align2::LEFT_CENTER,
        "После дождя",
        13.0,
        LIGHT,
    );
    let search = Rect::from_min_size(pos2(right - 249.0, top + 21.0), vec2(249.0, 27.0));
    ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
        ui.visuals_mut().text_edit_bg_color = Some(DEEP);
        app.search(ui, search.width() - 10.0);
    });

    let tabs = Rect::from_min_max(pos2(left, top + 67.0), pos2(right, top + 111.0));
    let tab_width = tabs.width() / SECTIONS.len() as f32;
    for (i, category) in SECTIONS.iter().enumerate() {
        let name = category.unwrap_or("Все работы");
        let tab = Rect::from_min_size(
            tabs.min + vec2(tab_width * i as f32, 0.0),
            vec2(tab_width, tabs.height()),
        );
        let response = hit(ui, tab, ("azimuth-section", i), name);
        let active = app.azimuth_section == i;
        if active || response.hovered() || response.has_focus() {
            p.rect_filled(
                tab.shrink2(vec2(1.0, 0.0)),
                0,
                if active { SIGNAL } else { GRID },
            );
        }
        p.vline(tab.right(), tab.y_range(), Stroke::new(1.0, GRID));
        text(
            &p,
            tab.center(),
            Align2::CENTER_CENTER,
            name,
            12.0,
            if active { DEEP } else { WHITE },
        );
        if response.clicked() {
            app.azimuth_section = i;
            app.sync_selection();
        }
    }
    p.hline(tabs.x_range(), tabs.bottom(), Stroke::new(1.0, LIGHT));

    let top = tabs.bottom() + 24.0;
    let body_bottom = bottom - 77.0;
    let list_width = ((right - left) * 0.29).clamp(205.0, 295.0);
    let list = Rect::from_min_max(pos2(left, top), pos2(left + list_width, body_bottom));
    let field = Rect::from_min_max(pos2(list.right() + 24.0, top), pos2(right, body_bottom));
    p.rect_filled(field, 0, DEEP);
    for i in 1..12 {
        let fraction = i as f32 / 12.0;
        p.vline(
            field.left() + fraction * field.width(),
            field.y_range(),
            Stroke::new(1.0, GRID),
        );
        p.hline(
            field.x_range(),
            field.top() + fraction * field.height(),
            Stroke::new(1.0, GRID),
        );
    }
    if let Some(index) = app.selected {
        let side = field.height().min(field.width()) - 46.0;
        let point = field.center();
        let snapshot = Rect::from_center_size(point, vec2(side, side));
        p.image(
            app.textures[index].id(),
            snapshot,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let corner = point + vec2(x, y) * (side * 0.5 + 5.0);
            p.line_segment(
                [corner, corner - vec2(x * 12.0, 0.0)],
                Stroke::new(2.0, SIGNAL),
            );
            p.line_segment(
                [corner, corner - vec2(0.0, y * 12.0)],
                Stroke::new(2.0, SIGNAL),
            );
        }
        let work = &WORKS[index];
        text(
            &p,
            pos2(field.left() + 8.0, field.top() + 11.0),
            Align2::LEFT_CENTER,
            work.place,
            10.0,
            LIGHT,
        );
        text(
            &p,
            pos2(field.right() - 8.0, field.bottom() - 11.0),
            Align2::RIGHT_CENTER,
            &work.year.to_string(),
            10.0,
            LIGHT,
        );
        if hit(ui, snapshot, ("azimuth-preview", index), work.title).clicked() {
            app.open_preview();
        }
    } else {
        text(
            &p,
            field.center(),
            Align2::CENTER_CENTER,
            "Снимки не найдены",
            15.0,
            LIGHT,
        );
    }

    p.vline(list.left() + 10.0, list.y_range(), Stroke::new(1.0, GRID));
    let row_h = if app.compact {
        34.0
    } else {
        (list.height() / WORKS.len() as f32).min(58.0)
    };
    for (slot, index) in app.matches().into_iter().enumerate() {
        let row = Rect::from_min_size(
            list.min + vec2(0.0, slot as f32 * row_h),
            vec2(list.width(), row_h),
        );
        let work = &WORKS[index];
        let response = hit(ui, row, ("azimuth-row", index), work.title);
        let active = app.selected == Some(index);
        if active || response.hovered() || response.has_focus() {
            p.rect_filled(
                row,
                0,
                if active {
                    Color32::from_rgb(33, 53, 117)
                } else {
                    GRID
                },
            );
        }
        let origin = pos2(row.left() + 10.0, row.center().y);
        p.circle_filled(
            origin,
            if active { 4.0 } else { 2.0 },
            if active { SIGNAL } else { LIGHT },
        );
        text(
            &p,
            origin + vec2(13.0, 0.0),
            Align2::LEFT_CENTER,
            &format!("{:02}", index + 1),
            11.0,
            LIGHT,
        );
        text(
            &p,
            origin + vec2(37.0, 0.0),
            Align2::LEFT_CENTER,
            work.title,
            12.0,
            if active { SIGNAL } else { WHITE },
        );
        if app.marked[index] {
            p.circle_filled(pos2(row.right() - 7.0, row.center().y), 3.0, SIGNAL);
        }
        if response.clicked() {
            app.select(index);
            if response.double_clicked() {
                app.open_preview();
            }
        }
    }
    let foot = bottom - 56.0;
    text(
        &p,
        pos2(left, foot + 19.0),
        Align2::LEFT_CENTER,
        "← →  Раздел   /   ↑ ↓  Кадр",
        11.0,
        LIGHT,
    );
    if stem(ui, &p, pos2(right - 309.0, foot), "Просмотр", "Enter") {
        app.open_preview();
    }
    if stem(ui, &p, pos2(right - 206.0, foot), "Действия", "Ctrl+K") {
        app.open_actions();
    }
    if stem(ui, &p, pos2(right - 103.0, foot), "Настройки", "Ctrl+,") {
        app.open_settings();
    }
}

fn text(p: &Painter, point: Pos2, align: Align2, value: &str, size: f32, color: Color32) {
    p.text(point, align, value, FontId::proportional(size), color);
}

fn hit(ui: &mut egui::Ui, rect: Rect, id: impl egui::AsIdSalt, name: &str) -> egui::Response {
    let r = ui
        .interact(rect, ui.make_persistent_id(id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    r.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name));
    r
}

fn stem(ui: &mut egui::Ui, p: &Painter, at: Pos2, name: &str, shortcut: &str) -> bool {
    let tile = Rect::from_min_size(at, vec2(99.0, 41.0));
    let response = hit(ui, tile, ("azimuth-control", name), name);
    let active = response.hovered() || response.has_focus();
    p.rect_filled(tile, 0, if active { SIGNAL } else { GRID });
    p.rect_stroke(
        tile,
        0,
        Stroke::new(1.0, if active { SIGNAL } else { LIGHT }),
        StrokeKind::Inside,
    );
    let foreground = if active { DEEP } else { WHITE };
    text(
        p,
        pos2(tile.center().x, at.y + 11.0),
        Align2::CENTER_CENTER,
        name,
        12.0,
        foreground,
    );
    p.text(
        pos2(tile.center().x, at.y + 29.0),
        Align2::CENTER_CENTER,
        shortcut,
        FontId::monospace(10.0),
        if active { DEEP } else { SIGNAL },
    );
    response.clicked()
}
