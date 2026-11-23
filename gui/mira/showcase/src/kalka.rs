use eframe::egui::{
    self, Align2, Color32, CursorIcon, EventFilter, FocusDirection, FontFamily, FontId, Key,
    Painter, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, TextureHandle, UiBuilder,
    WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{App, Overlay, SERIF, WORKS};

const DESK: Color32 = Color32::from_rgb(234, 237, 229);
const MAT: Color32 = Color32::from_rgb(221, 231, 225);
const PAPER: Color32 = Color32::from_rgb(250, 249, 239);
const FROST: Color32 = Color32::from_rgb(228, 239, 241);
const INK: Color32 = Color32::from_rgb(29, 51, 62);
const QUIET: Color32 = Color32::from_rgb(81, 103, 111);
const BLUE: Color32 = Color32::from_rgb(35, 77, 141);
const LINE: Color32 = Color32::from_rgb(163, 186, 187);
const ACCENT: Color32 = Color32::from_rgb(255, 221, 125);
const TEXTURE_SHADOW: Color32 = Color32::from_black_alpha(34);
const TRANSMISSIONS: [u8; 3] = [97, 178, 255];

pub(super) struct State {
    layer: usize,
    crease: f32,
    opacity: usize,
    pick_layer: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            layer: 1,
            crease: 0.56,
            opacity: 1,
            pick_layer: false,
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, app: &mut App) {
    let window = ui.available_rect_before_wrap();
    let p = ui.painter_at(window);
    p.rect_filled(window, 0, DESK);
    let margin = (window.width() * 0.031).clamp(20.0, 39.0);
    let left = window.left() + margin;
    let right = window.right() - margin;
    let rail_width = (window.width() * 0.235).clamp(216.0, 270.0);
    let body_top = window.top() + 81.0;
    let body_bottom = window.bottom() - 68.0;
    let rail = Rect::from_min_max(pos2(right - rail_width, body_top), pos2(right, body_bottom));
    let platen = Rect::from_min_max(pos2(left, body_top), pos2(rail.left() - 22.0, body_bottom));

    header(ui, &p, app, pos2(left, window.top()), right);
    let base = app.selected;
    if base == Some(app.kalka.layer) {
        app.kalka.layer = (app.kalka.layer + 1) % WORKS.len();
    }
    table(ui, &p, app, platen, base);
    if app.overlay == Some(Overlay::Actions) {
        actions(ui, &p, app, rail);
    } else {
        tray(ui, &p, app, rail);
    }
    footer(ui, &p, app, pos2(left, body_bottom + 23.0), right);
}

fn header(ui: &mut egui::Ui, p: &Painter, app: &mut App, origin: Pos2, right: f32) {
    let icon = origin + vec2(17.0, 40.0);
    let sheet = Rect::from_center_size(icon, vec2(22.0, 28.0));
    p.rect_stroke(
        sheet.translate(vec2(-4.0, 3.0)),
        0,
        Stroke::new(1.0, BLUE),
        StrokeKind::Inside,
    );
    p.add(Shape::convex_polygon(
        vec![
            sheet.left_top(),
            sheet.right_top() - vec2(7.0, 0.0),
            sheet.right_top() + vec2(0.0, 7.0),
            sheet.right_bottom(),
            sheet.left_bottom(),
        ],
        FROST,
        Stroke::new(1.0, BLUE),
    ));
    p.line_segment(
        [
            sheet.right_top() - vec2(7.0, 0.0),
            sheet.right_top() - vec2(7.0, -7.0),
        ],
        Stroke::new(1.0, BLUE),
    );
    p.hline(
        sheet.right() - 7.0..=sheet.right(),
        sheet.top() + 7.0,
        Stroke::new(1.0, BLUE),
    );
    p.text(
        origin + vec2(44.0, 39.0),
        Align2::LEFT_CENTER,
        "Калька",
        serif(39.0),
        INK,
    );
    p.text(
        origin + vec2(178.0, 33.0),
        Align2::LEFT_CENTER,
        "После дождя",
        FontId::proportional(16.0),
        INK,
    );
    p.text(
        origin + vec2(178.0, 53.0),
        Align2::LEFT_CENTER,
        "Световой стол",
        FontId::proportional(12.0),
        QUIET,
    );

    let search = Rect::from_min_size(pos2(right - 239.0, origin.y + 26.0), vec2(239.0, 29.0));
    p.hline(search.x_range(), search.bottom(), Stroke::new(1.0, LINE));
    ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
        ui.spacing_mut().interact_size.y = 27.0;
        let visuals = ui.visuals_mut();
        visuals.override_text_color = Some(INK);
        visuals.text_edit_bg_color = Some(DESK);
        visuals.widgets.inactive.bg_stroke = Stroke::new(0.0, DESK);
        app.search(ui, search.width() - 8.0);
    });
}

