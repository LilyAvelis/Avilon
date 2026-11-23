use std::time::Duration;

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Vec2, WidgetInfo, WidgetType, pos2, vec2};

const WIDTH: f32 = 320.0;
const HEIGHT: f32 = 140.0;
const GROUND: f32 = 109.0;
const ORIGIN_X: f32 = 78.0;

const FRAMES: [[&str; 12]; 6] = [
    [
        "..................++",
        "................++..",
        "..............++....",
        "...........####.....",
        "........###hhss#....",
        ".....###hrssswkss...",
        "r####rssrsrsssrr#...",
        ".rrrrrssrrsrrrr#....",
        "..##rrr#rrrrr##.....",
        "....####r#r##.......",
        "........r..r........",
        ".......r....r.......",
    ],
    [
        "....................",
        "....................",
        "..................++",
        "...............+++..",
        "...........####+....",
        "........###hhss#....",
        "...##.##hrssswkss...",
        ".##rr#rsrsrsssrr#...",
        "..rrrrsrrrsrrrr#....",
        "....###rrrrrr##.....",
        ".......##r##r#......",
        ".........rr.rr......",
    ],
    [
        "..................++",
        "................++..",
        "..............++....",
        "............##+.....",
        "..........##hs##....",
        "........##hrswkss...",
        ".......#hsrsssrr#...",
        "......#srsrrr##.....",
        "...###srrrrr#.......",
        "..#rrr#rrr##r.......",
        "...rrr####.r........",
        "....##...r..........",
    ],
    [
        "...................+",
        "................+++.",
        ".............+++....",
        "...........####.....",
        ".........##hhss#....",
        ".......##hrsswkss...",
        "......#srsrsssrr#...",
        "...###rsrrsrrrr#....",
        "..#rrr##rrrrr##.....",
        "...rrrr######.......",
        "....##.rr..r........",
        "....................",
    ],
    [
        "...............+++..",
        "..................++",
        ".............####...",
        "....###....##hss#...",
        "..##rrr##.#hrswkss..",
        "...rrrssr#srsssrr#..",
        "....##rsrsrrsrrr#...",
        "......##rrrrr###....",
        "........###r###.....",
        "..........r..r......",
        "..........r...r.....",
        ".........r.....r....",
    ],
    [
        "....................",
        "....................",
        ".................+++",
        "..............+++...",
        "..........####......",
        ".......###hhss##....",
        "..#####hrssswkss....",
        "r#rrrssrsrsssrrr#...",
        ".rrrrsrrrsrrrr##....",
        "...##rrrrrr###......",
        ".....##r#r##........",
        "......r...rr........",
    ],
];

struct Jump {
    from: f32,
    to: f32,
    elapsed: f32,
    duration: f32,
}

#[derive(Default)]
pub(super) struct Pixel {
    offset: f32,
    facing_left: bool,
    jump: Option<Jump>,
}

