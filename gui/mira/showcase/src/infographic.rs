use std::time::Duration;

use eframe::egui::{
    self, Align2, Color32, CursorIcon, Key, Rect, Sense, Stroke, StrokeKind, TextureHandle,
    TextureOptions, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{animation::complexity_badge, consume_key, verstka};

mod html;

const ASPECT: f32 = 16.0 / 9.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Cut {
    #[default]
    Weight,
    Relay,
    ListToMap,
    NewFocus,
    Sync,
    Explained,
    Short,
    Spectre,
}

const CUTS: [Cut; 8] = [
    Cut::Weight,
    Cut::Relay,
    Cut::ListToMap,
    Cut::NewFocus,
    Cut::Sync,
    Cut::Explained,
    Cut::Short,
    Cut::Spectre,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Category {
    #[default]
    All,
    Remotion,
    Html,
    Python,
}

impl Category {
    const TABS: [(Self, &'static str); 4] = [
        (Self::All, "Все"),
        (Self::Remotion, "Remotion"),
        (Self::Html, "HTML"),
        (Self::Python, "Python"),
    ];

    fn includes(self, cut: Cut) -> bool {
        self == Self::All || self == cut.category()
    }

    fn cuts(self) -> impl Iterator<Item = Cut> {
        CUTS.into_iter().filter(move |cut| self.includes(*cut))
    }
}

impl Cut {
    fn title(self) -> &'static str {
        match self {
            Self::Weight => "Вес",
            Self::Relay => "Передача",
            Self::ListToMap => "Из списка в карту",
            Self::NewFocus => "Новое главное",
            Self::Sync => "Настройка",
            Self::Explained => "С объяснением",
            Self::Short => "Короткая",
            Self::Spectre => "Луч Spectre 360°",
        }
    }

    fn complexity(self) -> Option<usize> {
        match self {
            Self::Weight => Some(1),
            Self::Relay | Self::NewFocus => Some(2),
            Self::Sync => Some(3),
            Self::ListToMap => Some(4),
            Self::Explained | Self::Short | Self::Spectre => None,
        }
    }

    fn category(self) -> Category {
        match self {
            Self::Spectre => Category::Html,
            _ => Category::Remotion,
        }
    }

    fn frames(self) -> Option<Frames> {
        Some(Frames(match self {
            Self::Weight => include_bytes!("../assets/atlas/weight.frames"),
            Self::Relay => include_bytes!("../assets/atlas/relay.frames"),
            Self::ListToMap => include_bytes!("../assets/atlas/list-to-map.frames"),
            Self::NewFocus => include_bytes!("../assets/atlas/new-focus.frames"),
            Self::Sync => include_bytes!("../assets/atlas/sync.frames"),
            Self::Short => include_bytes!("../assets/atlas/mira-cycle.frames"),
            Self::Explained => include_bytes!("../assets/atlas/mira-cycle-explained.frames"),
            Self::Spectre => return None,
        }))
    }

    #[cfg(windows)]
    fn audio(self) -> Option<&'static str> {
        match self {
            Self::Short => Some("mira-cycle.wav"),
            Self::Explained => Some("mira-cycle-explained.wav"),
            _ => None,
        }
    }
}

#[derive(Default)]
pub(super) struct Infographic {
    cut: Cut,
    category: Category,
    last_clip: Cut,
    started_at: f64,
    current_frame: Option<usize>,
    texture: Option<TextureHandle>,
    html_preview: Option<TextureHandle>,
    launch_error: Option<String>,
}

impl Infographic {
    pub(super) fn restart(&mut self, now: f64) {
        self.started_at = now;
        #[cfg(windows)]
        if self.category.includes(self.cut)
            && let Some(audio) = self.cut.audio()
        {
            sound::play(audio);
        }
    }

    pub(super) fn stop(&self) {
        #[cfg(windows)]
        sound::stop();
    }

    fn select(&mut self, cut: Cut, now: f64) {
        if self.cut != cut {
            self.stop();
            self.cut = cut;
            if cut.category() == Category::Remotion {
                self.last_clip = cut;
            }
            self.current_frame = None;
            self.launch_error = None;
            self.restart(now);
        }
    }

    fn change_category(&mut self, category: Category, now: f64) {
        if self.category == category {
            return;
        }
        let was_visible = self.category.includes(self.cut);
        self.category = category;
        if category.includes(self.cut) {
            if !was_visible {
                self.restart(now);
            }
        } else {
            let first = match category {
                Category::Remotion => Some(self.last_clip),
                _ => category.cuts().next(),
            };
            if let Some(cut) = first {
                self.select(cut, now);
            } else {
                self.stop();
            }
        }
    }

    fn neighbor(&self, cuts: &[Cut], offset: isize) -> Cut {
        let index = cuts.iter().position(|&cut| cut == self.cut).unwrap() as isize;
        cuts[(index + offset).rem_euclid(cuts.len() as isize) as usize]
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, margin: f32) {
        ui.painter().rect_filled(ui.max_rect(), 0, verstka::PAPER);
        ui.add_space(8.0);
        let category =
            crate::graphics::section_tabs(ui, &Category::TABS, self.category, margin, 13.0);
        let now = ui.input(|input| input.time);
        self.change_category(category, now);
        let cuts: Vec<_> = self.category.cuts().collect();
        let window = ui.available_rect_before_wrap();
        let painter = ui.painter_at(window);
        let content = window.shrink2(vec2(margin, 18.0));
        if cuts.is_empty() {
            painter.text(
                content.left_top(),
                Align2::LEFT_TOP,
                "Анимаций на Python пока нет",
                verstka::serif(21.0),
                verstka::SECONDARY,
            );
            return;
        }
        let rail_width = (content.width() * 0.28).clamp(172.0, 315.0);
        let rail_left = content.right() - rail_width;
        let plate_right = rail_left - 28.0;
        let offset = consume_key(ui.ctx(), Key::ArrowRight, false) as isize
            - consume_key(ui.ctx(), Key::ArrowLeft, false) as isize;
        self.select(self.neighbor(&cuts, offset), now);
        let rail = Rect::from_min_max(pos2(rail_left, content.top()), content.max);
        painter.vline(
            rail.left() - 14.0,
            content.y_range(),
            Stroke::new(1.0, verstka::RULE),
        );
        self.show_contents(ui, &painter, rail, &cuts, now);

        let stage_width = plate_right - content.left();
        let stage_height = (stage_width / ASPECT).min((content.height() - 70.0).max(1.0));
        let stage = Rect::from_min_size(content.min, vec2(stage_width, stage_height));
        let controls = Rect::from_min_max(
            pos2(stage.left(), stage.bottom() + 29.0),
            pos2(stage.right(), stage.bottom() + 61.0),
        );
        if cuts.len() > 1 && controls.width() >= 344.0 {
            if verstka::text_button(
                ui,
                &painter,
                Rect::from_min_size(controls.min, vec2(87.0, 32.0)),
                "Назад",
                "←",
            ) {
                self.select(self.neighbor(&cuts, -1), now);
                ui.ctx().request_repaint();
            }
            if verstka::text_button(
                ui,
                &painter,
                Rect::from_min_size(controls.right_top() - vec2(87.0, 0.0), vec2(87.0, 32.0)),
                "Дальше",
                "→",
            ) {
                self.select(self.neighbor(&cuts, 1), now);
                ui.ctx().request_repaint();
            }
        }
        if let Some(frames) = self.cut.frames() {
            let replay = consume_key(ui.ctx(), Key::Space, false) > 0;
            if verstka::text_button(
                ui,
                &painter,
                Rect::from_center_size(controls.center(), vec2(118.0, 32.0)),
                "Сначала",
                "Пробел",
            ) || replay
            {
                self.restart(now);
                ui.ctx().request_repaint();
            }
            self.show_clip(frames, &painter, ui.ctx(), stage, now);
        } else {
            self.show_spectre(ui, &painter, stage, controls);
        }
    }

    fn show_spectre(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        stage: Rect,
        controls: Rect,
    ) {
        let texture = self.html_preview.get_or_insert_with(|| {
            let pixels =
                image::load_from_memory_with_format(html::PREVIEW, image::ImageFormat::Jpeg)
                    .expect("the Spectre preview must be a valid JPEG")
                    .into_rgba8();
            ui.ctx().load_texture(
                "spectre-360",
                egui::ColorImage::from_rgba_unmultiplied(
                    [pixels.width() as usize, pixels.height() as usize],
                    pixels.as_raw(),
                ),
                TextureOptions::LINEAR,
            )
        });
        painter.rect_filled(stage, 0, verstka::PANEL);
        let source = texture.size_vec2();
        let scale = (stage.width() / source.x).min(stage.height() / source.y);
        painter.image(
            texture.id(),
            Rect::from_center_size(stage.center(), source * scale),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        painter.rect_stroke(
            stage,
            0,
            Stroke::new(1.0, verstka::RULE),
            StrokeKind::Inside,
        );
        verstka::rule(painter, stage.left(), stage.right(), stage.bottom() + 27.0);
        let preview = ui
            .interact(
                stage,
                ui.make_persistent_id("spectre-preview"),
                Sense::click(),
            )
            .on_hover_cursor(CursorIcon::PointingHand);
        preview.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Button, true, "Открыть «Луч Spectre 360°»")
        });
        let open_button = verstka::text_button(
            ui,
            painter,
            Rect::from_center_size(controls.center(), vec2(162.0, 32.0)),
            "Открыть в браузере",
            "↗",
        );
        if preview.clicked() || open_button {
            self.launch_error = html::open().err().map(|err| err.to_string());
        }
        if let Some(error) = &self.launch_error {
            painter.text(
                controls.left_bottom() + vec2(0.0, 2.0),
                Align2::LEFT_TOP,
                format!("Не удалось открыть страницу · {error}"),
                egui::FontId::proportional(12.0),
                verstka::ACCENT,
            );
        }
    }

    fn show_contents(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rail: Rect,
        cuts: &[Cut],
        now: f64,
    ) {
        painter.text(
            rail.left_top() + vec2(0.0, 14.0),
            Align2::LEFT_CENTER,
            "Содержание",
            verstka::serif(23.0),
            verstka::INK,
        );
        let rows_top = rail.top() + 36.0;
        let row_height = ((rail.bottom() - rows_top - 32.0) / cuts.len() as f32).clamp(24.0, 48.0);
        verstka::rule(painter, rail.left(), rail.right(), rows_top);
        let mut rows = Vec::with_capacity(cuts.len());
        for (index, &cut) in cuts.iter().enumerate() {
            let row = Rect::from_min_size(
                pos2(rail.left(), rows_top + index as f32 * row_height),
                vec2(rail.width(), row_height),
            );
            let response = ui
                .interact(
                    row,
                    ui.make_persistent_id(("animation-cut", cut.title())),
                    Sense::click(),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            if response.clicked() {
                self.select(cut, now);
            }
            rows.push((cut, row, response));
        }
        for (cut, row, response) in rows {
            let active = self.cut == cut;
            response.widget_info(|| {
                WidgetInfo::selected(WidgetType::SelectableLabel, true, active, cut.title())
            });
            verstka::paint_row(
                painter,
                row,
                active,
                response.hovered() || response.has_focus(),
            );
            painter.text(
                row.left_center() + vec2(12.0, 0.0),
                Align2::LEFT_CENTER,
                cut.title(),
                verstka::serif(16.0),
                if active {
                    verstka::ACCENT
                } else {
                    verstka::INK
                },
            );
            let detail = cut.frames().map_or_else(
                || "HTML".to_owned(),
                |frames| format!("{:02} с", (frames.count() as f64 / frames.rate()) as u32),
            );
            painter.text(
                row.right_center() - vec2(8.0, 0.0),
                Align2::RIGHT_CENTER,
                detail,
                verstka::mono(11.0),
                verstka::SECONDARY,
            );
        }
        if let Some(complexity) = self.cut.complexity() {
            let badge = Rect::from_min_size(
                pos2(
                    rail.left() + 1.0,
                    rows_top + row_height * cuts.len() as f32 + 5.0,
                ),
                vec2(166.0, 26.0),
            );
            complexity_badge(ui, badge, complexity);
        }
    }

    fn show_clip(
        &mut self,
        frames: Frames,
        painter: &egui::Painter,
        ctx: &egui::Context,
        stage: Rect,
        now: f64,
    ) {
        let duration = frames.count() as f64 / frames.rate();
        let elapsed = (now - self.started_at).clamp(0.0, duration);
        let frame = frames.at(elapsed);
        if self.current_frame != Some(frame) {
            let pixels =
                image::load_from_memory_with_format(frames.bytes(frame), image::ImageFormat::Jpeg)
                    .expect("embedded infographic frame must be a valid JPEG")
                    .into_rgba8();
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [pixels.width() as usize, pixels.height() as usize],
                pixels.as_raw(),
            );
            if let Some(texture) = &mut self.texture {
                texture.set(image, TextureOptions::LINEAR);
            } else {
                self.texture = Some(ctx.load_texture("mira-cycle", image, TextureOptions::LINEAR));
            }
            self.current_frame = Some(frame);
        }
        if elapsed < duration {
            let next = (frame + 1) as f64 / frames.rate();
            ctx.request_repaint_after(Duration::from_secs_f64((next - elapsed).max(0.001)));
        }

        painter.rect_filled(stage, 0, verstka::PANEL);
        if let Some(texture) = &self.texture {
            let image = Rect::from_center_size(
                stage.center(),
                vec2(stage.height() * ASPECT, stage.height()),
            );
            painter.image(
                texture.id(),
                image,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        painter.rect_stroke(
            stage,
            0,
            Stroke::new(1.0, verstka::RULE),
            StrokeKind::Inside,
        );
        let y = stage.bottom() + 15.0;
        let end = stage.right() - 88.0;
        verstka::rule(painter, stage.left(), end, y);
        let position = egui::lerp(stage.left()..=end, (elapsed / duration) as f32);
        painter.hline(
            stage.left()..=position,
            y,
            Stroke::new(2.0, verstka::ACCENT),
        );
        painter.text(
            pos2(stage.right(), y),
            Align2::RIGHT_CENTER,
            format!("{:02} / {:02} с", elapsed as u32, duration as u32),
            verstka::mono(11.0),
            verstka::SECONDARY,
        );
        verstka::rule(painter, stage.left(), stage.right(), stage.bottom() + 27.0);
    }
}

#[cfg(windows)]
mod sound {
    use std::{os::windows::ffi::OsStrExt, path::Path, ptr::null_mut};

    use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};

    pub(super) fn play(filename: &str) {
        let Ok(mut path) = std::env::current_exe() else {
            return;
        };
        path.set_file_name(filename);
        if !path.is_file() {
            let bundled = Path::new("gui/mira/showcase/assets/animation").join(filename);
            let Ok(asset) = crate::animation::demo_executable(&bundled) else {
                return;
            };
            path = asset;
        }
        let path: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        // PlaySound copies the filename before returning in asynchronous mode.
        unsafe {
            PlaySoundW(
                path.as_ptr(),
                null_mut(),
                SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
            );
        }
    }

    pub(super) fn stop() {
        // A null sound stops playback started by this process.
        unsafe {
            PlaySoundW(std::ptr::null(), null_mut(), 0);
        }
    }
}