fn table(ui: &mut egui::Ui, p: &Painter, app: &mut App, platen: Rect, base: Option<usize>) {
    p.rect_filled(platen, 2, MAT);
    let grid = p.with_clip_rect(platen.shrink(1.0));
    for i in 0..(platen.width() / 24.0).ceil() as usize {
        let x = platen.left() + i as f32 * 24.0;
        grid.vline(
            x,
            platen.y_range(),
            Stroke::new(1.0, Color32::from_rgb(207, 222, 216)),
        );
    }
    for i in 0..(platen.height() / 24.0).ceil() as usize {
        let y = platen.top() + i as f32 * 24.0;
        grid.hline(
            platen.x_range(),
            y,
            Stroke::new(1.0, Color32::from_rgb(207, 222, 216)),
        );
    }
    p.rect_stroke(platen, 2, Stroke::new(1.0, LINE), StrokeKind::Inside);

    let Some(base) = base else {
        p.text(
            platen.center(),
            Align2::CENTER_CENTER,
            "Снимки не найдены",
            serif(23.0),
            QUIET,
        );
        return;
    };
    let layer = app.kalka.layer;
    let frame = Rect::from_min_max(platen.min + vec2(25.0, 43.0), platen.max - vec2(25.0, 59.0));
    let image = frame.shrink(9.0);
    let axis = Rect::from_min_max(
        pos2(image.left() + 4.0, image.top()),
        pos2(image.right() - 4.0, image.bottom()),
    );
    let fold_x = egui::lerp(axis.x_range(), app.kalka.crease);
    let drag_zone = Rect::from_min_max(
        pos2(fold_x - 13.0, frame.top() - 16.0),
        pos2(fold_x + 13.0, frame.bottom() + 12.0),
    );
    let fold = ui
        .interact(
            drag_zone,
            ui.make_persistent_id("kalka-crease"),
            Sense::click_and_drag(),
        )
        .on_hover_cursor(CursorIcon::ResizeHorizontal);
    if fold.drag_started() {
        fold.request_focus();
    }
    let moved = fold.dragged() || fold.drag_stopped();
    if moved && let Some(pointer) = fold.interact_pointer_pos() {
        app.kalka.crease = ((pointer.x - axis.left()) / axis.width()).clamp(0.0, 1.0);
        ui.ctx().request_repaint();
    }
    if fold.has_focus() && app.overlay.is_none() {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                fold.id,
                EventFilter {
                    horizontal_arrows: true,
                    ..Default::default()
                },
            );
        });
        let right = crate::consume_key(ui.ctx(), Key::ArrowRight, false);
        let left = crate::consume_key(ui.ctx(), Key::ArrowLeft, false);
        if right != left {
            app.kalka.crease =
                (app.kalka.crease + (right as f32 - left as f32) * 0.045).clamp(0.0, 1.0);
            ui.memory_mut(|memory| memory.move_focus(FocusDirection::None));
        }
    }
    fold.widget_info(|| {
        WidgetInfo::slider(true, f64::from(1.0 - app.kalka.crease), "Покрытие калькой")
    });
    let fold_x = egui::lerp(axis.x_range(), app.kalka.crease);
    if fold.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
    }

    let image_response = interact(ui, image, "kalka-preview", "Открыть основу", Sense::click());
    if image_response.double_clicked() && !fold.hovered() {
        app.open_preview();
    }
    p.rect_filled(frame.translate(vec2(4.0, 5.0)), 0, TEXTURE_SHADOW);
    p.rect_filled(frame, 0, PAPER);
    photo(p, image, &app.textures[base], Color32::WHITE);

    let veil = Rect::from_min_max(pos2(fold_x, image.top()), image.right_bottom());
    let transmission = TRANSMISSIONS[app.kalka.opacity];
    let filtered = p.with_clip_rect(veil);
    photo(
        &filtered,
        image,
        &app.textures[layer],
        Color32::from_white_alpha(transmission),
    );
    if transmission < 255 {
        filtered.rect_filled(image, 0, Color32::from_white_alpha(17));
    }
    p.line_segment(
        [
            pos2(fold_x + 2.0, frame.top()),
            pos2(fold_x + 2.0, frame.bottom()),
        ],
        Stroke::new(3.0, Color32::from_black_alpha(28)),
    );
    p.line_segment(
        [
            pos2(fold_x, frame.top() - 4.0),
            pos2(fold_x, frame.bottom() + 4.0),
        ],
        Stroke::new(1.5, Color32::WHITE),
    );

    let fold_tip = pos2(fold_x, frame.top() - 4.0);
    let face_right = frame.right() - fold_x >= 76.0;
    let direction = if face_right { 1.0 } else { -1.0 };
    let mut edge = vec![
        fold_tip,
        fold_tip + vec2(17.0 * direction, -18.0),
        fold_tip + vec2(81.0 * direction, -18.0),
        fold_tip + vec2(81.0 * direction, 0.0),
    ];
    if !face_right {
        edge.reverse();
    }
    p.add(Shape::convex_polygon(
        edge,
        Color32::from_rgba_unmultiplied(237, 249, 250, 229),
        Stroke::new(1.0, LINE),
    ));
    p.text(
        fold_tip + vec2(47.0 * direction, -9.0),
        Align2::CENTER_CENTER,
        "СГИБ",
        FontId::monospace(9.0),
        BLUE,
    );

    let grip = Rect::from_center_size(pos2(fold_x, image.center().y), vec2(26.0, 38.0));
    p.rect_filled(grip.translate(vec2(0.0, 2.0)), 5, TEXTURE_SHADOW);
    p.rect_filled(
        grip,
        5,
        if fold.hovered() || fold.has_focus() || moved {
            ACCENT
        } else {
            PAPER
        },
    );
    p.text(
        grip.center(),
        Align2::CENTER_CENTER,
        "↔",
        FontId::proportional(16.0),
        BLUE,
    );
    registration(p, frame);

    let caption = frame.left_bottom() + vec2(0.0, 23.0);
    label(p, caption, "ОСНОВА", BLUE);
    p.text(
        caption + vec2(60.0, 0.0),
        Align2::LEFT_CENTER,
        WORKS[base].title,
        FontId::proportional(13.0),
        INK,
    );
    let right_caption = frame.right_bottom() + vec2(0.0, 23.0);
    p.text(
        right_caption,
        Align2::RIGHT_CENTER,
        WORKS[layer].title,
        FontId::proportional(13.0),
        INK,
    );
    let size = p
        .layout_no_wrap(WORKS[layer].title.into(), FontId::proportional(13.0), INK)
        .size();
    p.text(
        right_caption - vec2(size.x + 10.0, 0.0),
        Align2::RIGHT_CENTER,
        "КАЛЬКА",
        FontId::monospace(10.0),
        BLUE,
    );
}

