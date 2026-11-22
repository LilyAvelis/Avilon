use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Painter, PointerButton, Pos2, Rect, Sense, Shape,
    Stroke, UiBuilder, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{Action, App, Overlay, WORKS, search_id};

const VOID: Color32 = Color32::from_rgb(18, 16, 28);
const PANEL: Color32 = Color32::from_rgb(32, 24, 46);
const TEXT: Color32 = Color32::from_rgb(237, 230, 255);
const MUTED: Color32 = Color32::from_rgb(170, 159, 192);
const ACCENT: Color32 = Color32::from_rgb(205, 180, 255);
const FAINT: Color32 = Color32::from_rgb(79, 64, 100);
const WHEEL_RADIUS: f32 = 119.0;
const WHEEL_HOLE: f32 = 25.0;
const CENTER_UV: Rect = Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0));

#[derive(Default)]
pub(super) struct State {
    origin: Option<Pos2>,
    centered: bool,
    sector: usize,
}

impl State {
    pub(super) fn radial_open(&self) -> bool {
        self.centered || self.origin.is_some()
    }

    pub(super) fn close_radial(&mut self) {
        self.origin = None;
        self.centered = false;
    }

    pub(super) fn open_center(&mut self, has_frame: bool) {
        self.centered = true;
        self.sector = if has_frame { 0 } else { 2 };
    }

    pub(super) fn step(&mut self, offset: isize, has_frame: bool, can_undo: bool) {
        let enabled = [has_frame, has_frame, true, can_undo];
        if offset == 0 && enabled[self.sector] {
            return;
        }
        let sectors: Vec<usize> = (0..4).filter(|&i| enabled[i]).collect();
        let current = sectors
            .iter()
            .position(|&index| index == self.sector)
            .unwrap_or(0);
        let next = (current as isize + offset).rem_euclid(sectors.len() as isize) as usize;
        self.sector = sectors[next];
    }

    pub(super) fn action(&self) -> Action {
        action_for(self.sector)
    }
}

pub(super) fn show(ui: &mut egui::Ui, app: &mut App) {
    let bounds = ui.available_rect_before_wrap();
    let painter = ui.painter_at(bounds);
    let left = bounds.left() + 31.0;
    let right = bounds.right() - 31.0;
    let top = bounds.top();
    let bottom = bounds.bottom();

    if matches!(app.overlay, Some(Overlay::Preview | Overlay::Settings)) {
        app.orbit.close_radial();
    }
    let radial_open = app.orbit.radial_open();
    let field = Rect::from_min_max(pos2(left + 196.0, top + 59.0), pos2(right, bottom - 73.0));
    let footer_y = bottom - 37.0;

    painter.rect_filled(bounds, 0, VOID);
    if let Some(index) = app.selected {
        reflection(&painter, app, index, bounds);
    }
    add_header(ui, &painter, app, left, right, top, radial_open);
    if let Some(index) = app.selected {
        let photo = picture_rect(field);
        painter.rect_filled(
            photo.translate(vec2(0.0, 6.0)).expand(1.0),
            0,
            Color32::from_black_alpha(72),
        );
        painter.image(app.textures[index].id(), photo, CENTER_UV, Color32::WHITE);
        draw_caption(&painter, app, index, left, photo.bottom());
        let photo_response = ui.interact(
            photo,
            ui.make_persistent_id("orbit-current-photo"),
            Sense::click(),
        );
        photo_response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, WORKS[index].title));
        if photo_response.secondary_clicked() {
            app.close_overlay();
            app.orbit.origin = ui.input(|input| input.pointer.interact_pos());
            app.orbit.sector = 0;
        } else if !radial_open && photo_response.clicked() {
            app.open_preview();
        }
    } else {
        text(
            &painter,
            field.center(),
            Align2::CENTER_CENTER,
            "Кадры не найдены",
            18.0,
            MUTED,
        );
    }

    add_navigation(ui, &painter, app, bounds, footer_y, radial_open);
    if let Some(origin) = app
        .orbit
        .origin
        .or_else(|| app.orbit.centered.then(|| bounds.center()))
    {
        show_radial(ui.ctx(), &painter, app, origin, bounds);
    }
}

fn picture_rect(field: Rect) -> Rect {
    let size = field.width().min(field.height());
    Rect::from_center_size(field.center(), Vec2::splat(size))
}

