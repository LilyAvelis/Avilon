use std::f32::consts::PI;

use eframe::egui::{
    self, Align2, Color32, CursorIcon, EventFilter, FocusDirection, FontId, Key, Modifiers,
    Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, UiBuilder, Vec2, WidgetInfo,
    WidgetType, pos2, vec2,
};

use crate::{Action, App, Overlay, WORKS};

const CHASSIS: Color32 = Color32::from_rgb(223, 226, 214);
const STEEL: Color32 = Color32::from_rgb(192, 203, 185);
const INK: Color32 = Color32::from_rgb(31, 45, 38);
const QUIET: Color32 = Color32::from_rgb(68, 87, 73);
const BEZEL: Color32 = Color32::from_rgb(30, 44, 38);
const SCREEN: Color32 = Color32::from_rgb(16, 25, 22);
const SCREEN_TEXT: Color32 = Color32::from_rgb(231, 239, 220);
const SCREEN_QUIET: Color32 = Color32::from_rgb(167, 189, 168);
const JADE: Color32 = Color32::from_rgb(126, 230, 198);
const ORANGE: Color32 = Color32::from_rgb(255, 180, 126);
const CITRON: Color32 = Color32::from_rgb(220, 236, 139);

pub(super) struct State {
    zoom: f32,
    density: f32,
    signal: f32,
}

const ZOOM: f32 = 0.12;
const DENSITY: f32 = 0.42;
const SIGNAL: f32 = 0.75;

impl Default for State {
    fn default() -> Self {
        Self {
            zoom: ZOOM,
            density: DENSITY,
            signal: SIGNAL,
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, app: &mut App) {
    let window = ui.available_rect_before_wrap();
    let p = ui.painter_at(window);
    p.rect_filled(window, 0, CHASSIS);
    let margin = (window.width() * 0.028).clamp(19.0, 31.0);
    let left = window.left() + margin;
    let right = window.right() - margin;
    let top = window.top();
    let bottom = window.bottom();
    let control_y = bottom - 152.0;

    text(
        &p,
        pos2(left, top + 34.0),
        Align2::LEFT_CENTER,
        "ПУЛЬТ",
        FontId::proportional(31.0),
        INK,
    );
    p.vline(
        left + 105.0,
        top + 23.0..=top + 46.0,
        Stroke::new(1.0, STEEL),
    );
    text(
        &p,
        pos2(left + 120.0, top + 34.0),
        Align2::LEFT_CENTER,
        "После дождя",
        FontId::proportional(14.0),
        QUIET,
    );

    let search = Rect::from_min_size(pos2(right - 235.0, top + 23.0), vec2(235.0, 29.0));
    ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
        ui.spacing_mut().interact_size.y = 26.0;
        let visuals = ui.visuals_mut();
        visuals.override_text_color = Some(INK);
        visuals.text_edit_bg_color = Some(Color32::from_rgb(237, 242, 225));
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, STEEL);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, QUIET);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, INK);
        app.search(ui, search.width() - 9.0);
    });

    controls(
        ui,
        &p,
        app,
        Rect::from_min_max(pos2(left, control_y), pos2(right, bottom - 16.0)),
    );
    let console = Rect::from_min_max(pos2(left, top + 61.0), pos2(right, control_y - 12.0));
    monitor(ui, &p, app, console);
}