fn registration(p: &Painter, rect: Rect) {
    for (at, sign) in [
        (rect.left_top(), vec2(-1.0, -1.0)),
        (rect.right_top(), vec2(1.0, -1.0)),
        (rect.left_bottom(), vec2(-1.0, 1.0)),
        (rect.right_bottom(), vec2(1.0, 1.0)),
    ] {
        p.line_segment(
            [at + vec2(sign.x * 3.0, 0.0), at + vec2(sign.x * 12.0, 0.0)],
            Stroke::new(1.0, BLUE),
        );
        p.line_segment(
            [at + vec2(0.0, sign.y * 3.0), at + vec2(0.0, sign.y * 12.0)],
            Stroke::new(1.0, BLUE),
        );
    }
}

fn tray(ui: &mut egui::Ui, p: &Painter, app: &mut App, rail: Rect) {
    p.rect_filled(rail.translate(vec2(3.0, 4.0)), 0, TEXTURE_SHADOW);
    p.rect_filled(rail, 0, PAPER);
    p.rect_stroke(rail, 0, Stroke::new(1.0, LINE), StrokeKind::Inside);
    let origin = rail.min + vec2(13.0, 16.0);
    p.text(origin, Align2::LEFT_TOP, "Листы", serif(24.0), INK);
    let switch = Rect::from_min_size(origin + vec2(0.0, 39.0), vec2(rail.width() - 26.0, 25.0));
    let first = Rect::from_min_max(switch.min, pos2(switch.center().x - 3.0, switch.bottom()));
    let second = Rect::from_min_max(pos2(switch.center().x + 3.0, switch.top()), switch.max);
    if paper_tab(
        ui,
        p,
        first,
        "kalka-basis-tab",
        "Основа",
        !app.kalka.pick_layer,
    ) {
        app.kalka.pick_layer = false;
    }
    if paper_tab(
        ui,
        p,
        second,
        "kalka-sheet-tab",
        "Калька",
        app.kalka.pick_layer,
    ) {
        app.kalka.pick_layer = true;
    }
    let work_top = switch.bottom() + 12.0;
    let step = ((rail.bottom() - work_top - 49.0) / WORKS.len() as f32).min(if app.compact {
        42.0
    } else {
        53.0
    });
    let matches = app.matches();
    if matches.is_empty() {
        p.text(
            pos2(rail.center().x, work_top + 30.0),
            Align2::CENTER_CENTER,
            "Нет совпадений",
            FontId::proportional(13.0),
            QUIET,
        );
    }
    for (position, index) in matches.iter().copied().enumerate() {
        let row = Rect::from_min_size(
            pos2(switch.left(), work_top + step * position as f32),
            vec2(switch.width(), step - 4.0),
        );
        let response = interact(
            ui,
            row,
            ("kalka-photo", index),
            WORKS[index].title,
            Sense::click(),
        )
        .on_hover_cursor(CursorIcon::PointingHand);
        let base = app.selected == Some(index);
        let layer = app.kalka.layer == index;
        if base || layer || response.hovered() || response.has_focus() {
            p.rect_filled(
                row,
                0,
                if base {
                    Color32::from_rgb(217, 230, 220)
                } else {
                    FROST
                },
            );
            p.rect_filled(
                Rect::from_min_size(row.min, vec2(3.0, row.height())),
                0,
                if base { BLUE } else { LINE },
            );
        }
        let thumb_side = (row.height() - 7.0).clamp(24.0, 38.0);
        let thumb = Rect::from_center_size(
            pos2(row.left() + thumb_side * 0.5 + 8.0, row.center().y),
            vec2(thumb_side, thumb_side),
        );
        photo(p, thumb, &app.textures[index], Color32::WHITE);
        let at = pos2(thumb.right() + 8.0, row.center().y - 6.0);
        p.text(
            at,
            Align2::LEFT_CENTER,
            WORKS[index].title,
            FontId::proportional(12.0),
            INK,
        );
        label(
            p,
            at + vec2(0.0, 13.0),
            if base {
                "ОСНОВА"
            } else if layer {
                "КАЛЬКА"
            } else {
                WORKS[index].place
            },
            QUIET,
        );
        if app.marked[index] {
            p.circle_filled(pos2(row.right() - 7.0, row.center().y), 3.0, BLUE);
        }
        if response.clicked() {
            if app.kalka.pick_layer {
                if !base {
                    app.kalka.layer = index;
                }
            } else {
                app.select(index);
            }
        }
    }
    p.hline(
        switch.x_range(),
        rail.bottom() - 43.0,
        Stroke::new(1.0, LINE),
    );
    if let Some(previous) = app.selected
        && previous != app.kalka.layer
        && matches.contains(&app.kalka.layer)
    {
        let swap = Rect::from_min_size(
            pos2(switch.left(), rail.bottom() - 33.0),
            vec2(switch.width(), 23.0),
        );
        if paper_tab(ui, p, swap, "kalka-swap", "Поменять местами", false) {
            app.select(app.kalka.layer);
            app.kalka.layer = previous;
        }
    }
}

