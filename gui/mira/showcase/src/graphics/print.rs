use super::*;
use egui::Pos2;

pub(super) const SAMPLES: [Specimen; 5] = [
    Specimen {
        name: "Гравюра",
        rating: ("S+", 3),
        face: "Times New Roman Bold",
        file: "timesbd.ttf",
    },
    Specimen {
        name: "Коллаж",
        rating: ("S", 4),
        face: "Arial Bold",
        file: "arialbd.ttf",
    },
    Specimen {
        name: "Ар-деко",
        rating: ("B", 2),
        face: "Century Gothic",
        file: "GOTHIC.TTF",
    },
    Specimen {
        name: "Книжная обложка",
        rating: ("C", 2),
        face: "Bookman Old Style",
        file: "BOOKOS.TTF",
    },
    Specimen {
        name: "Этикетка",
        rating: ("A", 4),
        face: "Century Gothic Bold",
        file: "GOTHICB.TTF",
    },
];

pub(super) fn paint(p: &Painter, rect: Rect, index: usize, font: impl Fn(f32) -> FontId) {
    let at = |x, y| rect.min + rect.size() * vec2(x, y);
    let scale = rect.width() / 417.0;
    let caption = || FontId::proportional(13.0 * scale);
    let polygon = |points: &[(f32, f32)], color| {
        let points: Vec<Pos2> = points.iter().map(|&(x, y)| at(x, y)).collect();
        p.add(Shape::convex_polygon(points, color, Stroke::NONE));
    };
    let path = |points: &[(f32, f32)], stroke| {
        let points: Vec<Pos2> = points.iter().map(|&(x, y)| at(x, y)).collect();
        p.add(Shape::line(points, stroke));
    };

    match index {
        0 => {
            let paper = Color32::from_rgb(232, 226, 205);
            let ink = Color32::from_rgb(24, 45, 52);
            let wash = Color32::from_rgb(109, 126, 124);
            p.rect_filled(rect, 0, paper);

            for line in 0..27 {
                let row = line as f32 / 26.0;
                let left = 0.48 + (row - 0.51).powi(2) * 0.31;
                let points = (0..=48)
                    .map(|point| {
                        let t = point as f32 / 48.0;
                        let x = left + t * (1.05 - left);
                        let y = 0.11 + row * 0.76 - t * 0.17 * (t * 7.4 + row * 5.0).sin()
                            + t * 0.024 * (t * 27.0 + row * 8.0).sin();
                        at(x, y)
                    })
                    .collect();
                let stroke = if line % 4 == 0 {
                    Stroke::new(1.2 * scale, ink)
                } else {
                    Stroke::new(0.70 * scale, wash)
                };
                p.add(Shape::line(points, stroke));
            }
            p.rect_stroke(
                rect.shrink(10.0 * scale),
                0,
                Stroke::new(0.7 * scale, wash),
                StrokeKind::Inside,
            );
            p.text(at(0.067, 0.29), Align2::LEFT_CENTER, FIRST, font(52.0), ink);
            p.text(
                at(0.063, 0.54),
                Align2::LEFT_CENTER,
                SECOND,
                font(60.0),
                ink,
            );
            path(&[(0.071, 0.69), (0.43, 0.69)], Stroke::new(scale, ink));
            p.text(
                at(0.073, 0.795),
                Align2::LEFT_CENTER,
                LINE,
                FontId::proportional(12.0 * scale),
                ink,
            );
        }
        1 => {
            let clay = Color32::from_rgb(211, 106, 78);
            let pink = Color32::from_rgb(222, 172, 164);
            let sage = Color32::from_rgb(80, 110, 94);
            let teal = Color32::from_rgb(37, 75, 80);
            let yellow = Color32::from_rgb(237, 196, 99);
            let paper = Color32::from_rgb(255, 242, 215);
            let ink = Color32::from_rgb(43, 33, 35);
            p.rect_filled(rect, 0, pink);
            polygon(&[(0.55, 0.0), (1.0, 0.0), (1.0, 1.0), (0.89, 1.0)], clay);
            polygon(&[(0.0, 0.46), (0.41, 0.62), (0.70, 1.0), (0.0, 1.0)], sage);
            p.circle_filled(at(0.78, 0.36), 53.0 * scale, yellow);
            polygon(&[(0.70, 0.60), (1.0, 0.41), (1.0, 1.0), (0.57, 1.0)], teal);
            for row in 0..8 {
                let y = 0.67 + row as f32 * 0.032;
                path(
                    &[(0.81, y + 0.13), (0.96, y)],
                    Stroke::new(0.7 * scale, sage),
                );
            }

            let slip = [
                (0.035, 0.18),
                (0.67, 0.14),
                (0.69, 0.68),
                (0.663, 0.743),
                (0.053, 0.79),
            ];
            let shadow = slip.map(|(x, y)| (x + 0.012, y + 0.019));
            polygon(&shadow, Color32::from_black_alpha(43));
            polygon(&slip, paper);
            polygon(
                &[
                    (0.111, 0.105),
                    (0.28, 0.063),
                    (0.293, 0.222),
                    (0.119, 0.263),
                ],
                Color32::from_rgba_unmultiplied(232, 180, 112, 167),
            );
            p.text(at(0.091, 0.35), Align2::LEFT_CENTER, FIRST, font(46.0), ink);
            p.text(
                at(0.091, 0.57),
                Align2::LEFT_CENTER,
                SECOND,
                font(58.0),
                ink,
            );
            p.text(
                at(0.057, 0.89),
                Align2::LEFT_CENTER,
                LINE,
                caption(),
                Color32::WHITE,
            );
        }
        2 => {
            let base = Color32::from_rgb(17, 31, 32);
            let inset = Color32::from_rgb(29, 49, 48);
            let bronze = Color32::from_rgb(196, 154, 94);
            let cream = Color32::from_rgb(247, 225, 185);
            p.rect_filled(rect, 0, base);
            p.rect_filled(Rect::from_min_max(at(0.18, 0.0), at(0.82, 1.0)), 0, inset);

            for (dx, dy) in [(0.0, 0.0), (0.016, 0.028), (0.032, 0.056)] {
                let side = [
                    (0.236 + dx, 0.08 + dy),
                    (0.144 + dx, 0.08 + dy),
                    (0.144 + dx, 0.18 + dy),
                    (0.102 + dx, 0.18 + dy),
                    (0.102 + dx, 0.29 + dy),
                    (0.056 + dx, 0.29 + dy),
                    (0.056 + dx, 0.71 - dy),
                    (0.102 + dx, 0.71 - dy),
                    (0.102 + dx, 0.82 - dy),
                    (0.144 + dx, 0.82 - dy),
                    (0.144 + dx, 0.92 - dy),
                    (0.236 + dx, 0.92 - dy),
                ];
                let mirrored = side.map(|(x, y)| (1.0 - x, y));
                path(&side, Stroke::new(scale, bronze));
                path(&mirrored, Stroke::new(scale, bronze));
            }

            for i in -4..=4 {
                let x = i as f32 * 0.029;
                path(
                    &[(0.5 + x * 0.38, 0.298), (0.5 + x, 0.122 + x.abs() * 0.53)],
                    Stroke::new(0.8 * scale, bronze),
                );
            }
            polygon(
                &[(0.5, 0.278), (0.516, 0.31), (0.5, 0.342), (0.484, 0.31)],
                bronze,
            );
            p.text(
                at(0.5, 0.49),
                Align2::CENTER_CENTER,
                TITLE,
                font(39.0),
                cream,
            );
            path(&[(0.229, 0.61), (0.771, 0.61)], Stroke::new(scale, bronze));
            p.text(
                at(0.5, 0.716),
                Align2::CENTER_CENTER,
                LINE,
                caption(),
                cream,
            );
            polygon(
                &[(0.5, 0.775), (0.512, 0.803), (0.5, 0.831), (0.488, 0.803)],
                bronze,
            );
        }
        3 => {
            let fabric = Color32::from_rgb(67, 80, 54);
            let spine = Color32::from_rgb(53, 63, 44);
            let gold = Color32::from_rgb(180, 153, 92);
            let paper = Color32::from_rgb(230, 221, 188);
            let ink = Color32::from_rgb(54, 60, 41);
            p.rect_filled(rect, 0, fabric);
            let texture = Stroke::new(0.45 * scale, Color32::from_white_alpha(13));
            for column in 0..65 {
                let x = column as f32 / 65.0;
                path(&[(x, 0.0), (x + 0.023, 1.0)], texture);
            }
            for row in 0..32 {
                let y = row as f32 / 32.0;
                path(&[(0.0, y), (1.0, y)], texture);
            }
            p.rect_filled(Rect::from_min_max(at(0.0, 0.0), at(0.102, 1.0)), 0, spine);
            for x in [0.085, 0.103, 0.120] {
                path(&[(x, 0.0), (x, 1.0)], Stroke::new(0.8 * scale, gold));
            }
            let plate = Rect::from_min_max(at(0.208, 0.131), at(0.899, 0.869));
            p.rect_filled(plate.translate(vec2(2.0, 3.0) * scale), 1, spine);
            p.rect_filled(plate, 1, paper);
            for inset in [6.0, 10.0] {
                p.rect_stroke(
                    plate.shrink(inset * scale),
                    0,
                    Stroke::new(0.7 * scale, gold),
                    StrokeKind::Inside,
                );
            }
            let center_x = 0.554;
            p.text(
                at(center_x, 0.36),
                Align2::CENTER_CENTER,
                TITLE,
                font(32.0),
                ink,
            );
            p.text(
                at(center_x, 0.555),
                Align2::CENTER_CENTER,
                LINE,
                caption(),
                ink,
            );
            for dir in [-1.0, 1.0] {
                path(
                    &[
                        (center_x, 0.727),
                        (center_x + dir * 0.08, 0.685),
                        (center_x + dir * 0.107, 0.69),
                    ],
                    Stroke::new(scale, ink),
                );
                for offset in [0.033, 0.064] {
                    let x = center_x + dir * offset;
                    polygon(
                        &[
                            (x, 0.718),
                            (x + dir * 0.022, 0.656),
                            (x + dir * 0.032, 0.674),
                            (x + dir * 0.017, 0.704),
                        ],
                        ink,
                    );
                }
            }
        }
        _ => {
            let sky = Color32::from_rgb(173, 213, 217);
            let dark = Color32::from_rgb(21, 58, 70);
            let paper = Color32::from_rgb(247, 240, 207);
            let orange = Color32::from_rgb(208, 103, 57);
            p.rect_filled(rect, 0, sky);
            p.rect_filled(Rect::from_min_max(at(0.0, 0.22), at(1.0, 0.78)), 0, dark);
            for y in [0.18, 0.82] {
                path(&[(0.0, y), (1.0, y)], Stroke::new(scale, dark));
            }
            let sheet = Rect::from_min_max(at(0.077, 0.13), at(0.923, 0.87));
            p.rect_filled(sheet, 9.0 * scale, paper);
            p.rect_stroke(
                sheet.shrink(4.0 * scale),
                7.0 * scale,
                Stroke::new(1.25 * scale, dark),
                StrokeKind::Inside,
            );
            path(
                &[(0.713, 0.249), (0.713, 0.752)],
                Stroke::new(1.0 * scale, orange),
            );
            p.text(
                at(0.117, 0.452),
                Align2::LEFT_CENTER,
                TITLE,
                font(33.0),
                dark,
            );
            p.text(at(0.120, 0.655), Align2::LEFT_CENTER, LINE, caption(), dark);
            let seal = at(0.813, 0.5);
            p.circle_stroke(seal, 29.0 * scale, Stroke::new(1.1 * scale, orange));
            p.circle_stroke(seal, 25.0 * scale, Stroke::new(0.8 * scale, orange));
            polygon(
                &[
                    (0.813, 0.378),
                    (0.848, 0.511),
                    (0.838, 0.559),
                    (0.813, 0.586),
                    (0.788, 0.559),
                    (0.778, 0.511),
                ],
                dark,
            );
            path(
                &[(0.79, 0.509), (0.793, 0.538), (0.807, 0.558)],
                Stroke::new(1.4 * scale, sky),
            );
        }
    }
}
