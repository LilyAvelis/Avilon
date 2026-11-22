use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontFamily, FontId, Painter, Pos2, Rect, Sense, Stroke,
    StrokeKind, UiBuilder, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{App, Overlay, SERIF, WORKS};

const PAGE: Color32 = Color32::from_rgb(243, 239, 218);
const FIELD: Color32 = Color32::from_rgb(223, 226, 200);
const DOT: Color32 = Color32::from_rgb(188, 197, 161);
const CARD: Color32 = Color32::from_rgb(250, 245, 222);
const LINE: Color32 = Color32::from_rgb(159, 171, 128);
const FERN: Color32 = Color32::from_rgb(51, 72, 53);
const STEM: Color32 = Color32::from_rgb(97, 115, 74);
const INK: Color32 = Color32::from_rgb(40, 52, 37);
const LIME: Color32 = Color32::from_rgb(215, 235, 158);
const LIGHT: Color32 = Color32::from_rgb(239, 242, 218);
const RUST: Color32 = Color32::from_rgb(155, 68, 44);
const UV: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

const ORIGINS: [(f32, f32); WORKS.len()] = [
    (0.18, 0.30),
    (0.47, 0.23),
    (0.81, 0.32),
    (0.24, 0.74),
    (0.54, 0.63),
    (0.84, 0.75),
];
const CONNECTIONS: [(usize, usize, &str); 5] = [
    (0, 5, "Вода"),
    (0, 2, "Дождь"),
    (1, 3, "Геометрия"),
    (1, 4, "Растения"),
    (2, 4, "Стекло"),
];

pub(super) struct State {
    offsets: [Vec2; WORKS.len()],
    zoom: f32,
    pan: Vec2,
    last_selected: Option<usize>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            offsets: [Vec2::ZERO; WORKS.len()],
            zoom: 1.0,
            pan: Vec2::ZERO,
            last_selected: None,
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, app: &mut App) {
    let window = ui.available_rect_before_wrap();
    let p = ui.painter_at(window);
    p.rect_filled(window, 0.0, PAGE);
    let inset = (window.width() * 0.027).clamp(17.0, 32.0);
    let left = window.left() + inset;
    let right = window.right() - inset;
    let top = window.top();
    let bottom = window.bottom();
    let inspector_width = (window.width() * 0.23).clamp(196.0, 263.0);
    let split = right - inspector_width;

    text(
        &p,
        pos2(left, top + 35.0),
        Align2::LEFT_CENTER,
        "Гербарий",
        serif(39.0),
        INK,
    );
    text(
        &p,
        pos2(left + 204.0, top + 39.0),
        Align2::LEFT_CENTER,
        "После дождя",
        serif(20.0),
        STEM,
    );

    let search = Rect::from_min_size(pos2(right - 244.0, top + 23.0), vec2(244.0, 32.0));
    p.hline(search.x_range(), search.bottom(), Stroke::new(1.0, LINE));
    ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
        ui.style_mut().spacing.interact_size.y = 30.0;
        let visuals = ui.visuals_mut();
        visuals.text_edit_bg_color = Some(PAGE);
        visuals.override_text_color = Some(INK);
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, STEM);
        app.search(ui, search.width() - 8.0);
    });

    let board = Rect::from_min_max(pos2(left, top + 69.0), pos2(split - 13.0, bottom - 41.0));
    draw_board(ui, app, board);
    let sidebar = Rect::from_min_max(pos2(split, board.top()), pos2(right, board.bottom()));
    draw_sidebar(ui, app, sidebar);

    let y = bottom - 22.0;
    text(
        &p,
        pos2(left, y),
        Align2::LEFT_CENTER,
        "Колесо — масштаб   ·   Перетащи карточку или холст",
        FontId::proportional(11.5),
        STEM,
    );
    let label = format!(
        "{}%  ·  Вписать",
        (app.herbarium.zoom * 100.0).round() as u32
    );
    let fit = Rect::from_min_size(pos2(board.right() - 122.0, y - 12.0), vec2(122.0, 24.0));
    if control(ui, &p, fit, "herb-fit", &label, None, false) {
        app.herbarium.zoom = 1.0;
        app.herbarium.pan = Vec2::ZERO;
    }
}