fn monitor(ui: &mut egui::Ui, p: &Painter, app: &mut App, outer: Rect) {
    p.rect_filled(
        outer.translate(vec2(0.0, 3.0)),
        15,
        Color32::from_rgb(165, 174, 160),
    );
    p.rect_filled(outer, 15, BEZEL);
    p.rect_stroke(
        outer,
        15,
        Stroke::new(1.0, Color32::from_rgb(63, 81, 68)),
        StrokeKind::Inside,
    );
    let screen = outer.shrink(11.0);
    p.rect_filled(screen, 7, SCREEN);
    let screen = screen.shrink(13.0);
    let track_width = (screen.width() * 0.34).clamp(215.0, 325.0);
    let divider = screen.right() - track_width;
    let photograph = Rect::from_min_max(screen.min, pos2(divider - 16.0, screen.bottom() - 38.0));
    let signal = mix(Color32::from_rgb(115, 146, 119), CITRON, app.pult.signal);

    if let Some(index) = app.selected {
        let work = &WORKS[index];
        let texture = &app.textures[index];
        let aspect = photograph.aspect_ratio() / texture.aspect_ratio();
        let zoom = zoom_factor(app.pult.zoom);
        let visible = vec2(aspect.min(1.0), aspect.recip().min(1.0)) / zoom;
        let uv = Rect::from_center_size(pos2(0.5, 0.5), visible);
        p.image(texture.id(), photograph, uv, Color32::WHITE);
        let response = click(
            ui,
            photograph,
            ("pult-image", index),
            "Открыть просмотр",
            true,
        );
        meter_corners(p, photograph, signal);
        if response.clicked() {
            app.open_preview();
        }

        let caption = pos2(photograph.left(), photograph.bottom() + 18.0);
        text(
            p,
            caption,
            Align2::LEFT_CENTER,
            work.title,
            FontId::proportional(17.0),
            SCREEN_TEXT,
        );
        text(
            p,
            pos2(photograph.right(), caption.y),
            Align2::RIGHT_CENTER,
            &format!("{} / {}", work.place, work.year),
            FontId::monospace(11.0),
            SCREEN_QUIET,
        );
    } else {
        meter_corners(p, photograph, signal);
        text(
            p,
            photograph.center(),
            Align2::CENTER_CENTER,
            "Снимки не найдены",
            FontId::proportional(16.0),
            SCREEN_QUIET,
        );
    }

    let tracks = Rect::from_min_max(pos2(divider + 1.0, screen.top()), screen.max);
    p.vline(
        divider,
        screen.y_range(),
        Stroke::new(1.0, Color32::from_rgb(64, 80, 69)),
    );
    let matches = app.matches();
    let heading = tracks.left_top() + vec2(13.0, 10.0);
    text(
        p,
        heading,
        Align2::LEFT_CENTER,
        "КАДРЫ",
        FontId::monospace(11.0),
        SCREEN_QUIET,
    );
    text(
        p,
        pos2(tracks.right() - 2.0, heading.y),
        Align2::RIGHT_CENTER,
        &format!("{:02} / {:02}", matches.len(), WORKS.len()),
        FontId::monospace(11.0),
        SCREEN_QUIET,
    );
    let row_area = Rect::from_min_max(tracks.min + vec2(12.0, 24.0), tracks.max);
    let density = (app.pult.density + if app.compact { 0.35 } else { 0.0 }).min(1.0);
    let row_max = ((row_area.height() - 1.0) / WORKS.len() as f32).clamp(16.0, 57.0);
    let row_h = row_max * (1.0 - 0.34 * density);
    for (slot, index) in matches.into_iter().enumerate() {
        let row = Rect::from_min_size(
            row_area.min + vec2(0.0, slot as f32 * row_h),
            vec2(row_area.width(), row_h),
        );
        let work = &WORKS[index];
        let response = click(ui, row, ("pult-track", index), work.title, true);
        let active = app.selected == Some(index);
        if active || response.hovered() || response.has_focus() {
            let fill = if active {
                mix(
                    Color32::from_rgb(39, 52, 43),
                    Color32::from_rgb(73, 86, 56),
                    app.pult.signal,
                )
            } else {
                Color32::from_rgb(33, 47, 38)
            };
            p.rect_filled(row.shrink2(vec2(0.0, 2.0)), 4, fill);
        }
        p.hline(
            row.x_range(),
            row.bottom(),
            Stroke::new(1.0, Color32::from_rgb(53, 68, 58)),
        );
        let center_y = row.center().y;
        let led = pos2(row.left() + 11.0, center_y);
        p.circle_filled(
            led,
            3.0,
            if active {
                signal
            } else {
                Color32::from_rgb(78, 102, 85)
            },
        );
        if active && app.pult.signal > 0.04 {
            p.circle_stroke(
                led,
                5.0,
                Stroke::new(1.0, signal.gamma_multiply(app.pult.signal * 0.32)),
            );
        }
        text(
            p,
            pos2(row.left() + 23.0, center_y),
            Align2::LEFT_CENTER,
            &format!("{:02}", index + 1),
            FontId::monospace(10.0),
            SCREEN_QUIET,
        );
        text(
            p,
            pos2(row.left() + 49.0, center_y),
            Align2::LEFT_CENTER,
            work.title,
            FontId::proportional(13.0),
            if active { signal } else { SCREEN_TEXT },
        );
        if app.marked[index] {
            p.rect_filled(
                Rect::from_center_size(pos2(row.right() - 9.0, center_y), vec2(5.0, 8.0)),
                1,
                ORANGE,
            );
        }
        if response.clicked() {
            app.select(index);
            if response.double_clicked() {
                app.open_preview();
            }
            ui.ctx().request_repaint();
        }
    }
}