fn reflection(painter: &Painter, app: &App, index: usize, bounds: Rect) {
    let texture = &app.textures[index];
    let view = bounds.aspect_ratio();
    let uv = Rect::from_center_size(pos2(0.5, 0.5), vec2(view.min(1.0), view.recip().min(1.0)));
    // A quiet reflection of the same frame keeps the empty space tied to the image.
    painter.image(texture.id(), bounds, uv, Color32::from_white_alpha(28));
}

fn add_header(
    ui: &mut egui::Ui,
    painter: &Painter,
    app: &mut App,
    left: f32,
    right: f32,
    top: f32,
    radial_open: bool,
) {
    let mark = pos2(left + 12.0, top + 33.0);
    let slant = |angle: f32| {
        let direction = egui::Vec2::angled(angle);
        mark + vec2(
            direction.x * 13.0 + direction.y * 5.0,
            direction.y * 6.0 - direction.x * 4.0,
        )
    };
    let ellipse = (0..=30)
        .map(|step| slant(step as f32 / 30.0 * TAU))
        .collect();
    painter.add(Shape::line(ellipse, Stroke::new(1.1, ACCENT)));
    painter.circle_filled(mark, 2.9, ACCENT);
    painter.circle_filled(slant(-0.7), 2.2, TEXT);
    text(
        painter,
        mark + vec2(24.0, 0.0),
        Align2::LEFT_CENTER,
        "Орбита",
        24.0,
        TEXT,
    );
    text(
        painter,
        pos2(left + 163.0, mark.y),
        Align2::LEFT_CENTER,
        "После дождя",
        13.0,
        MUTED,
    );

    let search = Rect::from_min_size(pos2(right - 208.0, top + 17.0), vec2(208.0, 30.0));
    let lens = search.left_center() + vec2(8.0, -2.0);
    painter.circle_stroke(lens, 4.5, Stroke::new(1.0, ACCENT));
    painter.line_segment(
        [lens + vec2(3.5, 3.5), lens + vec2(7.5, 7.5)],
        Stroke::new(1.0, ACCENT),
    );
    let input = Rect::from_min_max(search.min + vec2(23.0, 4.0), search.max);
    let response = ui
        .scope_builder(UiBuilder::new().max_rect(input), |ui| {
            if radial_open {
                ui.disable();
            }
            let style = ui.visuals_mut();
            style.text_edit_bg_color = Some(Color32::TRANSPARENT);
            style.weak_text_color = Some(MUTED);
            style.widgets.inactive.bg_stroke = Stroke::NONE;
            style.widgets.hovered.bg_stroke = Stroke::NONE;
            style.widgets.active.bg_stroke = Stroke::NONE;
            app.search(ui, input.width() - 8.0)
        })
        .inner;
    painter.hline(
        search.x_range(),
        search.bottom(),
        Stroke::new(1.0, if response.has_focus() { ACCENT } else { FAINT }),
    );
}

fn draw_caption(painter: &Painter, app: &App, index: usize, left: f32, bottom: f32) {
    let work = &WORKS[index];
    let caption_width = 179.0;
    let base_size = if app.compact { 23.0 } else { 29.0 };
    let longest_word = work.title.split_whitespace().fold(1.0_f32, |width, word| {
        let galley = painter.layout_no_wrap(word.to_owned(), FontId::proportional(base_size), TEXT);
        width.max(galley.size().x)
    });
    let size = base_size * ((caption_width - 2.0) / longest_word).min(1.0);
    let title = painter.layout(
        work.title.to_owned(),
        FontId::proportional(size),
        TEXT,
        caption_width,
    );
    let rest_y = bottom - title.size().y - if app.compact { 31.0 } else { 53.0 };
    let eyebrow = format!("{}  /  {}", work.place, work.year);
    text(
        painter,
        pos2(left, rest_y),
        Align2::LEFT_TOP,
        &eyebrow,
        12.0,
        ACCENT,
    );
    let pos = pos2(left, rest_y + 23.0);
    let title_bottom = pos.y + title.size().y;
    painter.galley(pos, title, TEXT);
    if !app.compact {
        text(
            painter,
            pos2(left, title_bottom + 7.0),
            Align2::LEFT_TOP,
            work.category,
            12.0,
            MUTED,
        );
    }
    if app.marked[index] {
        painter.circle_filled(pos2(left + caption_width - 11.0, rest_y + 7.0), 3.5, ACCENT);
    }
}

