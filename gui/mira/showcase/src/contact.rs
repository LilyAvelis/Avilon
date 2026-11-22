use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind,
    UiBuilder, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{App, WORKS};

const ROOM: Color32 = Color32::from_rgb(34, 30, 26);
const TRAY: Color32 = Color32::from_rgb(46, 40, 34);
const PAPER: Color32 = Color32::from_rgb(247, 233, 209);
const MUTED: Color32 = Color32::from_rgb(187, 168, 142);
const AMBER: Color32 = Color32::from_rgb(242, 183, 108);

pub(super) fn show(ui: &mut egui::Ui, app: &mut App) {
    let bounds = ui.available_rect_before_wrap();
    let p = ui.painter_at(bounds);
    let left = bounds.left() + 28.0;
    let right = bounds.right() - 28.0;
    let top = bounds.top();
    let bottom = bounds.bottom();
    p.rect_filled(bounds, 0, ROOM);
    label(
        &p,
        pos2(left, top + 31.0),
        Align2::LEFT_CENTER,
        "Контакт",
        25.0,
        PAPER,
    );
    label(
        &p,
        pos2(left + 121.0, top + 33.0),
        Align2::LEFT_CENTER,
        "После дождя",
        13.0,
        MUTED,
    );
    let search = Rect::from_min_size(pos2(right - 248.0, top + 19.0), vec2(248.0, 30.0));
    ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
        ui.visuals_mut().text_edit_bg_color = Some(TRAY);
        app.search(ui, search.width() - 12.0);
    });

    let strip_height = if app.compact { 66.0 } else { 92.0 };
    let strip_y = bottom - 58.0 - strip_height;
    let stage = Rect::from_min_max(pos2(left, top + 62.0), pos2(right, strip_y - 14.0));
    let matches = app.matches();
    if let Some(index) = app.selected {
        let other = matches
            .iter()
            .position(|candidate| *candidate == index)
            .and_then(|position| {
                matches
                    .get(position + 1)
                    .or_else(|| position.checked_sub(1).and_then(|i| matches.get(i)))
            })
            .copied();
        let center = stage.center();
        if let Some(other) = other.filter(|_| app.comparing) {
            let half = (stage.width() - 22.0) * 0.5;
            let size = stage.height().min(half);
            plate(
                ui,
                &p,
                app,
                index,
                pos2(center.x - (size + 22.0) * 0.5, center.y),
                size,
            );
            plate(
                ui,
                &p,
                app,
                other,
                pos2(center.x + (size + 22.0) * 0.5, center.y),
                size,
            );
        } else {
            plate(
                ui,
                &p,
                app,
                index,
                center,
                stage.height().min(stage.width()),
            );
        }
    } else {
        label(
            &p,
            stage.center(),
            Align2::CENTER_CENTER,
            "Снимки не найдены",
            20.0,
            MUTED,
        );
    }

    p.rect_filled(
        Rect::from_min_max(
            pos2(bounds.left(), strip_y),
            pos2(bounds.right(), bottom - 54.0),
        ),
        0,
        TRAY,
    );
    let tile = strip_height - 25.0;
    let stride = tile + 13.0;
    let origin = bounds.center().x - matches.len().saturating_sub(1) as f32 * stride * 0.5;
    for (slot, index) in matches.iter().copied().enumerate() {
        let center = pos2(origin + slot as f32 * stride, strip_y + tile * 0.5 + 6.0);
        let rect = Rect::from_center_size(center, vec2(tile, tile));
        let response = ui
            .interact(
                rect,
                ui.make_persistent_id(("negative", index)),
                Sense::click(),
            )
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(WORKS[index].title);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, WORKS[index].title));
        if app.selected == Some(index) || response.has_focus() {
            p.rect_stroke(
                rect.expand(3.0),
                2,
                Stroke::new(2.0, AMBER),
                StrokeKind::Outside,
            );
        }
        photograph(&p, app, index, rect);
        let caption = if app.marked[index] {
            format!("{:02}  •", index + 1)
        } else {
            format!("{:02}", index + 1)
        };
        label(
            &p,
            pos2(center.x, rect.bottom() + 12.0),
            Align2::CENTER_CENTER,
            &caption,
            11.0,
            MUTED,
        );
        if response.clicked() {
            app.select(index);
            if response.double_clicked() {
                app.open_preview();
            }
        }
    }

    let footer = bottom - 28.0;
    label(
        &p,
        pos2(left, footer),
        Align2::LEFT_CENTER,
        "↑ ↓  По кадрам",
        12.0,
        MUTED,
    );
    if dial(
        ui,
        &p,
        pos2(right - 348.0, footer),
        "↔",
        "Сравнить",
        app.comparing,
    ) {
        app.comparing = !app.comparing;
    }
    if dial(ui, &p, pos2(right - 235.0, footer), "↗", "Enter", false) {
        app.open_preview();
    }
    if dial(ui, &p, pos2(right - 142.0, footer), "···", "Ctrl+K", false) {
        app.open_actions();
    }
    if dial(ui, &p, pos2(right - 51.0, footer), "≡", "Ctrl+,", false) {
        app.open_settings();
    }
}

fn plate(ui: &mut egui::Ui, p: &Painter, app: &mut App, index: usize, center: Pos2, size: f32) {
    let photo = Rect::from_center_size(center - vec2(0.0, 13.0), vec2(size - 31.0, size - 31.0));
    p.rect_filled(photo.expand(4.0), 0, PAPER);
    photograph(p, app, index, photo);
    let caption = format!("{}  ·  {}", WORKS[index].title, WORKS[index].place);
    label(
        p,
        pos2(center.x, photo.bottom() + 20.0),
        Align2::CENTER_CENTER,
        &caption,
        13.0,
        PAPER,
    );
    let response = ui
        .interact(
            photo,
            ui.make_persistent_id(("plate", index)),
            Sense::click(),
        )
        .on_hover_cursor(CursorIcon::PointingHand);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, WORKS[index].title));
    if response.clicked() {
        app.select(index);
        app.open_preview();
    }
}

fn photograph(p: &Painter, app: &App, index: usize, rect: Rect) {
    p.image(
        app.textures[index].id(),
        rect,
        Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn label(p: &Painter, point: Pos2, align: Align2, value: &str, size: f32, color: Color32) {
    p.text(point, align, value, FontId::proportional(size), color);
}

fn dial(
    ui: &mut egui::Ui,
    p: &Painter,
    point: Pos2,
    glyph: &str,
    hint: &str,
    active: bool,
) -> bool {
    let name = match glyph {
        "↔" => "Сравнить соседние кадры",
        "↗" => "Открыть просмотр",
        "···" => "Действия",
        _ => "Настройки",
    };
    let touch = Rect::from_min_size(point - vec2(18.0, 17.0), vec2(94.0, 34.0));
    let response = ui
        .interact(
            touch,
            ui.make_persistent_id(("dial", glyph)),
            Sense::click(),
        )
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(name);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name));
    let lit = active || response.hovered() || response.has_focus();
    p.circle_filled(point, 16.0, if lit { AMBER } else { PAPER });
    label(p, point, Align2::CENTER_CENTER, glyph, 18.0, ROOM);
    label(
        p,
        point + vec2(23.0, 0.0),
        Align2::LEFT_CENTER,
        hint,
        12.0,
        MUTED,
    );
    response.clicked()
}
