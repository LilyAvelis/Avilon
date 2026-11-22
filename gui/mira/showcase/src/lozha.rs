use std::f32::consts::{PI, TAU};

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontFamily, FontId, Painter, Pos2, Rect, Sense, Shape,
    Stroke, StrokeKind, UiBuilder, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{Action, App, Overlay, SERIF, WORKS};

#[derive(Clone, Copy)]
struct Palette {
    wall: Color32,
    shadow: Color32,
    seat: Color32,
    raised: Color32,
    ink: Color32,
    quiet: Color32,
    gold: Color32,
    fine: Color32,
    flare: Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

const NIGHT: Palette = Palette {
    wall: rgb(37, 25, 32),
    shadow: rgb(19, 14, 21),
    seat: rgb(44, 29, 37),
    raised: rgb(89, 55, 53),
    ink: rgb(250, 233, 204),
    quiet: rgb(190, 160, 143),
    gold: rgb(237, 188, 112),
    fine: rgb(116, 79, 63),
    flare: rgb(188, 108, 61),
};

const DAY: Palette = Palette {
    wall: rgb(246, 234, 208),
    shadow: rgb(205, 177, 143),
    seat: rgb(236, 216, 184),
    raised: rgb(234, 195, 143),
    ink: rgb(55, 35, 31),
    quiet: rgb(106, 76, 59),
    gold: rgb(135, 77, 37),
    fine: rgb(171, 131, 95),
    flare: rgb(255, 247, 212),
};

pub(super) fn show(ui: &mut egui::Ui, app: &mut App) {
    let window = ui.available_rect_before_wrap();
    let p = ui.painter_at(window);
    let palette = if app.lozha_light { DAY } else { NIGHT };
    let inset = (window.width() * 0.032).clamp(22.0, 40.0);
    let origin = window.min + vec2(inset, 0.0);
    let right = window.right() - inset;
    let side = ((right - origin.x) * 0.286).clamp(219.0, 320.0);
    let split = right - side;
    let stage = Rect::from_min_max(
        origin + vec2(0.0, 74.0),
        pos2(split - 22.0, window.bottom() - 114.0),
    );
    let rail = Rect::from_min_max(
        pos2(split, stage.top()),
        pos2(right, window.bottom() - 28.0),
    );
    let footer = Rect::from_min_max(
        pos2(stage.left(), stage.bottom() + 16.0),
        pos2(stage.right(), rail.bottom()),
    );

    gradient(&p, window, palette.wall, palette.shadow);
    header(ui, app, &p, origin, right, palette);
    stage_show(ui, app, &p, stage, palette);
    side_show(ui, app, &p, rail, palette);
    strip(ui, app, &p, footer, palette);
}

fn header(ui: &mut egui::Ui, app: &mut App, p: &Painter, left: Pos2, right: f32, c: Palette) {
    let center = left + vec2(15.0, 36.0);
    p.circle_stroke(center, 12.0, Stroke::new(1.0, c.fine));
    p.add(Shape::line(
        diamond(center, 7.0, 10.0),
        Stroke::new(1.0, c.gold),
    ));
    p.circle_filled(center, 2.0, c.gold);
    p.text(
        center + vec2(24.0, 0.0),
        Align2::LEFT_CENTER,
        "Ложа",
        serif(33.0),
        c.ink,
    );
    p.text(
        left + vec2(143.0, 39.0),
        Align2::LEFT_CENTER,
        "После дождя",
        serif(16.0),
        c.quiet,
    );
    let day = app.lozha_light;
    for (index, (label, light)) in [("Ночь", false), ("День", true)].into_iter().enumerate()
    {
        let rect = Rect::from_min_size(
            pos2(right - 112.0 + index as f32 * 56.0, left.y + 22.0),
            vec2(52.0, 30.0),
        );
        let hit = hit(ui, rect, ("lighting", index), label);
        let highlighted = day == light || hit.hovered() || hit.has_focus();
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(13.0),
            if highlighted { c.ink } else { c.quiet },
        );
        if day == light {
            p.hline(
                rect.left() + 9.0..=rect.right() - 9.0,
                rect.bottom() - 2.0,
                Stroke::new(1.0, c.gold),
            );
        }
        if hit.clicked() && day != light {
            app.lozha_light = light;
            ui.ctx().set_visuals(if light {
                egui::Visuals::light()
            } else {
                egui::Visuals::dark()
            });
        }
    }
    let search = Rect::from_min_size(pos2(right - 349.0, left.y + 26.0), vec2(217.0, 27.0));
    ui.scope_builder(UiBuilder::new().max_rect(search), |ui| {
        let visuals = ui.visuals_mut();
        visuals.text_edit_bg_color = Some(Color32::TRANSPARENT);
        visuals.weak_text_color = Some(c.quiet);
        visuals.override_text_color = Some(c.ink);
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        visuals.widgets.active.bg_stroke = Stroke::NONE;
        let response = app.search(ui, search.width() - 10.0);
        let color = if response.has_focus() { c.gold } else { c.fine };
        p.hline(search.x_range(), search.bottom(), Stroke::new(1.0, color));
    });
    p.hline(left.x..=right, left.y + 63.0, Stroke::new(1.0, c.fine));
}

