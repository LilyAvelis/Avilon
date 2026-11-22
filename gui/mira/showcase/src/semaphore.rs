use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Painter, Rect, Sense, Shape, Stroke, StrokeKind,
    UiBuilder, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{Action, App, Overlay, WORKS, search_id};

const PAPER: Color32 = Color32::from_rgb(245, 237, 217);
const INK: Color32 = Color32::from_rgb(30, 39, 38);
const DARK: Color32 = Color32::from_rgb(21, 31, 30);
const SLATE: Color32 = Color32::from_rgb(35, 51, 47);
const WHITE: Color32 = Color32::from_rgb(246, 243, 218);
const QUIET: Color32 = Color32::from_rgb(93, 99, 85);
const PALE: Color32 = Color32::from_rgb(181, 190, 166);
const RULE: Color32 = Color32::from_rgb(193, 182, 151);
const SIGNAL: Color32 = Color32::from_rgb(193, 53, 32);
const BRIGHT: Color32 = Color32::from_rgb(255, 133, 93);

pub(super) fn show(ui: &mut egui::Ui, app: &mut App) {
    let window = ui.available_rect_before_wrap();
    let painter = ui.painter_at(window);
    let content = window.shrink2(vec2(24.0, 0.0));
    painter.rect_filled(window, 0, PAPER);
    header(ui, &painter, app, content);

    let stage = Rect::from_min_max(
        content.left_top() + vec2(0.0, 80.0),
        content.right_bottom() - vec2(0.0, 172.0),
    );
    let route = Rect::from_min_max(
        pos2(content.left(), stage.bottom() + 18.0),
        content.right_bottom() - vec2(0.0, 23.0),
    );
    stage_view(ui, &painter, app, stage);
    route_view(ui, &painter, app, route);
}

fn header(ui: &mut egui::Ui, p: &Painter, app: &mut App, content: Rect) {
    let top = content.top();
    let x = content.left();
    let right = content.right();
    let flag = pos2(x + 3.0, top + 23.0);
    p.line_segment([flag, flag + vec2(0.0, 29.0)], Stroke::new(2.0, INK));
    p.add(Shape::convex_polygon(
        vec![
            flag,
            flag + vec2(24.0, 0.0),
            flag + vec2(17.0, 12.0),
            flag + vec2(0.0, 12.0),
        ],
        SIGNAL,
        Stroke::NONE,
    ));
    p.text(
        pos2(x + 36.0, top + 36.0),
        Align2::LEFT_CENTER,
        "СЕМАФОР",
        FontId::proportional(27.0),
        INK,
    );
    p.text(
        pos2(x + 194.0, top + 38.0),
        Align2::LEFT_CENTER,
        "После дождя",
        FontId::proportional(14.0),
        QUIET,
    );
    ruler(p, content.left()..=content.right(), top + 64.0);

    let search = Rect::from_min_size(pos2(right - 232.0, top + 22.0), vec2(232.0, 31.0));
    ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
        let visuals = ui.visuals_mut();
        visuals.text_edit_bg_color = Some(PAPER);
        visuals.override_text_color = Some(INK);
        visuals.weak_text_color = Some(QUIET);
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        visuals.widgets.active.bg_stroke = Stroke::NONE;
        let r = app.search(ui, search.width() - 9.0);
        p.hline(
            search.x_range(),
            search.bottom(),
            Stroke::new(
                if r.has_focus() { 2.0 } else { 1.0 },
                if r.has_focus() { SIGNAL } else { RULE },
            ),
        );
    });
}