fn draw_board(ui: &mut egui::Ui, app: &mut App, board: Rect) {
    let mut matches = app.matches();
    let selected = app.selected;
    if let Some(index) = selected
        && let Some(slot) = matches.iter().position(|&item| item == index)
    {
        matches.remove(slot);
        matches.push(index);
    }

    let background = ui.interact(
        board,
        ui.make_persistent_id("herbarium-canvas"),
        Sense::drag(),
    );
    app.herbarium.pan += background.drag_delta();
    if background.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    }
    let wheel = ui.input(|input| {
        input
            .pointer
            .hover_pos()
            .filter(|cursor| board.contains(*cursor))
            .map(|cursor| (cursor, input.smooth_scroll_delta.y))
    });
    if let Some((cursor, delta)) = wheel
        && delta != 0.0
        && !matches!(app.overlay, Some(Overlay::Preview | Overlay::Settings))
    {
        let state = &mut app.herbarium;
        let next = (state.zoom * (delta * 0.003).exp()).clamp(0.55, 2.35);
        let anchor = cursor - board.center();
        state.pan = anchor + (state.pan - anchor) * (next / state.zoom);
        state.zoom = next;
    }

    if app.herbarium.last_selected != selected {
        app.herbarium.last_selected = selected;
        if let Some(index) = selected {
            let center = card_center(board, &app.herbarium, index);
            let visible = board.shrink2(card_size(board, &app.herbarium, app.compact) * 0.5);
            let corrected = pos2(
                center.x.clamp(visible.left(), visible.right()),
                center.y.clamp(visible.top(), visible.bottom()),
            );
            app.herbarium.pan += corrected - center;
        }
    }

    let size = card_size(board, &app.herbarium, app.compact);
    let hit_zone = board.shrink(1.0);
    let previous_clip = ui.clip_rect();
    ui.set_clip_rect(previous_clip.intersect(hit_zone));
    let mut hovered = [false; WORKS.len()];
    let mut focused = [false; WORKS.len()];
    for &index in &matches {
        let rect = Rect::from_center_size(card_center(board, &app.herbarium, index), size);
        let hit = rect.intersect(hit_zone);
        if !hit.is_positive() {
            continue;
        }
        let response = ui
            .interact(
                hit,
                ui.make_persistent_id(("herbarium-card", index)),
                Sense::click_and_drag(),
            )
            .on_hover_cursor(CursorIcon::Grab);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, WORKS[index].title));
        hovered[index] = response.hovered();
        focused[index] = response.has_focus();
        if response.dragged() {
            let scale = board.size() * app.herbarium.zoom;
            let (x, y) = ORIGINS[index];
            let offset = &mut app.herbarium.offsets[index];
            *offset += response.drag_delta() / scale;
            offset.x = offset.x.clamp(0.12 - x, 0.88 - x);
            offset.y = offset.y.clamp(0.19 - y, 0.81 - y);
            app.select(index);
            app.herbarium.last_selected = Some(index);
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }
        if response.clicked() {
            app.select(index);
            if response.double_clicked() {
                app.open_preview();
            }
        }
    }
    ui.set_clip_rect(previous_clip);

    let p = ui.painter_at(board);
    p.rect_filled(board, 16.0, FIELD);
    p.rect_stroke(board, 16.0, Stroke::new(1.0, DOT), StrokeKind::Inside);
    let grid = board.shrink(12.0);
    let step = (21.0 * app.herbarium.zoom).clamp(13.0, 33.0);
    let mut paper_marks = egui::Mesh::default();
    let mut y = grid.top() + app.herbarium.pan.y.rem_euclid(step);
    while y < grid.bottom() {
        let mut x = grid.left() + app.herbarium.pan.x.rem_euclid(step);
        while x < grid.right() {
            paper_marks
                .add_colored_rect(Rect::from_center_size(pos2(x, y), Vec2::splat(1.25)), DOT);
            x += step;
        }
        y += step;
    }
    p.add(egui::Shape::mesh(paper_marks));

    for (a, b, _) in CONNECTIONS {
        if !matches.contains(&a) || !matches.contains(&b) {
            continue;
        }
        let active = matches!(app.selected, Some(index) if index == a || index == b);
        let start = card_center(board, &app.herbarium, a);
        let end = card_center(board, &app.herbarium, b);
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let lift = 0.14 * dy + 25.0 * app.herbarium.zoom;
        p.add(egui::epaint::CubicBezierShape::from_points_stroke(
            [
                start,
                start + vec2(dx * 0.42, -lift),
                end - vec2(dx * 0.38, -lift),
                end,
            ],
            false,
            Color32::TRANSPARENT,
            Stroke::new(
                if active { 2.0 } else { 1.2 },
                if active { STEM } else { LINE },
            ),
        ));
        let leaf = start.lerp(end, 0.5);
        let axis = (end - start).normalized() * 4.8;
        let cross = vec2(-axis.y, axis.x) * 0.52;
        p.add(egui::Shape::convex_polygon(
            vec![leaf - axis, leaf + cross, leaf + axis, leaf - cross],
            if active { STEM } else { LINE },
            Stroke::NONE,
        ));
    }

    for &index in &matches {
        let rect = Rect::from_center_size(card_center(board, &app.herbarium, index), size);
        let active = app.selected == Some(index);
        let shadow = Color32::from_black_alpha(if active { 28 } else { 14 });
        p.rect_filled(rect.translate(vec2(2.0, 4.0)), 3.0, shadow);
        p.rect_filled(rect, 3.0, CARD);
        let stroke = if active || focused[index] {
            Some(FERN)
        } else if hovered[index] {
            Some(STEM)
        } else {
            None
        };
        if let Some(color) = stroke {
            p.rect_stroke(rect, 3.0, Stroke::new(2.0, color), StrokeKind::Inside);
        }

        let inset = (size.x * 0.054).clamp(5.0, 10.0);
        let baseline = rect.bottom() - (size.y * 0.12).clamp(8.0, 17.0);
        let image = Rect::from_min_max(
            rect.min + vec2(inset, inset),
            pos2(rect.right() - inset, baseline - 10.0),
        );
        photo(&p, app, index, image);
        let title = &WORKS[index].title;
        let clipped = p.with_clip_rect(Rect::from_min_max(
            pos2(rect.left() + 4.0, image.bottom()),
            pos2(
                rect.right() - if app.marked[index] { 13.0 } else { 4.0 },
                rect.bottom(),
            ),
        ));
        text(
            &clipped,
            pos2(rect.left() + inset, baseline),
            Align2::LEFT_CENTER,
            title,
            serif((size.x * 0.086).clamp(9.0, 16.0)),
            INK,
        );
        if app.marked[index] {
            p.circle_filled(pos2(rect.right() - inset, baseline), 3.0, RUST);
        }
    }
    if matches.is_empty() {
        text(
            &p,
            board.center(),
            Align2::CENTER_CENTER,
            "На холсте нет совпадений",
            serif(23.0),
            STEM,
        );
    }
}