struct Frames(&'static [u8]);

impl Frames {
    fn word(&self, offset: usize) -> usize {
        u32::from_le_bytes(self.0[offset..offset + 4].try_into().unwrap()) as usize
    }

    fn rate(&self) -> f64 {
        self.word(0) as f64
    }

    fn count(&self) -> usize {
        self.word(4)
    }

    fn at(&self, elapsed: f64) -> usize {
        ((elapsed * self.rate()) as usize).min(self.count() - 1)
    }

    fn bytes(&self, frame: usize) -> &'static [u8] {
        let payload = 8 + (self.count() + 1) * 4;
        let entry = 8 + frame * 4;
        &self.0[payload + self.word(entry)..payload + self.word(entry + 4)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_has_valid_frames_and_holds_the_final_frame() {
        for cut in Category::Remotion.cuts() {
            let frames = cut.frames().unwrap();
            assert_eq!(frames.rate(), 24.0);
            assert_eq!(frames.at(0.0), 0);
            assert_eq!(frames.at(2.0), 48);
            assert_eq!(frames.at(120.0), frames.count() - 1);
            for frame in [0, frames.count() / 2, frames.count() - 1] {
                let image = image::load_from_memory_with_format(
                    frames.bytes(frame),
                    image::ImageFormat::Jpeg,
                )
                .expect("every frame in the preview must decode");
                assert_eq!((image.width(), image.height()), (768, 432));
            }
        }
    }

    #[test]
    fn new_studies_are_the_first_three_cuts() {
        let player = Infographic::default();
        let clips: Vec<_> = Category::Remotion.cuts().collect();
        assert_eq!(player.cut.title(), "Вес");
        assert_eq!(player.neighbor(&clips, 1).complexity(), Some(2));
        assert_eq!(player.neighbor(&clips, 2).complexity(), Some(4));
        assert_eq!(player.neighbor(&clips, -1).title(), "Короткая");
        for (cut, duration) in [
            (Cut::Weight, 4),
            (Cut::Relay, 5),
            (Cut::ListToMap, 8),
            (Cut::NewFocus, 5),
            (Cut::Sync, 6),
        ] {
            assert_eq!(cut.frames().unwrap().count(), duration * 24);
            #[cfg(windows)]
            assert!(cut.audio().is_none());
        }
    }

    #[test]
    fn filters_include_spectre_and_remember_the_selected_clip() {
        assert_eq!(Category::All.cuts().count(), 8);
        assert_eq!(Category::Remotion.cuts().count(), 7);
        assert!(Category::Html.cuts().eq([Cut::Spectre]));
        assert_eq!(Category::Python.cuts().count(), 0);

        let mut player = Infographic::default();
        player.select(Cut::Relay, 1.0);
        player.change_category(Category::Remotion, 2.0);
        assert_eq!(player.started_at, 1.0);
        player.change_category(Category::Html, 3.0);
        assert_eq!(player.cut, Cut::Spectre);
        assert!(player.cut.frames().is_none());
        let ctx = egui::Context::default();
        crate::install_fonts(&ctx);
        let input = egui::RawInput {
            time: Some(3.1),
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(700.0, 410.0))),
            ..Default::default()
        };
        ctx.run_ui(input, |ui| player.show(ui, 22.0))
            .textures_delta
            .clear();
        assert!(player.html_preview.is_some());

        player.change_category(Category::Python, 4.0);
        player.change_category(Category::Remotion, 5.0);
        assert_eq!(player.cut, Cut::Relay);
        assert_eq!(player.started_at, 5.0);
    }

    #[test]
    fn editorial_navigation_switches_and_restarts_scenes() {
        let ctx = egui::Context::default();
        crate::install_fonts(&ctx);
        let mut player = Infographic::default();
        let draw = |player: &mut Infographic, time: f64, events: Vec<egui::Event>| {
            let input = egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    vec2(700.0, 410.0),
                )),
                events,
                ..Default::default()
            };
            ctx.run_ui(input, |ui| player.show(ui, 22.0))
                .textures_delta
                .clear();
        };
        let arrow = |pressed| egui::Event::Key {
            key: Key::ArrowRight,
            physical_key: Some(Key::ArrowRight),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        draw(&mut player, 0.0, vec![]);
        assert_eq!(player.cut.title(), "Вес");
        draw(&mut player, 1.0, vec![arrow(true), arrow(false)]);
        assert_eq!(player.cut.title(), "Передача");
        assert_eq!(player.current_frame, Some(0));
        draw(&mut player, 2.0, vec![arrow(true), arrow(false)]);
        assert_eq!(player.cut.title(), "Из списка в карту");
        assert_eq!(player.current_frame, Some(0));
        let row = pos2(550.0, 250.0);
        let click = |pressed| egui::Event::PointerButton {
            pos: row,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        draw(
            &mut player,
            3.0,
            vec![egui::Event::PointerMoved(row), click(true)],
        );
        draw(&mut player, 3.1, vec![click(false)]);
        assert_eq!(player.cut.title(), "Настройка");
        assert_eq!(player.current_frame, Some(0));
        draw(
            &mut player,
            4.0,
            vec![egui::Event::Key {
                key: Key::Space,
                physical_key: Some(Key::Space),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(player.cut.title(), "Настройка");
        assert_eq!(player.current_frame, Some(0));
    }
}