fn add_navigation(
    ui: &mut egui::Ui,
    painter: &Painter,
    app: &mut App,
    bounds: Rect,
    y: f32,
    radial_open: bool,
) {
    let left = bounds.left() + 31.0;
    let right = bounds.right() - 31.0;
    if app.selected.is_some() {
        text(
            painter,
            pos2(left, y - 3.0),
            Align2::LEFT_CENTER,
            "ПКМ по кадру — круговое меню",
            12.0,
            MUTED,
        );
    }
    let visible = app.matches();
    if !visible.is_empty() {
        let width = (visible.len().saturating_sub(1) as f32 * 25.0).max(2.0);
        let middle = bounds.center().x;
        let x0 = middle - width * 0.5;
        let x1 = middle + width * 0.5;
        painter.hline(x0..=x1, y - 5.0, Stroke::new(1.0, FAINT));
        for (slot, index) in visible.iter().copied().enumerate() {
            let center = if visible.len() == 1 {
                middle
            } else {
                x0 + slot as f32 * 25.0
            };
            let center = pos2(center, y - 5.0);
            let target = Rect::from_center_size(center, vec2(23.0, 27.0));
            let response = ui
                .interact(
                    target,
                    ui.make_persistent_id(("orbit-dot", index)),
                    Sense::click(),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            response
                .widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, WORKS[index].title));
            let active = app.selected == Some(index);
            let color = if active || response.hovered() || response.has_focus() {
                ACCENT
            } else {
                MUTED
            };
            painter.circle_filled(center, if active { 4.1 } else { 2.8 }, color);
            if active {
                painter.circle_stroke(center, 7.4, Stroke::new(1.0, ACCENT));
            }
            if !radial_open && response.clicked() {
                app.select(index);
            }
        }
        let current = visible
            .iter()
            .position(|&index| app.selected == Some(index))
            .map_or(0, |i| i + 1);
        let arrows = if !app.query.is_empty() && ui.memory(|memory| memory.has_focus(search_id())) {
            "↑ ↓"
        } else {
            "← →"
        };
        let hint = format!("{arrows}   {current} / {}", visible.len());
        text(
            painter,
            pos2(middle, y + 15.0),
            Align2::CENTER_CENTER,
            &hint,
            11.0,
            MUTED,
        );
    }

    if flat_button(ui, painter, right - 202.0, y, "Действия", "Ctrl+K") && !radial_open {
        app.open_actions();
    }
    if flat_button(ui, painter, right - 82.0, y, "Настройки", "Ctrl+,") && !radial_open {
        app.open_settings();
    }
}

fn flat_button(
    ui: &mut egui::Ui,
    painter: &Painter,
    x: f32,
    y: f32,
    label: &str,
    shortcut: &str,
) -> bool {
    let hit = Rect::from_min_size(pos2(x - 4.0, y - 16.0), vec2(87.0, 47.0));
    let response = ui
        .interact(
            hit,
            ui.make_persistent_id(("orbit-control", label)),
            Sense::click(),
        )
        .on_hover_cursor(CursorIcon::PointingHand);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    let active = response.hovered() || response.has_focus();
    text(
        painter,
        pos2(x, y - 12.0),
        Align2::LEFT_TOP,
        label,
        15.0,
        if active { ACCENT } else { TEXT },
    );
    text(
        painter,
        pos2(x, y + 9.0),
        Align2::LEFT_TOP,
        shortcut,
        11.0,
        MUTED,
    );
    if active {
        painter.hline(x..=x + 69.0, y + 6.0, Stroke::new(1.0, ACCENT));
    }
    response.clicked()
}

fn action_for(sector: usize) -> Action {
    match sector {
        0 => Action::Preview,
        1 => Action::Mark,
        2 => Action::Settings,
        _ => Action::Undo,
    }
}

fn caption_for(app: &App, sector: usize) -> &'static str {
    match sector {
        0 => "Просмотр",
        1 if app.selected.is_some_and(|index| app.marked[index]) => "Снять",
        1 => "Отметить",
        2 => "Настройки",
        _ => "Отменить",
    }
}

fn sector_at(center: Pos2, cursor: Pos2) -> Option<usize> {
    let delta = cursor - center;
    if !(WHEEL_HOLE..=WHEEL_RADIUS).contains(&delta.length()) {
        return None;
    }
    let angle = (delta.angle() + FRAC_PI_2 + FRAC_PI_4).rem_euclid(TAU);
    Some((angle / FRAC_PI_2) as usize)
}

