use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Painter, Pos2, Rect, Sense, Stroke, StrokeKind,
    UiBuilder, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{App, WORKS};

const BACKGROUND: Color32 = Color32::from_rgb(14, 17, 24);
const SURFACE: Color32 = Color32::from_rgb(23, 28, 38);
const LINE: Color32 = Color32::from_rgb(45, 54, 69);
const INK: Color32 = Color32::from_rgb(237, 241, 248);
const QUIET: Color32 = Color32::from_rgb(156, 170, 191);
const ACCENT: Color32 = Color32::from_rgb(151, 231, 206);
const SELECTED: Color32 = Color32::from_rgb(34, 51, 60);
const SELECTED_LINE: Color32 = Color32::from_rgb(64, 100, 106);
const HOVER: Color32 = Color32::from_rgb(28, 36, 49);

pub(crate) fn show(ui: &mut egui::Ui, app: &mut App) {
    let (window, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let content = window.shrink2(vec2(22.0, 0.0));
    let painter = ui.painter().clone();
    painter.rect_filled(window, 0, BACKGROUND);

    let mark = pos2(content.left() + 8.0, content.top() + 29.0);
    let diamond = vec![
        mark - vec2(0.0, 7.0),
        mark + vec2(7.0, 0.0),
        mark + vec2(0.0, 7.0),
        mark - vec2(7.0, 0.0),
    ];
    painter.add(egui::Shape::convex_polygon(diamond, ACCENT, Stroke::NONE));
    painter.circle_filled(mark, 2.1, BACKGROUND);
    painter.text(
        mark + vec2(17.0, 0.0),
        Align2::LEFT_CENTER,
        "Люмен",
        FontId::proportional(19.0),
        INK,
    );
    painter.text(
        pos2(content.right(), mark.y),
        Align2::RIGHT_CENTER,
        "После дождя",
        FontId::proportional(13.0),
        QUIET,
    );

    let search = Rect::from_min_max(
        pos2(content.left(), window.top() + 54.0),
        pos2(content.right(), window.top() + 106.0),
    );
    search_box(ui, app, &painter, search);

    let footer_top = window.bottom() - 51.0;
    let body = Rect::from_min_max(
        pos2(content.left(), search.bottom() + 17.0),
        pos2(content.right(), footer_top - 15.0),
    );
    let list_width = body.width() * 0.43;
    let list = Rect::from_min_size(body.min, vec2(list_width, body.height()));
    let inspector = Rect::from_min_max(pos2(list.right() + 14.0, body.top()), body.max);
    draw_list(ui, app, list);
    draw_inspector(ui, app, inspector);

    painter.rect_filled(
        Rect::from_min_max(pos2(window.left(), footer_top), window.max),
        0,
        SURFACE,
    );
    painter.hline(window.x_range(), footer_top, Stroke::new(1.0, LINE));
    let footer_y = footer_top + 25.0;
    let available = app.selected.is_some();
    painter.text(
        pos2(content.left(), footer_y),
        Align2::LEFT_CENTER,
        "↑ ↓  Выбор",
        FontId::proportional(12.0),
        QUIET,
    );
    let right = content.right();
    if footer_button(
        ui,
        "settings",
        pos2(right - 118.0, footer_y),
        118.0,
        "Настройки",
        "Ctrl ,",
        true,
    ) {
        app.open_settings();
    }
    if footer_button(
        ui,
        "actions",
        pos2(right - 238.0, footer_y),
        113.0,
        "Действия",
        "Ctrl K",
        true,
    ) {
        app.open_actions();
    }
    if footer_button(
        ui,
        "preview",
        pos2(right - 373.0, footer_y),
        128.0,
        "Просмотр",
        "Enter",
        available,
    ) {
        app.open_preview();
    }
}

fn search_box(ui: &mut egui::Ui, app: &mut App, painter: &Painter, rect: Rect) {
    painter.rect_filled(rect, 11, SURFACE);
    let lens = pos2(rect.left() + 24.0, rect.center().y - 2.0);
    painter.circle_stroke(lens, 5.3, Stroke::new(1.5, ACCENT));
    painter.line_segment(
        [lens + vec2(4.0, 4.0), lens + vec2(8.5, 8.5)],
        Stroke::new(1.5, ACCENT),
    );

    let input = Rect::from_min_max(
        pos2(rect.left() + 40.0, rect.top() + 12.0),
        pos2(rect.right() - 12.0, rect.bottom() - 8.0),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(input)
            .layout(Layout::top_down(Align::Min)),
    );
    let style = child.style_mut();
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(17.0));
    style.visuals.text_edit_bg_color = Some(SURFACE);
    style.visuals.override_text_color = Some(INK);
    style.visuals.weak_text_color = Some(QUIET);
    style.visuals.selection.stroke = Stroke::NONE;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    style.visuals.widgets.noninteractive.fg_stroke.color = QUIET;
    let response = app.search(&mut child, input.width());
    let color = if response.has_focus() {
        SELECTED_LINE
    } else {
        LINE
    };
    painter.rect_stroke(rect, 11, Stroke::new(1.0, color), StrokeKind::Inside);
}