fn actions(ui: &mut egui::Ui, p: &Painter, app: &mut App, rail: Rect) {
    p.rect_filled(rail.translate(vec2(3.0, 5.0)), 0, TEXTURE_SHADOW);
    p.rect_filled(rail, 0, FROST);
    p.rect_stroke(rail, 0, Stroke::new(1.0, LINE), StrokeKind::Inside);
    let origin = rail.min + vec2(14.0, 17.0);
    p.text(origin, Align2::LEFT_TOP, "Откидной лист", serif(23.0), INK);
    p.text(
        origin + vec2(0.0, 33.0),
        Align2::LEFT_TOP,
        "Действия · ↑ ↓ и Enter",
        FontId::proportional(12.0),
        QUIET,
    );
    let actions = app.actions();
    let mut choice = None;
    for (index, action) in actions.into_iter().enumerate() {
        let strip = Rect::from_min_size(
            origin + vec2(0.0, 62.0 + index as f32 * 48.0),
            vec2(rail.width() - 28.0, 42.0),
        );
        if paper_tab(
            ui,
            p,
            strip,
            ("kalka-action", index),
            app.action_label(action),
            index == app.action_index,
        ) {
            choice = Some(action);
        }
    }
    let close = Rect::from_min_size(
        pos2(origin.x, rail.bottom() - 41.0),
        vec2(rail.width() - 28.0, 26.0),
    );
    if paper_tab(ui, p, close, "kalka-hide-actions", "Сложить · Esc", false) {
        app.close_overlay();
    }
    if let Some(action) = choice {
        app.execute(ui.ctx(), action);
    }
}