fn meter_corners(p: &Painter, bounds: Rect, color: Color32) {
    let width = 12.0;
    let bounds = bounds.expand(1.0);
    for (pos, direction) in [
        (bounds.left_top(), vec2(1.0, 1.0)),
        (bounds.right_top(), vec2(-1.0, 1.0)),
        (bounds.left_bottom(), vec2(1.0, -1.0)),
        (bounds.right_bottom(), vec2(-1.0, -1.0)),
    ] {
        p.line_segment(
            [pos, pos + vec2(direction.x * width, 0.0)],
            Stroke::new(1.5, color),
        );
        p.line_segment(
            [pos, pos + vec2(0.0, direction.y * width)],
            Stroke::new(1.5, color),
        );
    }
}

fn controls(ui: &mut egui::Ui, p: &Painter, app: &mut App, plate: Rect) {
    p.rect_filled(plate, 13, STEEL);
    p.rect_stroke(
        plate,
        13,
        Stroke::new(1.0, Color32::from_rgb(174, 190, 173)),
        StrokeKind::Inside,
    );
    if app.overlay == Some(Overlay::Actions) {
        action_bank(ui, p, app, plate);
        return;
    }
    let bank_width = 171.0;
    let seam = plate.right() - bank_width - 22.0;
    let spacing = (seam - plate.left() - 23.0) / 3.0;
    let center = |index: usize| {
        pos2(
            plate.left() + 13.0 + spacing * (index as f32 + 0.5),
            plate.top() + 54.0,
        )
    };
    p.vline(
        seam,
        plate.top() + 15.0..=plate.bottom() - 15.0,
        Stroke::new(1.0, Color32::from_rgb(149, 169, 151)),
    );
    dial(
        ui,
        p,
        center(0),
        Dial {
            title: "Масштаб",
            tint: JADE,
            resting: ZOOM,
            readout: zoom_readout,
        },
        &mut app.pult.zoom,
    );
    dial(
        ui,
        p,
        center(1),
        Dial {
            title: "Плотность",
            tint: ORANGE,
            resting: DENSITY,
            readout: percent,
        },
        &mut app.pult.density,
    );
    dial(
        ui,
        p,
        center(2),
        Dial {
            title: "Сигнал",
            tint: CITRON,
            resting: SIGNAL,
            readout: percent,
        },
        &mut app.pult.signal,
    );
    text(
        p,
        pos2((plate.left() + seam) * 0.5, plate.bottom() - 9.0),
        Align2::CENTER_CENTER,
        "Повернуть: потянуть или прокрутить · клавиатура: Tab, ← →",
        FontId::proportional(10.0),
        QUIET,
    );

    let x = seam + 11.0;
    let y = plate.top() + 13.0;
    let preview = Rect::from_min_size(pos2(x, y), vec2(bank_width, 43.0));
    if lever(
        ui,
        p,
        preview,
        "Просмотр",
        "Enter",
        app.selected.is_some(),
        true,
    ) {
        app.open_preview();
    }
    let actions = Rect::from_min_size(pos2(x, y + 51.0), vec2(bank_width, 24.0));
    if lever(ui, p, actions, "Действия", "Ctrl+K", true, false) {
        if app.overlay == Some(Overlay::Actions) {
            app.close_overlay();
        } else {
            app.open_actions();
        }
        ui.ctx().request_repaint();
    }
    let settings = actions.translate(vec2(0.0, 30.0));
    if lever(ui, p, settings, "Настройки", "Ctrl+,", true, false) {
        app.open_settings();
    }
}