fn draw_list(ui: &mut egui::Ui, app: &mut App, rect: Rect) {
    let painter = ui.painter().with_clip_rect(rect);
    let matches = app.matches();
    if matches.is_empty() {
        painter.text(
            rect.left_top() + vec2(5.0, 18.0),
            Align2::LEFT_CENTER,
            "Ничего не найдено",
            FontId::proportional(15.0),
            INK,
        );
        painter.text(
            rect.left_top() + vec2(5.0, 45.0),
            Align2::LEFT_CENTER,
            "Попробуй другой запрос",
            FontId::proportional(12.0),
            QUIET,
        );
        return;
    }

    let row_gap = 4.0;
    let max_height: f32 = if app.compact { 39.0 } else { 52.0 };
    let row_height = ((rect.height() - row_gap * 5.0) / 6.0).min(max_height);
    for (row, index) in matches.into_iter().enumerate() {
        let work = &WORKS[index];
        let head = rect.min + vec2(0.0, row as f32 * (row_height + row_gap));
        let row_rect = Rect::from_min_size(head, vec2(rect.width(), row_height));
        let response = ui
            .interact(
                row_rect,
                ui.make_persistent_id(("lumen-work", index)),
                Sense::click(),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() {
            app.select(index);
            if response.double_clicked() {
                app.open_preview();
            }
        }

        let selected = app.selected == Some(index);
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::SelectableLabel, true, selected, work.title)
        });
        if selected {
            painter.rect_filled(row_rect, 8, SELECTED);
            painter.rect_stroke(
                row_rect,
                8,
                Stroke::new(1.0, SELECTED_LINE),
                StrokeKind::Inside,
            );
            let rail = Rect::from_center_size(
                pos2(row_rect.left() + 2.0, row_rect.center().y),
                vec2(2.5, row_height - 16.0),
            );
            painter.rect_filled(rail, 2, ACCENT);
        } else if response.hovered() || response.has_focus() {
            painter.rect_filled(row_rect, 8, HOVER);
        }
        if response.has_focus() {
            painter.rect_stroke(row_rect, 8, Stroke::new(1.0, ACCENT), StrokeKind::Inside);
        }

        let thumb_height = (row_height - 13.0).min(32.0);
        let thumb = Rect::from_center_size(
            pos2(row_rect.left() + 33.0, row_rect.center().y),
            vec2(46.0, thumb_height),
        );
        paint_photo(&painter, app, index, thumb, 4);
        let label_x = thumb.right() + 11.0;
        let label = painter.with_clip_rect(Rect::from_min_max(
            pos2(label_x, row_rect.top()),
            pos2(row_rect.right() - 30.0, row_rect.bottom()),
        ));
        label.text(
            pos2(label_x, row_rect.center().y - 8.0),
            Align2::LEFT_CENTER,
            work.title,
            FontId::proportional(14.0),
            INK,
        );
        label.text(
            pos2(label_x, row_rect.center().y + 10.0),
            Align2::LEFT_CENTER,
            work.category,
            FontId::proportional(11.0),
            QUIET,
        );
        if app.marked[index] {
            let point = row_rect.right_center() - vec2(17.0, 0.0);
            painter.circle_filled(point, 7.0, SELECTED_LINE);
            painter.add(egui::Shape::line(
                vec![
                    point - vec2(3.0, 0.0),
                    point + vec2(-1.0, 2.0),
                    point + vec2(3.0, -2.0),
                ],
                Stroke::new(1.3, ACCENT),
            ));
        }
    }
}

