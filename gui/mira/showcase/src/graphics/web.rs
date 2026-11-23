use super::*;
use eframe::egui::Pos2;

pub(super) const SAMPLES: [Specimen; 5] = [
    Specimen {
        name: "Веб-герой",
        rating: ("A", 6),
        face: "Segoe UI Bold",
        file: "segoeuib.ttf",
    },
    Specimen {
        name: "Пиксель-арт",
        rating: ("S", 5),
        face: "Сетка 5×7 · Cascadia Mono",
        file: "CascadiaMono.ttf",
    },
    Specimen {
        name: "Телетитр",
        rating: ("C", 1),
        face: "Impact",
        file: "impact.ttf",
    },
    Specimen {
        name: "Матовое стекло",
        rating: ("A", 5),
        face: "Calibri Light",
        file: "calibril.ttf",
    },
    Specimen {
        name: "Хром 2000-х",
        rating: ("A", 2),
        face: "Verdana Bold",
        file: "verdanab.ttf",
    },
];

pub(super) fn paint(p: &Painter, rect: Rect, index: usize, font: impl Fn(f32) -> FontId) {
    let scale = rect.width() / 417.0;
    let at = |x: f32, y: f32| rect.min + rect.size() * vec2(x / 417.0, y / 226.0);
    let bounds = |x, y, right, bottom| Rect::from_min_max(at(x, y), at(right, bottom));
    let stroke = |width: f32, color| Stroke::new(width * scale, color);
    let rgb = Color32::from_rgb;
    let sans = FontId::proportional(13.0 * scale);

    match index {
        0 => {
            let pale = rgb(248, 245, 255);
            let purple = rgb(95, 56, 226);
            let dark = rgb(28, 21, 54);
            vertical_gradient(p, rect, Color32::WHITE, pale);
            soft_light(
                p,
                at(348.0, 128.0),
                vec2(178.0, 129.0) * scale,
                rgb(216, 209, 255),
            );

            let diagram = bounds(252.0, 43.0, 389.0, 184.0);
            let radius = (19.0 * scale).round() as u8;
            p.rect_filled(
                diagram.translate(vec2(0.0, 5.0) * scale),
                radius,
                rgb(226, 217, 248),
            );
            p.rect_filled(diagram, radius, rgb(251, 249, 255));
            p.rect_stroke(
                diagram,
                radius,
                stroke(1.0, rgb(204, 191, 240)),
                StrokeKind::Inside,
            );

            let chart = bounds(266.0, 57.0, 375.0, 125.0);
            p.rect_filled(chart, (10.0 * scale).round() as u8, rgb(239, 232, 252));
            for row in [76.0, 94.0, 111.0] {
                p.line_segment(
                    [at(278.0, row), at(363.0, row)],
                    stroke(0.8, rgb(214, 201, 241)),
                );
            }
            p.add(Shape::line(
                vec![
                    at(278.0, 106.0),
                    at(294.0, 98.0),
                    at(311.0, 104.0),
                    at(327.0, 83.0),
                    at(345.0, 86.0),
                    at(361.0, 68.0),
                ],
                stroke(2.5, purple),
            ));
            p.circle_filled(at(361.0, 68.0), 3.7 * scale, purple);
            p.rect_filled(bounds(266.0, 136.0, 311.0, 169.0), 7, rgb(206, 191, 251));
            p.rect_filled(bounds(318.0, 136.0, 375.0, 169.0), 7, rgb(234, 225, 253));
            p.hline(
                at(329.0, 148.0).x..=at(363.0, 148.0).x,
                at(329.0, 148.0).y,
                stroke(2.0, rgb(162, 130, 235)),
            );
            p.hline(
                at(329.0, 157.0).x..=at(352.0, 157.0).x,
                at(329.0, 157.0).y,
                stroke(2.0, rgb(192, 166, 245)),
            );

            p.text(at(26.0, 86.0), Align2::LEFT_CENTER, FIRST, font(48.0), dark);
            p.text(
                at(25.0, 132.0),
                Align2::LEFT_CENTER,
                SECOND,
                font(48.0),
                purple,
            );
            p.text(
                at(26.0, 176.0),
                Align2::LEFT_CENTER,
                LINE,
                sans,
                rgb(64, 51, 93),
            );
        }
        1 => {
            let midnight = rgb(29, 30, 60);
            let lake = rgb(67, 101, 148);
            let sand = rgb(255, 224, 161);
            p.rect_filled(rect, 0, midnight);
            p.rect_filled(bounds(207.0, 0.0, 417.0, 151.0), 0, rgb(44, 47, 90));
            p.rect_filled(bounds(0.0, 151.0, 417.0, 226.0), 0, lake);

            const SUN: [&str; 11] = [
                "....#####....",
                "..#########..",
                ".###########.",
                ".###########.",
                "#############",
                "#############",
                "#############",
                ".###########.",
                ".###########.",
                "..#########..",
                "....#####....",
            ];
            for (row, cells) in SUN.iter().enumerate() {
                let y = 28.0 + row as f32 * 7.0;
                let color = if row < 5 { sand } else { rgb(246, 162, 132) };
                for (column, cell) in cells.bytes().enumerate() {
                    if cell == b'#' {
                        let x = 251.0 + column as f32 * 7.0;
                        p.rect_filled(bounds(x, y, x + 7.0, y + 7.0), 0, color);
                    }
                }
            }

            for (x, y, width) in [
                (226.0, 52.0, 49.0),
                (278.0, 84.0, 93.0),
                (302.0, 90.0, 62.0),
                (350.0, 38.0, 42.0),
            ] {
                p.rect_filled(bounds(x, y, x + width, y + 5.0), 0, rgb(83, 78, 123));
            }
            let shore = [
                129.0, 132.0, 137.0, 142.0, 145.0, 145.0, 141.0, 137.0, 126.0, 117.0, 109.0, 119.0,
                122.0, 130.0,
            ];
            let tile_width = 15.0;
            for (tile, y) in shore.into_iter().enumerate() {
                let x = 207.0 + tile as f32 * tile_width;
                p.rect_filled(bounds(x, y, x + tile_width, 151.0), 0, rgb(41, 59, 104));
                p.rect_filled(bounds(x, y, x + tile_width, y + 3.0), 0, rgb(90, 102, 156));
            }
            for (x, y, width) in [
                (261.0, 155.0, 56.0),
                (271.0, 162.0, 52.0),
                (283.0, 170.0, 33.0),
                (265.0, 177.0, 49.0),
                (316.0, 185.0, 22.0),
            ] {
                p.rect_filled(bounds(x, y, x + width, y + 3.0), 0, rgb(207, 142, 158));
            }

            let pixel = vec2(rect.width() / 417.0, rect.height() / 226.0) * 5.4;
            for (word, y) in [(FIRST, 47.0), (SECOND, 99.0)] {
                block_text(p, at(32.0, y + 4.0), pixel, word, rgb(165, 107, 127));
                block_text(p, at(29.0, y), pixel, word, sand);
            }
            p.rect_filled(bounds(22.0, 175.0, 259.0, 206.0), 0, midnight);
            p.text(
                at(29.0, 190.0),
                Align2::LEFT_CENTER,
                LINE,
                font(12.5),
                rgb(230, 225, 209),
            );
        }
        2 => {
            let navy = rgb(15, 27, 67);
            let blue = rgb(24, 67, 157);
            let amber = rgb(255, 189, 44);
            vertical_gradient(p, rect, navy, blue);
            for (x, color) in [
                (147.0, rgb(23, 49, 120)),
                (208.0, rgb(31, 71, 163)),
                (277.0, rgb(44, 88, 186)),
            ] {
                p.add(Shape::convex_polygon(
                    vec![
                        at(x, 0.0),
                        at(x + 34.0, 0.0),
                        at(x + 132.0, 226.0),
                        at(x + 99.0, 226.0),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
            for y in [43.0, 55.0, 67.0] {
                p.line_segment([at(328.0, y), at(388.0, y)], stroke(1.0, rgb(96, 140, 228)));
            }
            p.circle_filled(at(387.0, 32.0), 4.5 * scale, amber);
            p.add(Shape::convex_polygon(
                vec![
                    at(0.0, 103.0),
                    at(375.0, 103.0),
                    at(354.0, 163.0),
                    at(0.0, 163.0),
                ],
                rgb(247, 249, 254),
                Stroke::NONE,
            ));
            p.rect_filled(bounds(0.0, 105.0, 10.0, 162.0), 0, amber);
            p.add(Shape::convex_polygon(
                vec![
                    at(0.0, 164.0),
                    at(354.0, 164.0),
                    at(340.0, 198.0),
                    at(0.0, 198.0),
                ],
                navy,
                Stroke::NONE,
            ));
            p.text(
                at(24.0, 133.0),
                Align2::LEFT_CENTER,
                TITLE.to_uppercase(),
                font(41.0),
                navy,
            );
            p.text(
                at(25.0, 181.0),
                Align2::LEFT_CENTER,
                LINE,
                sans,
                rgb(234, 242, 255),
            );
        }
        3 => {
            let violet = rgb(205, 202, 249);
            let cyan = rgb(184, 216, 249);
            vertical_gradient(p, rect, rgb(224, 215, 247), cyan);
            soft_light(
                p,
                at(18.0, 2.0),
                vec2(200.0, 154.0) * scale,
                rgb(114, 100, 223),
            );
            soft_light(
                p,
                at(389.0, 213.0),
                vec2(183.0, 155.0) * scale,
                rgb(116, 166, 245),
            );
            p.circle_filled(at(100.0, 139.0), 72.0 * scale, rgb(169, 152, 238));
            p.circle_filled(at(326.0, 76.0), 64.0 * scale, rgb(146, 192, 246));
            p.circle_filled(at(300.0, 170.0), 21.0 * scale, rgb(220, 208, 248));

            let glass = bounds(44.0, 46.0, 371.0, 177.0);
            let corner = (20.0 * scale).round() as u8;
            p.rect_filled(
                glass.translate(vec2(3.0, 8.0) * scale),
                corner,
                Color32::from_rgba_unmultiplied(45, 42, 128, 35),
            );
            p.rect_filled(
                glass,
                corner,
                Color32::from_rgba_unmultiplied(247, 248, 255, 190),
            );
            p.rect_stroke(
                glass,
                corner,
                stroke(1.2, Color32::from_rgba_unmultiplied(255, 255, 255, 246)),
                StrokeKind::Inside,
            );
            p.text(
                at(208.0, 103.0),
                Align2::CENTER_CENTER,
                TITLE,
                font(44.0),
                rgb(36, 42, 92),
            );
            p.text(
                at(208.0, 143.0),
                Align2::CENTER_CENTER,
                LINE,
                sans,
                rgb(56, 65, 106),
            );
            p.circle_filled(at(349.0, 66.0), 4.0 * scale, violet);
        }
        _ => {
            let lilac = rgb(186, 162, 250);
            let steel = rgb(29, 45, 101);
            vertical_gradient(p, rect, rgb(13, 11, 33), rgb(36, 23, 73));
            soft_light(
                p,
                at(213.0, 110.0),
                vec2(210.0, 112.0) * scale,
                rgb(87, 65, 164),
            );
            p.add(Shape::ellipse_stroke(
                at(210.0, 106.0),
                vec2(179.0, 57.0) * scale,
                stroke(1.1, rgb(177, 137, 237)),
            ));
            p.add(Shape::ellipse_stroke(
                at(213.0, 106.0),
                vec2(156.0, 76.0) * scale,
                stroke(0.8, rgb(106, 98, 193)),
            ));
            for (x, y, length) in [
                (80.0, 49.0, 7.0),
                (337.0, 64.0, 9.0),
                (354.0, 146.0, 6.0),
                (57.0, 141.0, 5.0),
            ] {
                let c = at(x, y);
                let radius = length * scale;
                for (width, height) in [(radius * 0.36, radius), (radius, radius * 0.36)] {
                    p.add(Shape::convex_polygon(
                        vec![
                            c + vec2(0.0, -height),
                            c + vec2(width, 0.0),
                            c + vec2(0.0, height),
                            c + vec2(-width, 0.0),
                        ],
                        rgb(232, 222, 255),
                        Stroke::NONE,
                    ));
                }
            }

            let title = p.layout_no_wrap(TITLE.to_owned(), font(42.0), Color32::WHITE);
            let origin = at(208.0, 107.0) - title.size() * 0.5;
            for depth in (1..=5).rev() {
                let offset = depth as f32 * scale;
                let color = if depth < 3 { rgb(95, 125, 207) } else { steel };
                p.galley_with_override_text_color(
                    origin + vec2(offset, offset * 1.25),
                    title.clone(),
                    color,
                );
            }
            for offset in [
                vec2(-1.6, 0.0),
                vec2(0.0, -1.6),
                vec2(1.6, 0.0),
                vec2(0.0, 1.6),
            ] {
                p.galley_with_override_text_color(origin + offset * scale, title.clone(), lilac);
            }
            let chromed = [
                (0.00, 0.40, rgb(211, 229, 255)),
                (0.40, 0.50, rgb(249, 253, 255)),
                (0.50, 0.61, rgb(76, 97, 191)),
                (0.61, 0.76, rgb(177, 157, 241)),
                (0.76, 1.00, rgb(232, 218, 255)),
            ];
            for (from, to, color) in chromed {
                let clip = Rect::from_min_max(
                    pos2(rect.left(), origin.y + title.size().y * from),
                    pos2(rect.right(), origin.y + title.size().y * to),
                );
                p.with_clip_rect(p.clip_rect().intersect(clip))
                    .galley_with_override_text_color(origin, title.clone(), color);
            }
            p.text(
                at(208.0, 172.0),
                Align2::CENTER_CENTER,
                LINE,
                sans,
                rgb(228, 217, 252),
            );
        }
    }
}

fn block_text(p: &Painter, origin: Pos2, pixel: Vec2, text: &str, color: Color32) {
    for (letter, glyph) in text.chars().flat_map(char::to_uppercase).enumerate() {
        let rows: [u8; 7] = match glyph {
            'П' => [0x1f, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11],
            'О' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
            'С' => [0x0f, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0f],
            'Л' => [0x07, 0x09, 0x09, 0x09, 0x09, 0x09, 0x11],
            'Е' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
            'Д' => [0x0e, 0x0a, 0x0a, 0x0a, 0x12, 0x1f, 0x11],
            'Ж' => [0x15, 0x15, 0x0e, 0x04, 0x0e, 0x15, 0x15],
            'Я' => [0x0f, 0x11, 0x11, 0x0f, 0x05, 0x09, 0x11],
            _ => continue,
        };
        for (row, mask) in rows.into_iter().enumerate() {
            for column in 0..5 {
                if mask & (1 << (4 - column)) != 0 {
                    let offset = vec2((letter * 6 + column) as f32, row as f32) * pixel;
                    p.rect_filled(Rect::from_min_size(origin + offset, pixel), 0, color);
                }
            }
        }
    }
}

fn soft_light(p: &Painter, center: Pos2, radius: Vec2, color: Color32) {
    const SEGMENTS: u32 = 40;
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    for index in 0..SEGMENTS {
        let angle = std::f32::consts::TAU * (index as f32 / SEGMENTS as f32);
        mesh.colored_vertex(
            center + vec2(angle.cos(), angle.sin()) * radius,
            Color32::TRANSPARENT,
        );
        mesh.add_triangle(0, index + 1, (index + 1) % SEGMENTS + 1);
    }
    p.add(Shape::mesh(mesh));
}