fn action_bank(ui: &mut egui::Ui, p: &Painter, app: &mut App, plate: Rect) {
    let actions = app.actions();
    let left = plate.left() + 16.0;
    let right = plate.right() - 16.0;
    let header_y = plate.top() + 17.0;
    let well = Rect::from_min_max(
        pos2(left, plate.top() + 32.0),
        pos2(right, plate.bottom() - 34.0),
    );
    app.action_index = app.action_index.min(actions.len().saturating_sub(1));
    if !actions.is_empty() {
        let scroll = ui.input(|input| {
            if !input
                .pointer
                .hover_pos()
                .is_some_and(|pos| well.contains(pos))
            {
                return 0.0;
            }
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::MouseWheel { delta, .. } => {
                        Some(if delta.x == 0.0 { delta.y } else { delta.x })
                    }
                    _ => None,
                })
                .sum::<f32>()
        });
        if scroll < 0.0 {
            app.action_index = (app.action_index + 1) % actions.len();
        } else if scroll > 0.0 {
            app.action_index = (app.action_index + actions.len() - 1) % actions.len();
        }
    }

    text(
        p,
        pos2(left, header_y),
        Align2::LEFT_CENTER,
        "ДЕЙСТВИЯ",
        FontId::monospace(11.0),
        INK,
    );
    if let Some(index) = app.selected {
        text(
            p,
            pos2(left + 96.0, header_y),
            Align2::LEFT_CENTER,
            WORKS[index].title,
            FontId::proportional(13.0),
            QUIET,
        );
    }
    text(
        p,
        pos2(right, header_y),
        Align2::RIGHT_CENTER,
        "← → или колесо",
        FontId::proportional(11.0),
        QUIET,
    );

    p.rect_filled(well, 8, Color32::from_rgb(100, 118, 105));
    p.rect_stroke(
        well,
        8,
        Stroke::new(1.0, Color32::from_rgb(76, 94, 82)),
        StrokeKind::Inside,
    );
    let gap = 8.0;
    let key_width =
        (well.width() - gap * (actions.len() as f32 + 1.0)) / actions.len().max(1) as f32;
    let mut chosen = None;
    for (slot, action) in actions.into_iter().enumerate() {
        let key = Rect::from_min_size(
            well.left_top() + vec2(gap + slot as f32 * (key_width + gap), 6.0),
            vec2(key_width, well.height() - 15.0),
        );
        let action_name = app.action_label(action);
        let label = match action {
            Action::Preview => "Просмотр",
            Action::Mark if app.selected.is_some_and(|index| app.marked[index]) => "Снять отметку",
            Action::Mark => "Отметить",
            Action::Undo => "Отменить отметку",
            Action::Settings => "Настройки",
        };
        let response =
            click(ui, key, ("pult-action", slot), action_name, true).on_hover_text(action_name);
        let selected = slot == app.action_index;
        let cap = if response.is_pointer_button_down_on() {
            key.translate(vec2(0.0, 3.0))
        } else {
            key
        };
        p.rect_filled(key.translate(vec2(0.0, 4.0)), 5, INK);
        let fill = if selected {
            Color32::from_rgb(165, 223, 177)
        } else if response.hovered() || response.has_focus() {
            Color32::from_rgb(228, 234, 216)
        } else {
            Color32::from_rgb(208, 218, 199)
        };
        p.rect_filled(cap, 5, fill);
        p.hline(
            cap.left() + 5.0..=cap.right() - 5.0,
            cap.top() + 2.0,
            Stroke::new(1.0, fill.gamma_multiply(1.14)),
        );
        if response.has_focus() {
            p.rect_stroke(
                cap.shrink(2.0),
                3,
                Stroke::new(1.0, QUIET),
                StrokeKind::Inside,
            );
        }
        p.circle_filled(
            pos2(cap.center().x, cap.top() + 11.0),
            2.7,
            if selected {
                INK
            } else {
                Color32::from_rgb(158, 177, 157)
            },
        );
        text(
            p,
            pos2(cap.center().x, cap.center().y + 6.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(13.0),
            INK,
        );
        if response.clicked() {
            chosen = Some(action);
        }
    }
    text(
        p,
        pos2(left, plate.bottom() - 17.0),
        Align2::LEFT_CENTER,
        "Enter · Выполнить",
        FontId::proportional(11.0),
        QUIET,
    );
    let close = Rect::from_min_size(
        pos2(right - 124.0, plate.bottom() - 30.0),
        vec2(124.0, 24.0),
    );
    if lever(ui, p, close, "Закрыть", "Esc", true, false) {
        app.close_overlay();
    }
    if let Some(action) = chosen {
        app.execute(ui.ctx(), action);
    }
}