fn footer(ui: &mut egui::Ui, p: &Painter, app: &mut App, origin: Pos2, right: f32) {
    let p0 = origin + vec2(0.0, 0.0);
    let p1 = origin + vec2(83.0, 0.0);
    label(p, p0, "ПРОСВЕТ", QUIET);
    for (i, value) in ["Мягкий", "Средний", "Плотный"].into_iter().enumerate() {
        let area = Rect::from_min_size(p1 + vec2(i as f32 * 66.0, -11.0), vec2(62.0, 23.0));
        if paper_tab(
            ui,
            p,
            area,
            ("kalka-density", i),
            value,
            app.kalka.opacity == i,
        ) {
            app.kalka.opacity = i;
        }
    }
    p.text(
        origin + vec2(0.0, 25.0),
        Align2::LEFT_CENTER,
        "Потяни сгиб, чтобы сравнить снимки",
        FontId::proportional(11.0),
        QUIET,
    );
    let preview = Rect::from_min_size(pos2(right - 245.0, origin.y - 12.0), vec2(121.0, 25.0));
    if paper_tab(
        ui,
        p,
        preview,
        "kalka-open-photo",
        "Просмотр · Enter",
        false,
    ) {
        app.open_preview();
    }
    let commands = Rect::from_min_size(pos2(right - 117.0, origin.y - 12.0), vec2(117.0, 25.0));
    let opened = app.overlay == Some(Overlay::Actions);
    if paper_tab(
        ui,
        p,
        commands,
        "kalka-open-actions",
        "Действия · Ctrl+K",
        opened,
    ) {
        if opened {
            app.close_overlay();
        } else {
            app.open_actions();
        }
    }
}