fn stage_view(ui: &mut egui::Ui, p: &Painter, app: &mut App, stage: Rect) {
    p.rect_filled(stage, 0, DARK);
    let panel_width = (stage.width() * 0.23).clamp(172.0, 244.0);
    let photo = Rect::from_min_max(
        stage.min + vec2(12.0, 12.0),
        pos2(stage.right() - panel_width, stage.bottom() - 12.0),
    );
    let sidebar = Rect::from_min_max(
        pos2(photo.right() + 12.0, stage.top() + 12.0),
        stage.right_bottom() - vec2(12.0, 12.0),
    );
    p.rect_filled(sidebar, 0, SLATE);
    let left = sidebar.left() + 14.0;
    let right = sidebar.right() - 14.0;
    let label_at = pos2(left, sidebar.top() + 17.0);
    if let Some(index) = app.selected {
        let work = &WORKS[index];
        cover(p, photo, &app.textures[index]);
        if hit(ui, photo, "semaphore-preview", work.title)
            .on_hover_text("Открыть кадр")
            .clicked()
        {
            app.open_preview();
        }
        p.text(
            label_at,
            Align2::LEFT_TOP,
            format!("{:02} / {}", index + 1, work.year),
            FontId::monospace(12.0),
            BRIGHT,
        );
        let title = p.layout(
            work.title.to_owned(),
            FontId::proportional(if panel_width < 190.0 { 21.0 } else { 29.0 }),
            WHITE,
            right - left,
        );
        let top = sidebar.top() + 47.0;
        let details_y = top + title.size().y + 14.0;
        p.galley(pos2(left, top), title, WHITE);
        p.hline(
            left..=right,
            details_y - 6.0,
            Stroke::new(1.0, Color32::from_rgb(94, 111, 98)),
        );
        let details = p.layout(
            format!("{}  /  {}", work.place, work.category),
            FontId::proportional(12.0),
            PALE,
            right - left,
        );
        p.galley(pos2(left, details_y), details, PALE);
        if app.marked[index] {
            p.circle_filled(pos2(right - 3.0, label_at.y + 6.0), 3.0, BRIGHT);
        }
    } else {
        p.rect_filled(photo, 0, Color32::from_rgb(29, 40, 37));
        p.text(
            photo.center(),
            Align2::CENTER_CENTER,
            if app.semaphore_marked && app.query.is_empty() {
                "Пока нет отмеченных кадров"
            } else {
                "Кадры не найдены"
            },
            FontId::proportional(16.0),
            PALE,
        );
        p.text(
            label_at,
            Align2::LEFT_TOP,
            "После дождя",
            FontId::proportional(15.0),
            WHITE,
        );
    }
    let actions = app.overlay == Some(Overlay::Actions);
    if signal_button(
        ui,
        p,
        Rect::from_min_size(
            pos2(left, sidebar.bottom() - 78.0),
            vec2(right - left, 33.0),
        ),
        "switch-actions",
        if actions {
            "К маршруту"
        } else {
            "Действия"
        },
        if actions { "Esc" } else { "Ctrl+K" },
    ) {
        if actions {
            app.close_overlay();
        } else {
            app.open_actions();
        }
    }
    if signal_button(
        ui,
        p,
        Rect::from_min_size(
            pos2(left, sidebar.bottom() - 39.0),
            vec2(right - left, 28.0),
        ),
        "switch-settings",
        "Настройки",
        "Ctrl+,",
    ) {
        app.open_settings();
    }
}

fn route_view(ui: &mut egui::Ui, p: &Painter, app: &mut App, rect: Rect) {
    if app.overlay == Some(Overlay::Actions) {
        route_actions(ui, p, app, rect);
        return;
    }
    let x = rect.left();
    let right = rect.right();
    let top = rect.top();
    for (slot, (label, marked_only)) in [("Все кадры", false), ("Отмеченные", true)]
        .into_iter()
        .enumerate()
    {
        let r = Rect::from_min_size(pos2(x + (slot as f32) * 111.0, top), vec2(107.0, 24.0));
        let response = hit(ui, r, ("semaphore-route", slot), label);
        let active = app.semaphore_marked == marked_only;
        if active {
            p.rect_filled(r, 0, INK);
        } else if response.hovered() || response.has_focus() {
            p.rect_filled(r, 0, RULE);
        }
        p.text(
            r.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(12.0),
            if active { WHITE } else { INK },
        );
        if response.clicked() && !active {
            app.semaphore_marked = marked_only;
            app.sync_selection();
        }
    }

    let text_focus = !app.query.is_empty() && ui.memory(|memory| memory.has_focus(search_id()));
    p.text(
        pos2(right, top + 13.0),
        Align2::RIGHT_CENTER,
        if text_focus {
            "↑ ↓  Выбрать кадр"
        } else {
            "← →  По маршруту"
        },
        FontId::monospace(11.0),
        QUIET,
    );
    let items = app.matches();
    let track_y = top + 61.0;
    let route_x = (right - x) / items.len().max(1) as f32;
    ruler(p, x..=right, track_y);
    if items.is_empty() {
        p.text(
            pos2(rect.center().x, track_y + 25.0),
            Align2::CENTER_CENTER,
            "На этом маршруте пока нет кадров",
            FontId::proportional(13.0),
            QUIET,
        );
    }
    for (slot, index) in items.into_iter().enumerate() {
        let point = pos2(x + route_x * (slot as f32 + 0.5), track_y);
        let target = Rect::from_center_size(point + vec2(0.0, 14.0), vec2(route_x - 4.0, 63.0));
        let response = hit(ui, target, ("semaphore-station", index), WORKS[index].title)
            .on_hover_text(WORKS[index].title);
        let active = app.selected == Some(index);
        let hot = active || response.hovered() || response.has_focus();
        p.circle_filled(
            point,
            if active { 8.0 } else { 5.0 },
            if hot { SIGNAL } else { INK },
        );
        p.circle_filled(point, if active { 3.3 } else { 2.0 }, PAPER);
        p.text(
            point + vec2(0.0, 20.0),
            Align2::CENTER_CENTER,
            WORKS[index].place,
            FontId::proportional(13.0),
            if active { SIGNAL } else { INK },
        );
        if !app.compact {
            p.text(
                point + vec2(0.0, 40.0),
                Align2::CENTER_CENTER,
                format!("{:02}", index + 1),
                FontId::monospace(9.0),
                QUIET,
            );
        }
        if response.clicked() {
            app.select(index);
            if response.double_clicked() {
                app.open_preview();
            }
        }
    }
}