struct Dial {
    title: &'static str,
    tint: Color32,
    resting: f32,
    readout: fn(f32) -> String,
}

fn dial(ui: &mut egui::Ui, p: &Painter, center: Pos2, spec: Dial, value: &mut f32) {
    let radius = 27.0;
    let interaction = Rect::from_center_size(center, Vec2::splat(2.0 * (radius + 7.0)));
    let mut response = ui.interact(
        interaction,
        ui.make_persistent_id(("pult-dial", spec.title)),
        Sense::click_and_drag(),
    );
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
    let drag = response.drag_delta();
    let wheel = if response.hovered() {
        ui.input(|input| input.smooth_scroll_delta().y)
    } else {
        0.0
    };
    let mut delta = (drag.x - drag.y) / 155.0 + wheel / 540.0;
    if response.has_focus() {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                response.id,
                EventFilter {
                    horizontal_arrows: true,
                    ..Default::default()
                },
            );
        });
        let arrow_delta = ui.input_mut(|input| {
            let increase = input.consume_key(Modifiers::NONE, Key::ArrowRight);
            let decrease = input.consume_key(Modifiers::NONE, Key::ArrowLeft);
            (i32::from(increase) - i32::from(decrease)) as f32 * 0.05
        });
        if arrow_delta != 0.0 {
            ui.memory_mut(|memory| memory.move_focus(FocusDirection::None));
        }
        delta += arrow_delta;
    }
    let previous = *value;
    if response.double_clicked() {
        *value = spec.resting;
    } else {
        *value = (*value + delta).clamp(0.0, 1.0);
    }
    if previous != *value {
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::slider(true, f64::from(*value), spec.title));
    if response.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    } else if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }

    let angle = |position: f32| (0.75 + 1.50 * position) * PI;
    let radial = |a: f32| vec2(a.cos(), a.sin());
    for i in 0..21 {
        let position = i as f32 / 20.0;
        let direction = radial(angle(position));
        let color = if position <= *value {
            mix(INK, spec.tint, 0.65)
        } else {
            QUIET
        };
        let length = if i % 5 == 0 { 6.0 } else { 3.0 };
        p.line_segment(
            [
                center + direction * (radius + 6.0),
                center + direction * (radius + 6.0 + length),
            ],
            Stroke::new(1.1, color),
        );
    }
    p.circle_filled(
        center + vec2(0.0, 3.0),
        radius + 2.0,
        Color32::from_rgb(138, 157, 143),
    );
    p.circle_filled(center, radius + 0.8, INK);
    p.circle_filled(center, radius - 2.8, Color32::from_rgb(56, 72, 63));
    for i in 0..26 {
        let direction = radial(i as f32 * (2.0 * PI / 26.0));
        p.line_segment(
            [
                center + direction * (radius - 4.0),
                center + direction * (radius - 0.5),
            ],
            Stroke::new(1.0, INK),
        );
    }
    p.circle_filled(
        center - vec2(0.0, 1.2),
        radius - 6.2,
        Color32::from_rgb(77, 95, 83),
    );
    p.circle_stroke(
        center - vec2(0.0, 1.2),
        radius - 6.3,
        Stroke::new(1.0, Color32::from_rgb(112, 131, 118)),
    );
    if response.hovered() || response.has_focus() {
        p.circle_stroke(center, radius + 2.0, Stroke::new(1.6, spec.tint));
    }
    let pointer = radial(angle(*value));
    p.line_segment(
        [center + pointer * 10.0, center + pointer * 22.0],
        Stroke::new(3.0, spec.tint),
    );
    text(
        p,
        center + vec2(0.0, 36.0),
        Align2::CENTER_CENTER,
        spec.title,
        FontId::proportional(12.0),
        INK,
    );
    text(
        p,
        center + vec2(0.0, 54.0),
        Align2::CENTER_CENTER,
        &(spec.readout)(*value),
        FontId::monospace(13.0),
        INK,
    );
}