fn interact(
    ui: &mut egui::Ui,
    rect: Rect,
    id: impl egui::AsIdSalt,
    name: &str,
    sense: Sense,
) -> Response {
    let response = ui.interact(rect, ui.make_persistent_id(id), sense);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name));
    response
}

fn paper_tab(
    ui: &mut egui::Ui,
    p: &Painter,
    area: Rect,
    id: impl egui::AsIdSalt,
    text: &str,
    active: bool,
) -> bool {
    let response =
        interact(ui, area, id, text, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    let hot = response.hovered() || response.has_focus();
    let top_fold = 8.0;
    p.add(Shape::convex_polygon(
        vec![
            area.left_top(),
            area.right_top() - vec2(top_fold, 0.0),
            area.right_top() + vec2(0.0, top_fold),
            area.right_bottom(),
            area.left_bottom(),
        ],
        if active {
            ACCENT
        } else if hot {
            Color32::WHITE
        } else {
            PAPER
        },
        Stroke::new(1.0, if response.has_focus() { BLUE } else { LINE }),
    ));
    p.text(
        area.center() - vec2(2.0, 0.0),
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.0),
        if active { INK } else { BLUE },
    );
    response.clicked()
}

fn label(p: &Painter, point: Pos2, text: &str, color: Color32) {
    p.text(
        point,
        Align2::LEFT_CENTER,
        text,
        FontId::monospace(10.0),
        color,
    );
}

fn photo(p: &Painter, area: Rect, texture: &TextureHandle, color: Color32) {
    let ratio = area.aspect_ratio() / texture.aspect_ratio();
    let visible = vec2(ratio.min(1.0), ratio.recip().min(1.0));
    p.image(
        texture.id(),
        area,
        Rect::from_center_size(pos2(0.5, 0.5), visible),
        color,
    );
}

fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SERIF.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Edition;
    use egui::{Event, Id, Modifiers, PointerButton, RawInput};

    #[test]
    fn fold_moves_and_commands_leave_the_table_in_place_at_minimum_size() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, Edition::Kalka);
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 590.0));
        let mut frame = |events: Vec<Event>| {
            let mut ids = [Id::NULL; 3];
            let _ = ctx.run_ui(
                RawInput {
                    screen_rect: Some(screen),
                    events,
                    ..Default::default()
                },
                |ui| {
                    ids = [
                        ui.make_persistent_id("kalka-crease"),
                        ui.make_persistent_id("kalka-preview"),
                        ui.make_persistent_id(("kalka-action", 0_usize)),
                    ];
                    app.show(ui);
                },
            );
            (ids, app.kalka.crease)
        };
        let (ids, initial_crease) = frame(vec![]);
        let at = ctx.read_response(ids[0]).unwrap().rect.center();
        let initial_image = ctx.read_response(ids[1]).unwrap().rect;
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        frame(vec![Event::PointerMoved(at), button(at, true)]);
        let pulled = at - vec2(100.0, 0.0);
        let (_, after_pull) = frame(vec![Event::PointerMoved(pulled)]);
        assert!(after_pull < initial_crease - 0.2);
        frame(vec![button(pulled, false)]);
        let ctrl_k = |pressed| Event::Key {
            key: Key::K,
            physical_key: Some(Key::K),
            pressed,
            repeat: false,
            modifiers: Modifiers::CTRL,
        };
        let (_, after_palette) = frame(vec![ctrl_k(true), ctrl_k(false)]);
        assert_eq!(after_pull, after_palette);
        assert_eq!(ctx.read_response(ids[1]).unwrap().rect, initial_image);
        let command = ctx.read_response(ids[2]).unwrap().rect;
        assert!(screen.contains_rect(command));
        assert!(initial_image.right() < command.left());
    }
}