fn card_size(board: Rect, state: &State, compact: bool) -> Vec2 {
    let density = if compact { 0.8 } else { 1.0 };
    let preferred = (board.width() * 0.228).clamp(95.0, 177.0) * state.zoom * density;
    let width = preferred
        .min(board.width() * 0.68)
        .min(board.height() * 0.49);
    vec2(width, width * 0.87)
}

fn card_center(board: Rect, state: &State, index: usize) -> Pos2 {
    let (x, y) = ORIGINS[index];
    let ratio = vec2(x - 0.5, y - 0.5) + state.offsets[index];
    board.center() + state.pan + ratio * board.size() * state.zoom
}

fn draw_sidebar(ui: &mut egui::Ui, app: &mut App, rect: Rect) {
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 16.0, FERN);
    let inset = 15.0;
    let left = rect.left() + inset;
    let right = rect.right() - inset;
    let y = rect.top() + 24.0;
    text(
        &p,
        pos2(left, y),
        Align2::LEFT_CENTER,
        "ВЫБРАННЫЙ КАДР",
        FontId::proportional(10.5),
        LIME,
    );
    if control(
        ui,
        &p,
        Rect::from_center_size(pos2(right - 9.0, y), vec2(30.0, 27.0)),
        "herb-settings",
        "•••",
        None,
        true,
    ) {
        app.open_settings();
    }
    let bottom = rect.bottom() - 13.0;
    let action = Rect::from_min_max(pos2(left, bottom - 35.0), pos2(right, bottom));
    if control(
        ui,
        &p,
        action,
        "herb-actions",
        "Действия",
        Some("Ctrl+K"),
        true,
    ) {
        if app.overlay == Some(Overlay::Actions) {
            app.close_overlay();
        } else {
            app.open_actions();
        }
    }
    let Some(index) = app.selected else {
        text(
            &p,
            pos2(left, y + 38.0),
            Align2::LEFT_CENTER,
            "Кадр не выбран",
            serif(20.0),
            LIGHT,
        );
        return;
    };

    let work = &WORKS[index];
    let image_bottom = y + (rect.height() * 0.32).clamp(80.0, 176.0);
    let image = Rect::from_min_max(pos2(left, y + 23.0), pos2(right, image_bottom));
    photo(&p, app, index, image);
    if ui
        .interact(
            image,
            ui.make_persistent_id("herb-open-photo"),
            Sense::click(),
        )
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
    {
        app.open_preview();
    }
    let title = p.layout(work.title.into(), serif(24.0), LIGHT, image.width());
    let meta_y = image.bottom() + 17.0 + title.size().y;
    p.galley(pos2(left, image.bottom() + 8.0), title, LIGHT);
    let metadata = format!("{} · {}", work.place, work.category);
    text(
        &p,
        pos2(left, meta_y),
        Align2::LEFT_TOP,
        &metadata,
        FontId::proportional(12.0),
        LIME,
    );
    if app.marked[index] {
        p.circle_filled(pos2(right - 4.0, meta_y + 6.0), 3.0, LIME);
    }

    let view = action.translate(vec2(0.0, -42.0));
    if control(
        ui,
        &p,
        view,
        "herb-view",
        "Открыть кадр",
        Some("Enter"),
        true,
    ) {
        app.open_preview();
    }
    let label_y = meta_y + 32.0;
    if view.top() > label_y + 32.0 {
        p.hline(left..=right, label_y - 7.0, Stroke::new(1.0, STEM));
        text(
            &p,
            pos2(left, label_y),
            Align2::LEFT_TOP,
            "СВЯЗИ НА ХОЛСТЕ",
            FontId::proportional(10.5),
            LIME,
        );
        for (row, (related, label)) in CONNECTIONS
            .into_iter()
            .filter_map(|(a, b, label)| {
                if a == index {
                    Some((b, label))
                } else if b == index {
                    Some((a, label))
                } else {
                    None
                }
            })
            .enumerate()
        {
            let y = label_y + 24.0 + row as f32 * 23.0;
            if y + 10.0 > view.top() {
                break;
            }
            let related_title = WORKS[related].title;
            let available = app.matches().contains(&related);
            let edge = Rect::from_min_max(pos2(left, y - 9.0), pos2(right, y + 9.0));
            let response = ui
                .interact(
                    edge,
                    ui.make_persistent_id(("herb-link", related)),
                    Sense::click(),
                )
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(format!("{label} · {related_title}"));
            let color = if response.hovered() { LIME } else { LIGHT };
            p.circle_filled(pos2(left + 4.0, y), 2.4, color);
            text(
                &p,
                pos2(left + 12.0, y),
                Align2::LEFT_CENTER,
                related_title,
                FontId::proportional(11.5),
                color,
            );
            if response.clicked() && available {
                app.select(related);
            } else if response.clicked() {
                app.query.clear();
                app.select(related);
            }
        }
    }
}