fn stage_show(ui: &mut egui::Ui, app: &mut App, p: &Painter, room: Rect, c: Palette) {
    let p = p.with_clip_rect(room);
    gradient(&p, room, c.shadow, c.seat);
    let size = (room.height() - 60.0).min(room.width() - 66.0);
    let image = Rect::from_center_size(room.center() - vec2(0.0, 5.0), Vec2::splat(size));
    halo(
        &p,
        pos2(image.center().x, image.top() + 17.0),
        vec2(room.width() * 0.63, room.height() * 0.89),
        c.flare,
    );

    for (offset, weight) in [(21.0, 1.0), (13.0, 1.5), (6.0, 1.0)] {
        let rect = image.expand(offset);
        p.add(Shape::closed_line(
            arch(rect),
            Stroke::new(weight, if weight > 1.0 { c.gold } else { c.fine }),
        ));
    }
    for direction in [-1.0, 1.0] {
        let x = image.center().x + direction * (image.width() * 0.5 + 31.0);
        let a = pos2(x, image.top() + image.height() * 0.22);
        let b = pos2(x, image.bottom() + 21.0);
        p.line_segment([a, b], Stroke::new(1.0, c.fine));
        p.add(Shape::line(diamond(a, 4.0, 9.0), Stroke::new(1.0, c.gold)));
        p.circle_filled(b, 2.0, c.gold);
    }
    if let Some(index) = app.selected {
        draw_arch_image(&p, image, &app.textures[index]);
        let response = hit(ui, image, "lozha-stage", WORKS[index].title)
            .on_hover_text("Открыть снимок целиком");
        if response.clicked() {
            app.open_preview();
        }
    } else {
        p.add(Shape::convex_polygon(arch(image), c.shadow, Stroke::NONE));
        p.text(
            image.center(),
            Align2::CENTER_CENTER,
            "Кадры не найдены",
            serif(21.0),
            c.quiet,
        );
    }
    p.rect_stroke(room, 0.0, Stroke::new(1.0, c.fine), StrokeKind::Inside);
}