fn draw_inspector(ui: &mut egui::Ui, app: &mut App, rect: Rect) {
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 12, SURFACE);
    painter.rect_stroke(rect, 12, Stroke::new(1.0, LINE), StrokeKind::Inside);

    let Some(index) = app.selected else {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Выбери работу для просмотра",
            FontId::proportional(13.0),
            QUIET,
        );
        return;
    };
    let work = &WORKS[index];
    let inner = rect.shrink(13.0);
    let cover_height = (inner.height() - 111.0).min(inner.width() * 0.72);
    let cover = Rect::from_min_size(inner.min, vec2(inner.width(), cover_height));
    paint_photo(&painter, app, index, cover, 7);
    let response = ui
        .interact(cover, ui.make_persistent_id("lumen-cover"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    response.widget_info(|| {
        WidgetInfo::labeled(WidgetType::Button, true, format!("Открыть: {}", work.title))
    });
    if response.hovered() || response.has_focus() {
        painter.rect_stroke(cover, 7, Stroke::new(1.0, ACCENT), StrokeKind::Inside);
    }
    if response.clicked() {
        app.open_preview();
    }

    let label_y = cover.bottom() + 15.0;
    let details = painter.with_clip_rect(inner);
    let meta = format!("{}  /  {}", work.place, work.year);
    details.text(
        pos2(inner.left(), label_y),
        Align2::LEFT_TOP,
        meta,
        FontId::proportional(11.0),
        ACCENT,
    );
    details.text(
        pos2(inner.left(), label_y + 18.0),
        Align2::LEFT_TOP,
        work.title,
        FontId::proportional(23.0),
        INK,
    );
    let description = details.layout(
        work.description.to_owned(),
        FontId::proportional(12.0),
        QUIET,
        inner.width(),
    );
    details.galley(pos2(inner.left(), label_y + 53.0), description, QUIET);
}

fn paint_photo(painter: &Painter, app: &App, index: usize, rect: Rect, radius: u8) {
    let Some(texture) = app.textures.get(index) else {
        painter.rect_filled(rect, radius, SELECTED);
        return;
    };
    let image = texture.size_vec2();
    let scale = (rect.width() / image.x).max(rect.height() / image.y);
    let uv_size = rect.size() / (image * scale);
    let uv = Rect::from_center_size(pos2(0.5, 0.5), uv_size);
    painter.add(
        egui::epaint::RectShape::filled(rect, radius, Color32::WHITE)
            .with_texture(texture.id(), uv),
    );
}

fn footer_button(
    ui: &mut egui::Ui,
    id: &str,
    center_left: Pos2,
    width: f32,
    label: &str,
    keys: &str,
    enabled: bool,
) -> bool {
    let rect = Rect::from_min_size(center_left - vec2(0.0, 16.0), vec2(width, 32.0));
    let mut response = ui.interact(
        rect,
        ui.make_persistent_id(("lumen-footer", id)),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if enabled {
        response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
    let painter = ui.painter();
    if enabled && (response.hovered() || response.has_focus()) {
        painter.rect_filled(rect, 7, HOVER);
    }
    if response.has_focus() {
        painter.rect_stroke(rect, 7, Stroke::new(1.0, ACCENT), StrokeKind::Inside);
    }

    let ink = if enabled { INK } else { QUIET };
    painter.text(
        center_left + vec2(7.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        ink,
    );
    let cap = Rect::from_center_size(rect.right_center() - vec2(24.0, 0.0), vec2(43.0, 21.0));
    painter.rect_filled(cap, 5, BACKGROUND);
    painter.rect_stroke(cap, 5, Stroke::new(1.0, LINE), StrokeKind::Inside);
    painter.text(
        cap.center(),
        Align2::CENTER_CENTER,
        keys,
        FontId::proportional(11.0),
        QUIET,
    );
    enabled && response.clicked()
}