impl Pixel {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, canvas: Rect, dt: f32) {
        let scale = (canvas.width() / WIDTH).min(canvas.height() / HEIGHT);
        if scale <= 0.0 {
            return;
        }
        let field = Rect::from_center_size(canvas.center(), vec2(WIDTH, HEIGHT) * scale);
        let response = ui
            .interact(
                field,
                ui.make_persistent_id("pixel-jump"),
                egui::Sense::click(),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Прыгнуть"));
        let target_at = |point: Pos2| ((point.x - field.left()) / scale).clamp(26.0, WIDTH - 26.0);

        if let Some(jump) = &mut self.jump {
            jump.elapsed += dt.clamp(0.0, 0.06);
            if jump.elapsed >= jump.duration {
                self.offset = jump.to - ORIGIN_X;
                self.jump = None;
            }
        }
        let standing = ORIGIN_X + self.offset;
        if response.clicked() && self.jump.is_none() {
            let target = response.interact_pointer_pos().map(target_at).unwrap_or(
                if standing < WIDTH * 0.5 {
                    242.0
                } else {
                    ORIGIN_X
                },
            );
            let distance = (target - standing).abs();
            if distance >= 1.0 {
                self.facing_left = target < standing;
            }
            self.jump = Some(Jump {
                from: standing,
                to: target,
                elapsed: 0.0,
                duration: 0.64 + distance * 0.0014,
            });
        }

        let p = ui.painter_at(canvas);
        p.rect_filled(canvas, 0, Color32::from_rgb(18, 29, 46));
        let scene = PixelGrid {
            p: p.with_clip_rect(field.intersect(p.clip_rect())),
            origin: field.min,
            scale,
            pixels_per_point: ui.ctx().pixels_per_point(),
        };
        draw_scene(&scene);

        let (x, lift, frame) = if let Some(jump) = &self.jump {
            ui.ctx().request_repaint_after(Duration::from_millis(19));
            let progress = (jump.elapsed / jump.duration).clamp(0.0, 1.0);
            let travel = ((progress - 0.13) / 0.73).clamp(0.0, 1.0);
            let height = (23.0 + (jump.to - jump.from).abs() * 0.16).min(51.0);
            let lift = 4.0 * travel * (1.0 - travel) * height;
            let x = jump.from + (jump.to - jump.from) * travel;
            let frame = if progress < 0.13 {
                1
            } else if progress < 0.32 {
                2
            } else if progress < 0.60 {
                3
            } else if progress < 0.86 {
                4
            } else {
                5
            };
            draw_destination(&scene, jump.to);
            if progress >= 0.86 {
                let dust = (progress - 0.86) / 0.14;
                for (side, direction) in [(-1.0, 0.8), (1.0, 1.0)] {
                    let dx = (14.0 + 11.0 * dust * direction) * side;
                    let dy = 4.0 + 9.0 * dust * (1.0 - dust) * direction;
                    scene.fill(jump.to + dx, GROUND - dy, 2.0, 2.0, rgb(242, 198, 134));
                }
            }
            (x, lift, frame)
        } else {
            if let Some(pointer) = response.hover_pos() {
                draw_destination(&scene, target_at(pointer));
            }
            (standing, 0.0, 0)
        };

        let shadow = (14.0 - lift * 0.14).max(4.0);
        let x = x.round();
        scene.fill(x - shadow, GROUND + 1.0, shadow * 2.0, 2.0, rgb(39, 43, 68));
        draw_shrimp(&scene, x, GROUND - lift.round(), frame, self.facing_left);
        for index in 0..FRAMES.len() {
            let color = if index == frame {
                rgb(255, 221, 153)
            } else {
                rgb(72, 93, 106)
            };
            scene.fill(WIDTH - 51.0 + index as f32 * 6.0, 128.0, 3.0, 3.0, color);
        }
    }
}

struct PixelGrid {
    p: Painter,
    origin: Pos2,
    scale: f32,
    pixels_per_point: f32,
}

impl PixelGrid {
    fn point(&self, position: Vec2) -> Pos2 {
        let pos = self.origin + position * self.scale;
        pos2(
            (pos.x * self.pixels_per_point).round() / self.pixels_per_point,
            (pos.y * self.pixels_per_point).round() / self.pixels_per_point,
        )
    }

    fn fill(&self, x: f32, y: f32, w: f32, h: f32, color: Color32) {
        let rect = Rect::from_min_max(self.point(vec2(x, y)), self.point(vec2(x + w, y + h)));
        self.p.rect_filled(rect, 0, color);
    }
}