fn show_radial(ctx: &egui::Context, painter: &Painter, app: &mut App, origin: Pos2, bounds: Rect) {
    let inset = bounds.shrink(WHEEL_RADIUS + 7.0);
    let center = origin.clamp(inset.min, inset.max);
    let (cursor, clicked, moved) = ctx.input(|input| {
        (
            input.pointer.interact_pos(),
            input.pointer.button_clicked(PointerButton::Primary),
            input.pointer.delta() != Vec2::ZERO,
        )
    });
    let sector = cursor.and_then(|cursor| sector_at(center, cursor));
    let enabled = [
        app.selected.is_some(),
        app.selected.is_some(),
        true,
        !app.history.is_empty(),
    ];
    let hovered = sector.filter(|&index| enabled[index]);
    if moved && let Some(index) = hovered {
        app.orbit.sector = index;
    }
    painter.circle_filled(
        center,
        WHEEL_RADIUS,
        Color32::from_rgba_unmultiplied(15, 12, 24, 213),
    );
    for (i, available) in enabled.into_iter().enumerate() {
        let angle = (i as f32 - 1.0) * FRAC_PI_2;
        let active = app.orbit.sector == i && available;
        let fill = if active { ACCENT } else { PANEL };
        draw_sector(painter, center, angle, fill);
        let position = center + Vec2::angled(angle) * 68.0;
        let label = caption_for(app, i);
        let color = if active {
            VOID
        } else if !available {
            MUTED
        } else {
            TEXT
        };
        text(painter, position, Align2::CENTER_CENTER, label, 12.0, color);
    }
    let on_close = cursor.is_some_and(|cursor| center.distance(cursor) < WHEEL_HOLE);
    painter.circle_filled(
        center,
        WHEEL_HOLE - 1.0,
        if on_close { ACCENT } else { VOID },
    );
    text(
        painter,
        center,
        Align2::CENTER_CENTER,
        "×",
        18.0,
        if on_close { VOID } else { ACCENT },
    );
    if app.orbit.centered {
        let hint =
            Rect::from_center_size(center + vec2(0.0, WHEEL_RADIUS + 20.0), vec2(240.0, 25.0));
        painter.rect_filled(hint, 12.0, VOID);
        text(
            painter,
            hint.center(),
            Align2::CENTER_CENTER,
            "← →  Выбор   ·   Enter  Действие   ·   Esc",
            11.0,
            TEXT,
        );
    }
    if clicked {
        if let Some(i) = hovered {
            app.execute(ctx, action_for(i));
        } else if sector.is_none() {
            app.close_overlay();
        }
    }
}

fn draw_sector(painter: &Painter, center: Pos2, angle: f32, fill: Color32) {
    let mut mesh = egui::epaint::Mesh::default();
    const STEPS: u32 = 18;
    let half_angle = FRAC_PI_4 - 0.027;
    for i in 0..=STEPS {
        let angle = angle - half_angle + i as f32 / STEPS as f32 * half_angle * 2.0;
        let direction = Vec2::angled(angle);
        mesh.colored_vertex(center + direction * (WHEEL_HOLE + 2.0), fill);
        mesh.colored_vertex(center + direction * (WHEEL_RADIUS - 3.0), fill);
        if i < STEPS {
            let j = i * 2;
            mesh.add_triangle(j, j + 1, j + 3);
            mesh.add_triangle(j, j + 3, j + 2);
        }
    }
    painter.add(Shape::mesh(mesh));
}

fn text(painter: &Painter, pos: Pos2, anchor: Align2, label: &str, size: f32, color: Color32) {
    painter.text(pos, anchor, label, FontId::proportional(size), color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Edition;

    const MIN_SIZE: Vec2 = vec2(800.0, 570.0);

    fn draw(ctx: &egui::Context, app: &mut App, events: Vec<egui::Event>) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, MIN_SIZE)),
            events,
            ..Default::default()
        };
        ctx.run_ui(input, |ui| app.show(ui)).textures_delta.clear();
    }

    fn click(ctx: &egui::Context, app: &mut App, pos: Pos2, button: PointerButton) {
        for pressed in [true, false] {
            draw(
                ctx,
                app,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }

    #[test]
    fn right_click_near_edge_marks_the_work_and_escape_dismisses_the_wheel() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, Edition::Orbit);
        let origin = pos2(700.0, 476.0);
        draw(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, origin, PointerButton::Secondary);
        assert_eq!(app.orbit.origin, Some(origin));
        // The minimum-size viewport shifts the wheel left and up, so the target
        // lies east of the visible wheel center, not east of the click origin.
        click(&ctx, &mut app, pos2(742.0, 444.0), PointerButton::Primary);
        assert!(app.marked[0]);
        assert!(!app.orbit.radial_open());

        click(&ctx, &mut app, origin, PointerButton::Secondary);
        draw(
            &ctx,
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: Some(egui::Key::Escape),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(!app.orbit.radial_open());
        assert!(app.marked[0]);
    }
}