fn side_show(ui: &mut egui::Ui, app: &mut App, p: &Painter, rail: Rect, c: Palette) {
    gradient(p, rail, c.seat, c.wall);
    p.rect_stroke(rail, 0.0, Stroke::new(1.0, c.fine), StrokeKind::Inside);
    let x = rail.left() + 18.0;
    let width = rail.width() - 36.0;
    let selected = app.selected;
    let divider_y = rail.top() + 192.0;
    if let Some(index) = selected {
        let work = &WORKS[index];
        p.text(
            pos2(x, rail.top() + 30.0),
            Align2::LEFT_CENTER,
            format!("{}  /  {}", work.place, work.year),
            FontId::monospace(11.0),
            c.gold,
        );
        let galley = p.layout(
            work.title.to_owned(),
            serif(if width < 205.0 { 24.0 } else { 30.0 }),
            c.ink,
            width,
        );
        let y = rail.top() + 50.0;
        let last = y + galley.size().y;
        p.galley(pos2(x, y), galley, c.ink);
        let desc = p.layout(
            work.description.to_owned(),
            FontId::proportional(13.0),
            c.quiet,
            width,
        );
        p.galley(pos2(x, last + 11.0), desc, c.quiet);
        if app.marked[index] {
            p.add(Shape::convex_polygon(
                diamond(pos2(rail.right() - 19.0, rail.top() + 29.0), 4.0, 6.0),
                c.gold,
                Stroke::NONE,
            ));
        }
    } else {
        p.text(
            pos2(x, rail.top() + 32.0),
            Align2::LEFT_CENTER,
            "После дождя",
            serif(20.0),
            c.ink,
        );
    }
    p.hline(x..=x + width, divider_y, Stroke::new(1.0, c.fine));
    let actions_open = app.overlay == Some(Overlay::Actions);
    let top = divider_y + 17.0;

    if actions_open {
        p.text(
            pos2(x + 6.0, top + 8.0),
            Align2::LEFT_CENTER,
            "Действия",
            serif(18.0),
            c.gold,
        );
        for (index, action) in app.actions().into_iter().enumerate() {
            let label = match action {
                Action::Preview => "Открыть кадр",
                Action::Mark if selected.is_some_and(|i| app.marked[i]) => "Снять отметку",
                Action::Mark => "Отметить кадр",
                Action::Settings => "Настройки",
                Action::Undo => "Отменить отметку",
            };
            let rect =
                Rect::from_min_size(pos2(x, top + 28.0 + index as f32 * 43.0), vec2(width, 38.0));
            if facet(
                ui,
                rect,
                ("lozha-action", index),
                label,
                if index == app.action_index {
                    "Enter"
                } else {
                    ""
                },
                index == app.action_index,
                c,
            ) {
                app.execute(ui.ctx(), action);
            }
        }
        let rect = Rect::from_min_size(pos2(x, rail.bottom() - 39.0), vec2(width, 27.0));
        if facet(ui, rect, "lozha-close", "Закрыть", "Esc", false, c) {
            app.close_overlay();
        }
    } else {
        let controls = [
            ("Открыть кадр", "Enter"),
            ("Действия", "Ctrl+K"),
            ("Настройки", "Ctrl+,"),
        ];
        for (index, (label, key)) in controls.into_iter().enumerate() {
            if index == 0 && selected.is_none() {
                continue;
            }
            let rect = Rect::from_min_size(pos2(x, top + index as f32 * 49.0), vec2(width, 40.0));
            if facet(ui, rect, ("lozha-control", index), label, key, false, c) {
                match index {
                    0 => app.open_preview(),
                    1 => app.open_actions(),
                    _ => app.open_settings(),
                }
            }
        }
    }
}

fn strip(ui: &mut egui::Ui, app: &mut App, p: &Painter, strip: Rect, c: Palette) {
    let items = app.matches();
    if items.is_empty() {
        return;
    }
    let size = if app.compact { 40.0 } else { 55.0 };
    let step = size + 14.0;
    let start = strip.center().x - (items.len() - 1) as f32 * step * 0.5;
    let y = strip.top() + size * 0.5 + 5.0;
    p.hline(strip.x_range(), y, Stroke::new(1.0, c.fine));
    for (slot, index) in items.into_iter().enumerate() {
        let at = pos2(start + step * slot as f32, y);
        let card = Rect::from_center_size(at, Vec2::splat(size));
        let response = hit(
            ui,
            card.expand(2.0),
            ("lozha-frame", index),
            WORKS[index].title,
        )
        .on_hover_text(WORKS[index].title);
        let active = app.selected == Some(index);
        p.image(
            app.textures[index].id(),
            card,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            if active {
                Color32::WHITE
            } else {
                rgb(203, 186, 168)
            },
        );
        p.rect_stroke(
            card.expand(2.0),
            0.0,
            Stroke::new(
                if active { 2.0 } else { 1.0 },
                if active || response.hovered() || response.has_focus() {
                    c.gold
                } else {
                    c.fine
                },
            ),
            StrokeKind::Outside,
        );
        if active {
            p.add(Shape::convex_polygon(
                diamond(at + vec2(0.0, size * 0.5 + 10.0), 3.0, 5.0),
                c.gold,
                Stroke::NONE,
            ));
        }
        if response.clicked() {
            app.select(index);
            if response.double_clicked() {
                app.open_preview();
            }
        }
    }
}