fn draw_scene(scene: &PixelGrid) {
    scene.fill(0.0, 0.0, WIDTH, HEIGHT, rgb(26, 40, 60));
    scene.fill(0.0, 49.0, WIDTH, 60.0, rgb(34, 55, 75));
    scene.fill(0.0, 78.0, WIDTH, 31.0, rgb(44, 66, 88));

    for (column, step) in [3.0, 2.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0]
        .into_iter()
        .enumerate()
    {
        let x = 242.0 + column as f32 * 4.0;
        let height = 44.0 - step * 8.0;
        scene.fill(x, 45.0 - height * 0.5, 4.0, height, rgb(247, 163, 139));
    }
    scene.fill(238.0, 52.0, 53.0, 3.0, rgb(193, 114, 124));
    scene.fill(247.0, 59.0, 38.0, 3.0, rgb(193, 114, 124));

    for (x, y, w) in [
        (31.0, 25.0, 28.0),
        (45.0, 28.0, 22.0),
        (129.0, 38.0, 39.0),
        (143.0, 41.0, 36.0),
        (199.0, 20.0, 22.0),
        (203.0, 23.0, 32.0),
    ] {
        scene.fill(x, y, w, 2.0, rgb(63, 76, 104));
    }
    for (x, y) in [
        (23.0, 41.0),
        (74.0, 19.0),
        (95.0, 36.0),
        (107.0, 48.0),
        (182.0, 25.0),
        (215.0, 44.0),
        (301.0, 32.0),
    ] {
        scene.fill(x, y, 2.0, 2.0, rgb(87, 107, 127));
    }

    let peaks = [
        77, 73, 69, 65, 69, 73, 81, 81, 85, 89, 85, 81, 81, 77, 73, 77, 85, 89, 93, 89, 89, 85, 81,
        85, 89, 89, 93, 93, 93, 89, 89, 85, 85, 81, 77, 73, 73, 77, 81, 85,
    ];
    for (column, peak) in peaks.into_iter().enumerate() {
        let y = peak as f32;
        let x = column as f32 * 8.0;
        scene.fill(x, y, 8.0, GROUND - y, rgb(52, 76, 100));
        scene.fill(x, y, 8.0, 2.0, rgb(73, 96, 121));
    }
    scene.fill(0.0, 96.0, WIDTH, GROUND - 96.0, rgb(43, 64, 87));
    for (x, y, w) in [
        (0.0, 100.0, 32.0),
        (83.0, 103.0, 22.0),
        (113.0, 99.0, 38.0),
        (169.0, 101.0, 41.0),
        (218.0, 99.0, 36.0),
        (245.0, 103.0, 24.0),
        (272.0, 101.0, 17.0),
    ] {
        scene.fill(x, y, w, 1.0, rgb(78, 103, 121));
    }

    scene.fill(0.0, GROUND, WIDTH, 2.0, rgb(140, 147, 144));
    scene.fill(0.0, GROUND + 2.0, WIDTH, 6.0, rgb(94, 115, 116));
    scene.fill(
        0.0,
        GROUND + 8.0,
        WIDTH,
        HEIGHT - GROUND - 8.0,
        rgb(27, 45, 63),
    );
    for (x, y, w) in [
        (9.0, 120.0, 31.0),
        (16.0, 128.0, 12.0),
        (55.0, 126.0, 16.0),
        (84.0, 136.0, 35.0),
        (124.0, 121.0, 22.0),
        (156.0, 127.0, 31.0),
        (192.0, 133.0, 18.0),
        (213.0, 123.0, 24.0),
    ] {
        scene.fill(x, y, w, 2.0, rgb(45, 67, 86));
    }
}

fn draw_destination(scene: &PixelGrid, x: f32) {
    let x = x.round();
    let amber = rgb(248, 196, 137);
    scene.fill(x - 5.0, GROUND + 3.0, 11.0, 2.0, amber);
    scene.fill(x - 3.0, GROUND + 5.0, 7.0, 2.0, amber);
    scene.fill(x - 1.0, GROUND + 7.0, 3.0, 2.0, amber);
}

fn draw_shrimp(scene: &PixelGrid, x: f32, ground: f32, frame: usize, facing_left: bool) {
    for (row, cells) in FRAMES[frame].iter().enumerate() {
        for (column, pixel) in cells.bytes().enumerate() {
            let color = match pixel {
                b'#' => rgb(88, 45, 63),
                b'r' => rgb(219, 106, 101),
                b's' => rgb(248, 145, 115),
                b'h' => rgb(255, 204, 152),
                b'w' => rgb(255, 231, 183),
                b'k' => rgb(28, 26, 48),
                b'+' => rgb(255, 176, 133),
                _ => continue,
            };
            let column = if facing_left { 19 - column } else { column };
            scene.fill(
                x - 20.0 + column as f32 * 2.0,
                ground - 24.0 + row as f32 * 2.0,
                2.0,
                2.0,
                color,
            );
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}