fn zoom_factor(value: f32) -> f32 {
    1.0 + value * 1.5
}

fn zoom_readout(value: f32) -> String {
    format!("{:.0} %", zoom_factor(value) * 100.0)
}

fn percent(value: f32) -> String {
    format!("{:.0} %", value * 100.0)
}

fn lever(
    ui: &mut egui::Ui,
    p: &Painter,
    rect: Rect,
    name: &str,
    key: &str,
    enabled: bool,
    primary: bool,
) -> bool {
    let response = click(ui, rect, ("pult-lever", name), name, enabled);
    let lit = response.hovered() || response.has_focus();
    let pressed = response.is_pointer_button_down_on();
    let cap = if pressed {
        rect.translate(vec2(0.0, 2.0))
    } else {
        rect
    };
    if primary {
        let fill = if !enabled {
            Color32::from_rgb(141, 162, 145)
        } else if lit {
            Color32::from_rgb(179, 241, 193)
        } else {
            Color32::from_rgb(158, 216, 171)
        };
        p.rect_filled(rect.translate(vec2(0.0, 3.0)), 6, INK);
        p.rect_filled(cap, 6, fill);
        p.hline(
            cap.left() + 6.0..=cap.right() - 6.0,
            cap.top() + 2.0,
            Stroke::new(1.0, fill.gamma_multiply(1.14)),
        );
    } else if lit {
        p.rect_filled(cap, 4, Color32::from_rgb(216, 226, 203));
    } else {
        p.hline(
            cap.x_range(),
            cap.bottom(),
            Stroke::new(1.0, Color32::from_rgb(155, 176, 156)),
        );
    }
    if response.has_focus() {
        p.rect_stroke(cap, 5, Stroke::new(1.0, INK), StrokeKind::Inside);
    }
    let name_at = pos2(cap.left() + 11.0, cap.center().y);
    text(
        p,
        name_at,
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(13.0),
        INK,
    );
    text(
        p,
        pos2(cap.right() - 10.0, cap.center().y),
        Align2::RIGHT_CENTER,
        key,
        FontId::monospace(10.0),
        QUIET,
    );
    response.clicked()
}

fn click(
    ui: &mut egui::Ui,
    rect: Rect,
    id: impl egui::AsIdSalt,
    name: &str,
    enabled: bool,
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = ui.interact(rect, ui.make_persistent_id(id), sense);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, name));
    if enabled {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response
    }
}

fn mix(from: Color32, to: Color32, blend: f32) -> Color32 {
    let channel = |a: u8, b: u8| egui::lerp(f32::from(a)..=f32::from(b), blend).round() as u8;
    Color32::from_rgb(
        channel(from.r(), to.r()),
        channel(from.g(), to.g()),
        channel(from.b(), to.b()),
    )
}