fn facet(
    ui: &mut egui::Ui,
    rect: Rect,
    id: impl egui::AsIdSalt,
    label: &str,
    key: &str,
    active: bool,
    c: Palette,
) -> bool {
    let r = hit(ui, rect, id, label);
    let p = ui.painter();
    let emphasized = r.hovered() || r.has_focus() || active;
    let cut = 10.0;
    let shape = vec![
        rect.left_top() + vec2(cut, 0.0),
        rect.right_top() - vec2(cut, 0.0),
        rect.right_center(),
        rect.right_bottom() - vec2(cut, 0.0),
        rect.left_bottom() + vec2(cut, 0.0),
        rect.left_center(),
    ];
    p.add(Shape::convex_polygon(
        shape,
        if emphasized { c.raised } else { c.seat },
        Stroke::new(1.0, if emphasized { c.gold } else { c.fine }),
    ));
    let icon = pos2(rect.left() + 16.0, rect.center().y);
    p.add(Shape::convex_polygon(
        diamond(icon, 3.0, 5.0),
        c.gold,
        Stroke::NONE,
    ));
    p.text(
        pos2(rect.left() + 27.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        serif(14.0),
        c.ink,
    );
    p.text(
        pos2(rect.right() - 15.0, rect.center().y),
        Align2::RIGHT_CENTER,
        key,
        FontId::monospace(10.0),
        c.gold,
    );
    r.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, active, label));
    r.clicked()
}

fn hit(ui: &mut egui::Ui, rect: Rect, id: impl egui::AsIdSalt, label: &str) -> egui::Response {
    let response = ui
        .interact(rect, ui.make_persistent_id(id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    response
}

fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SERIF.into()))
}

fn diamond(center: Pos2, w: f32, h: f32) -> Vec<Pos2> {
    vec![
        center - vec2(0.0, h),
        center + vec2(w, 0.0),
        center + vec2(0.0, h),
        center - vec2(w, 0.0),
    ]
}

fn gradient(p: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(Shape::mesh(mesh));
}

fn halo(p: &Painter, center: Pos2, radii: Vec2, color: Color32) {
    let mut mesh = egui::Mesh::default();
    let rings = [(0.0, 0.38), (0.38, 0.25), (0.67, 0.08), (1.0, 0.0)];
    const SEGMENTS: u32 = 40;
    for (ring, (r, alpha)) in rings.into_iter().enumerate() {
        for i in 0..=SEGMENTS {
            let step = Vec2::angled(i as f32 * TAU / SEGMENTS as f32);
            mesh.colored_vertex(
                center + vec2(step.x * radii.x, step.y * radii.y) * r,
                color.gamma_multiply(alpha),
            );
            if ring > 0 && i < SEGMENTS {
                let here = ring as u32 * (SEGMENTS + 1) + i;
                let before = here - SEGMENTS - 1;
                mesh.add_triangle(before, here, here + 1);
                mesh.add_triangle(before, here + 1, before + 1);
            }
        }
    }
    p.add(Shape::mesh(mesh));
}

fn arch(rect: Rect) -> Vec<Pos2> {
    let top_height = rect.height() * 0.31;
    let mut points: Vec<_> = (0..=40)
        .map(|index| {
            let angle = PI + index as f32 * PI / 40.0;
            pos2(
                rect.center().x + angle.cos() * rect.width() * 0.5,
                rect.top() + top_height + angle.sin() * top_height,
            )
        })
        .collect();
    points.extend([rect.right_bottom(), rect.left_bottom()]);
    points
}

fn draw_arch_image(p: &Painter, rect: Rect, image: &egui::TextureHandle) {
    let mut mesh = egui::Mesh::with_texture(image.id());
    let path = arch(rect);
    mesh.vertices
        .extend(path.iter().map(|&pos| egui::epaint::Vertex {
            pos,
            uv: pos2(
                (pos.x - rect.left()) / rect.width(),
                (pos.y - rect.top()) / rect.height(),
            ),
            color: Color32::WHITE,
        }));
    for i in 1..path.len() as u32 - 1 {
        mesh.add_triangle(0, i, i + 1);
    }
    p.add(Shape::mesh(mesh));
}
