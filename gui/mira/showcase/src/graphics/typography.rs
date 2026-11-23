use super::*;

pub(super) const SAMPLES: [Specimen; 5] = [
    Specimen {
        name: "Швейцарская сетка",
        rating: ("B", 6),
        face: "Arial Bold",
        file: "arialbd.ttf",
    },
    Specimen {
        name: "Газетная полоса",
        rating: ("C", 2),
        face: "Cambria Bold",
        file: "cambriab.ttf",
    },
    Specimen {
        name: "Брутализм",
        rating: ("B", 3),
        face: "Impact",
        file: "impact.ttf",
    },
    Specimen {
        name: "Терминал",
        rating: ("A", 4),
        face: "Consolas",
        file: "consola.ttf",
    },
    Specimen {
        name: "Тихий каталог",
        rating: ("B", 5),
        face: "Times New Roman",
        file: "times.ttf",
    },
];

pub(super) fn paint(p: &Painter, rect: Rect, index: usize, font: impl Fn(f32) -> FontId) {
    let p = p.with_clip_rect(rect);
    let at = |x, y| rect.min + rect.size() * vec2(x, y);
    let scale = rect.width() / 417.0;
    let hairline = scale.max(0.65);
    let line = |left, right, y, width, color| {
        p.line_segment([at(left, y), at(right, y)], Stroke::new(width, color));
    };

    match index {
        0 => {
            let black = Color32::from_rgb(25, 25, 24);
            let red = Color32::from_rgb(200, 35, 37);
            let rule = Color32::from_rgb(174, 173, 161);
            p.rect_filled(rect, 0, Color32::from_rgb(249, 248, 241));
            p.rect_filled(
                Rect::from_min_max(at(0.714, 0.10), at(0.933, 0.322)),
                0,
                red,
            );
            line(0.066, 0.678, 0.10, 1.5 * scale, black);
            p.text(
                at(0.066, 0.485),
                Align2::LEFT_CENTER,
                TITLE,
                font(48.0),
                black,
            );
            line(0.066, 0.933, 0.657, hairline, black);
            p.vline(
                at(0.714, 0.0).x,
                at(0.0, 0.657).y..=at(0.0, 0.908).y,
                Stroke::new(hairline, rule),
            );
            p.text(
                at(0.066, 0.762),
                Align2::LEFT_CENTER,
                LINE,
                FontId::proportional(14.0 * scale),
                black,
            );
        }
        1 => {
            let charcoal = Color32::from_rgb(39, 30, 23);
            let paper = Color32::from_rgb(239, 223, 187);
            let fine = Color32::from_rgb(161, 139, 105);
            vertical_gradient(&p, rect, paper, Color32::from_rgb(230, 209, 168));
            line(0.074, 0.926, 0.147, 3.4 * scale, charcoal);
            line(0.074, 0.926, 0.176, hairline, charcoal);
            p.text(
                at(0.50, 0.391),
                Align2::CENTER_CENTER,
                TITLE,
                font(50.0),
                charcoal,
            );
            line(0.074, 0.926, 0.602, hairline, charcoal);
            p.text(
                at(0.50, 0.711),
                Align2::CENTER_CENTER,
                LINE,
                font(17.0),
                charcoal,
            );
            line(0.074, 0.926, 0.810, hairline, charcoal);
            for x in [0.358, 0.642] {
                p.vline(
                    at(x, 0.0).x,
                    at(0.0, 0.810).y..=at(0.0, 0.923).y,
                    Stroke::new(hairline, fine),
                );
            }
        }
        2 => {
            let black = Color32::from_rgb(21, 22, 19);
            let acid = Color32::from_rgb(215, 245, 66);
            p.rect_filled(rect, 0, acid);
            p.text(
                at(0.047, 0.205),
                Align2::LEFT_CENTER,
                FIRST,
                font(77.0),
                black,
            );
            let block = Rect::from_min_max(at(0.159, 0.404), at(0.959, 0.791));
            p.rect_filled(block, 0, black);
            p.text(
                at(0.938, 0.572),
                Align2::RIGHT_CENTER,
                SECOND,
                font(85.0),
                acid,
            );
            p.text(
                at(0.954, 0.904),
                Align2::RIGHT_CENTER,
                LINE,
                FontId::monospace(12.0 * scale),
                black,
            );
        }
        3 => {
            let violet = Color32::from_rgb(31, 25, 51);
            let magenta = Color32::from_rgb(234, 147, 255);
            let milk = Color32::from_rgb(229, 235, 250);
            vertical_gradient(&p, rect, Color32::from_rgb(44, 33, 75), violet);
            for m in 1..57 {
                let y = m as f32 / 58.0;
                line(0.0, 1.0, y, 0.65, Color32::from_white_alpha(6));
            }
            p.add(Shape::line(
                vec![at(0.066, 0.417), at(0.105, 0.485), at(0.066, 0.553)],
                Stroke::new(2.0 * scale, magenta),
            ));
            p.text(
                at(0.136, 0.482),
                Align2::LEFT_CENTER,
                TITLE,
                font(41.0),
                milk,
            );
            p.rect_filled(
                Rect::from_min_max(at(0.815, 0.427), at(0.845, 0.564)),
                0,
                magenta,
            );
            line(
                0.137,
                0.938,
                0.631,
                hairline,
                Color32::from_rgb(118, 91, 158),
            );
            p.text(
                at(0.138, 0.755),
                Align2::LEFT_CENTER,
                LINE,
                font(14.0),
                Color32::from_rgb(192, 174, 221),
            );
        }
        4 => {
            let ink = Color32::from_rgb(35, 40, 53);
            let brass = Color32::from_rgb(151, 125, 85);
            let paper = Color32::from_rgb(250, 248, 241);
            p.rect_filled(rect, 0, paper);
            p.rect_stroke(
                Rect::from_min_max(at(0.056, 0.102), at(0.943, 0.898)),
                0,
                Stroke::new(hairline, Color32::from_rgb(205, 192, 169)),
                StrokeKind::Inside,
            );
            line(0.119, 0.217, 0.243, 1.3 * scale, brass);
            p.text(
                at(0.116, 0.533),
                Align2::LEFT_CENTER,
                TITLE,
                font(47.0),
                ink,
            );
            p.text(at(0.120, 0.725), Align2::LEFT_CENTER, LINE, font(19.0), ink);
        }
        _ => unreachable!("unknown typographic specimen"),
    }
}