fn text(p: &Painter, pos: Pos2, align: Align2, label: &str, font: FontId, color: Color32) {
    p.text(pos, align, label, font, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Edition;
    use egui::{Event, Id, MouseWheelUnit, PointerButton, RawInput, TouchPhase};

    #[test]
    fn zoom_and_action_keys_work_at_minimum_size() {
        let window = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 570.0));
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, Edition::Pult);
        let mut frame = |events: Vec<Event>| {
            let mut ids = [Id::NULL; 7];
            let _ = ctx.run_ui(
                RawInput {
                    screen_rect: Some(window),
                    events,
                    ..Default::default()
                },
                |ui| {
                    ids = [
                        ui.make_persistent_id(("pult-dial", "Масштаб")),
                        ui.make_persistent_id(("pult-dial", "Плотность")),
                        ui.make_persistent_id(("pult-dial", "Сигнал")),
                        ui.make_persistent_id(("pult-image", 0_usize)),
                        ui.make_persistent_id(("pult-action", 0_usize)),
                        ui.make_persistent_id(("pult-action", 1_usize)),
                        ui.make_persistent_id(("pult-action", 2_usize)),
                    ];
                    app.show(ui);
                },
            );
            (ids, app.pult.zoom, app.action_index, app.marked[0])
        };

        let (ids, resting, _, _) = frame(vec![]);
        let center = ctx.read_response(ids[0]).unwrap().rect.center();
        let initial_photo = ctx.read_response(ids[3]).unwrap().rect;
        for dial_id in &ids[..3] {
            let bounds = ctx.read_response(*dial_id).unwrap().rect;
            assert!(window.contains_rect(bounds));
            assert!(initial_photo.bottom() < bounds.top());
        }
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        frame(vec![Event::PointerMoved(center), button(center, true)]);
        let dragged = center - vec2(0.0, 38.0);
        let (_, after_drag, _, _) = frame(vec![Event::PointerMoved(dragged)]);
        assert!(after_drag > resting);
        frame(vec![button(dragged, false), Event::PointerMoved(center)]);

        let (_, after_wheel, _, _) = frame(vec![Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: vec2(0.0, 80.0),
            phase: TouchPhase::Move,
            modifiers: Modifiers::NONE,
        }]);
        assert!(after_wheel > after_drag);

        frame(vec![button(center, true)]);
        frame(vec![button(center, false)]);
        let left = |pressed| Event::Key {
            key: Key::ArrowLeft,
            physical_key: Some(Key::ArrowLeft),
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let (_, before_key, _, _) = frame(vec![Event::PointerGone]);
        let (_, after_key, _, _) = frame(vec![left(true), left(false)]);
        assert!(after_key < before_key);
        assert!(ctx.memory(|memory| memory.has_focus(ids[0])));

        let ctrl_k = |pressed| Event::Key {
            key: Key::K,
            physical_key: Some(Key::K),
            pressed,
            repeat: false,
            modifiers: Modifiers::CTRL,
        };
        frame(vec![ctrl_k(true), ctrl_k(false)]);
        let photo = ctx.read_response(ids[3]).unwrap().rect;
        assert_eq!(photo, initial_photo);
        let keys = ids[4..]
            .iter()
            .map(|id| ctx.read_response(*id).unwrap().rect)
            .collect::<Vec<_>>();
        for bounds in &keys {
            assert!(window.contains_rect(*bounds));
            assert!(photo.bottom() < bounds.top());
            assert_eq!(bounds.top(), keys[0].top());
        }
        assert!(keys.windows(2).all(|pair| pair[0].right() < pair[1].left()));
        let (_, _, choice, _) = frame(vec![
            Event::PointerMoved(keys[0].center()),
            Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta: vec2(0.0, -80.0),
                phase: TouchPhase::Move,
                modifiers: Modifiers::NONE,
            },
        ]);
        assert_eq!(choice, 1);
        let at = keys[1].center();
        frame(vec![Event::PointerMoved(at), button(at, true)]);
        let (_, _, _, marked) = frame(vec![button(at, false)]);
        assert!(marked);
    }
}