fn route_actions(ui: &mut egui::Ui, p: &Painter, app: &mut App, rect: Rect) {
    let actions = app.actions();
    let origin = rect.left_top() + vec2(0.0, 70.0);
    let width = rect.width() / actions.len() as f32;
    let y = rect.top();
    p.text(
        pos2(rect.left(), y + 13.0),
        Align2::LEFT_CENTER,
        "Действия",
        FontId::proportional(16.0),
        INK,
    );
    p.text(
        pos2(rect.right(), y + 13.0),
        Align2::RIGHT_CENTER,
        "← →  Выбор   /   Enter  Действие",
        FontId::monospace(11.0),
        QUIET,
    );
    ruler(p, rect.left()..=rect.right(), origin.y);
    for (index, action) in actions.into_iter().enumerate() {
        let point = origin + vec2((index as f32 + 0.5) * width, 0.0);
        let label = match action {
            Action::Preview => "Просмотр",
            Action::Mark if app.selected.is_some_and(|i| app.marked[i]) => "Снять отметку",
            Action::Mark => "Отметить",
            Action::Undo => "Отменить отметку",
            Action::Settings => "Настройки",
        };
        let sign = Rect::from_center_size(
            point + vec2(0.0, 21.0),
            vec2((width - 16.0).min(200.0), 37.0),
        );
        let response = hit(ui, sign, ("signal-action", index), label);
        let active = index == app.action_index;
        let hot = active || response.hovered() || response.has_focus();
        let fill = if hot { SIGNAL } else { INK };
        let poly = vec![
            sign.left_top(),
            sign.right_top() - vec2(11.0, 0.0),
            sign.right_center(),
            sign.right_bottom() - vec2(11.0, 0.0),
            sign.left_bottom(),
        ];
        p.line_segment(
            [point - vec2(0.0, 11.0), point + vec2(0.0, 6.0)],
            Stroke::new(2.0, fill),
        );
        p.circle_filled(point - vec2(0.0, 11.0), 3.0, fill);
        p.add(Shape::convex_polygon(poly, fill, Stroke::NONE));
        p.text(
            sign.center() - vec2(3.0, 0.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(13.0),
            WHITE,
        );
        if response.clicked() {
            app.execute(ui.ctx(), action);
        }
    }
}

fn signal_button(
    ui: &mut egui::Ui,
    p: &Painter,
    rect: Rect,
    id: &str,
    label: &str,
    key: &str,
) -> bool {
    let response = hit(ui, rect, id, label);
    let active = response.has_focus() || response.hovered();
    p.hline(
        rect.x_range(),
        rect.top(),
        Stroke::new(
            1.0,
            if active {
                BRIGHT
            } else {
                Color32::from_rgb(97, 114, 98)
            },
        ),
    );
    p.text(
        rect.left_center(),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        if active { BRIGHT } else { WHITE },
    );
    p.text(
        rect.right_center(),
        Align2::RIGHT_CENTER,
        key,
        FontId::monospace(10.0),
        PALE,
    );
    response.clicked()
}

fn hit(ui: &mut egui::Ui, rect: Rect, id: impl egui::AsIdSalt, label: &str) -> egui::Response {
    let response = ui
        .interact(rect, ui.make_persistent_id(id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    response
}

fn ruler(p: &Painter, span: std::ops::RangeInclusive<f32>, y: f32) {
    p.hline(span.clone(), y, Stroke::new(1.0, RULE));
    p.hline(span.clone(), y + 5.0, Stroke::new(1.0, RULE));
    let mut x = *span.start() + 10.0;
    while x < *span.end() {
        p.vline(x, y - 2.0..=y + 7.0, Stroke::new(1.0, RULE));
        x += 19.0;
    }
}

fn cover(p: &Painter, rect: Rect, texture: &egui::TextureHandle) {
    let aspect = rect.aspect_ratio();
    let uv = Rect::from_center_size(
        pos2(0.5, 0.5),
        vec2(aspect.min(1.0), aspect.recip().min(1.0)),
    );
    p.image(texture.id(), rect, uv, Color32::WHITE);
    p.rect_stroke(
        rect,
        0,
        Stroke::new(1.0, Color32::from_rgb(102, 112, 98)),
        StrokeKind::Inside,
    );
}