fn control(
    ui: &mut egui::Ui,
    p: &Painter,
    rect: Rect,
    id: &'static str,
    name: &str,
    hint: Option<&str>,
    dark: bool,
) -> bool {
    let response = ui
        .interact(rect, ui.make_persistent_id(id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    let accessible = if id == "herb-settings" {
        "Настройки"
    } else {
        name
    };
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, accessible));
    let hot = response.hovered() || response.has_focus();
    if let Some(hint) = hint {
        let radius = rect.height() * 0.5;
        p.rect_filled(
            rect,
            radius,
            if hot {
                LIME
            } else {
                Color32::from_rgb(193, 214, 151)
            },
        );
        text(
            p,
            pos2(rect.left() + 12.0, rect.center().y),
            Align2::LEFT_CENTER,
            name,
            FontId::proportional(13.0),
            INK,
        );
        text(
            p,
            pos2(rect.right() - 11.0, rect.center().y),
            Align2::RIGHT_CENTER,
            hint,
            FontId::proportional(10.0),
            FERN,
        );
    } else {
        text(
            p,
            rect.center(),
            Align2::CENTER_CENTER,
            name,
            serif(if dark { 15.0 } else { 12.0 }),
            if dark { LIME } else { INK },
        );
        if hot {
            p.hline(
                rect.center().x - 28.0..=rect.center().x + 28.0,
                rect.bottom() - 1.0,
                Stroke::new(1.0, STEM),
            );
        }
    }
    response.clicked()
}

fn photo(p: &Painter, app: &App, index: usize, rect: Rect) {
    let ratio = rect.aspect_ratio();
    let uv = Rect::from_center_size(UV.center(), vec2(ratio.min(1.0), ratio.recip().min(1.0)));
    p.image(app.textures[index].id(), rect, uv, Color32::WHITE);
}

fn text(p: &Painter, at: Pos2, align: Align2, label: &str, font: FontId, color: Color32) {
    p.text(at, align, label, font, color);
}

fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SERIF.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_a_card_moves_its_anchor_without_panning_the_board() {
        let ctx = egui::Context::default();
        let board = Rect::from_min_size(Pos2::ZERO, vec2(740.0, 520.0));
        let mut app = App::new(&ctx, crate::Edition::Herbarium);
        let run = |app: &mut App, events| {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(board),
                    events,
                    ..Default::default()
                },
                |ui| draw_board(ui, app, board),
            )
            .textures_delta
            .clear();
        };
        run(&mut app, Vec::new());

        let start = card_center(board, &app.herbarium, 1);
        let end = start + vec2(48.0, 37.0);
        run(&mut app, vec![egui::Event::PointerMoved(start)]);
        run(
            &mut app,
            vec![egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        run(&mut app, vec![egui::Event::PointerMoved(end)]);
        run(
            &mut app,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        assert_eq!(app.selected, Some(1));
        assert_eq!(app.herbarium.pan, Vec2::ZERO);
        assert!(card_center(board, &app.herbarium, 1).distance(end) < 0.1);
    }
}
