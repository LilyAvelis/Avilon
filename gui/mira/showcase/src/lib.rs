use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Id, Key, RichText};

mod animation;
mod atlas;
mod azimuth;
mod contact;
mod graphics;
mod herbarium;
mod kalka;
mod lozha;
mod lumen;
mod nolox;
mod orbit;
mod overlays;
mod pult;
mod semaphore;
mod verstka;

pub use animation::run_animation;
pub use atlas::run_atlas;
pub use graphics::run_graphics;
pub use nolox::run_nolox;

const SERIF: &str = "editorial";

#[derive(Clone, Copy)]
pub enum Edition {
    Lumen,
    Verstka,
    Contact,
    Azimuth,
    Herbarium,
    Pult,
    Orbit,
    Lozha,
    Kalka,
    Semaphore,
}

impl Edition {
    fn docks_actions(self) -> bool {
        matches!(self, Self::Verstka | Self::Herbarium)
    }
}

struct Work {
    title: &'static str,
    category: &'static str,
    place: &'static str,
    description: &'static str,
    year: u16,
}

const WORKS: [Work; 6] = [
    Work {
        title: "Базальтовый берег",
        category: "Пейзаж",
        place: "Вик",
        description: "Туман между водой и камнем. Белая пена на чёрном берегу.",
        year: 2025,
    },
    Work {
        title: "Рыжий фасад",
        category: "Архитектура",
        place: "Марсель",
        description: "Рыжая плоскость стены, длинная тень и один кипарис.",
        year: 2024,
    },
    Work {
        title: "Последний трамвай",
        category: "Город",
        place: "Лиссабон",
        description: "Пустая остановка в синий час. В лужах остаётся свет фонаря.",
        year: 2025,
    },
    Work {
        title: "Солнечная лестница",
        category: "Архитектура",
        place: "Милан",
        description: "Полоса послеполуденного света пересекает бетонную спираль.",
        year: 2025,
    },
    Work {
        title: "Оранжерея",
        category: "Ботаника",
        place: "Кью",
        description: "Папоротники и влажное стекло в утренней тишине.",
        year: 2024,
    },
    Work {
        title: "Розовая соль",
        category: "Пейзаж",
        place: "Уюни",
        description: "Первые лучи на солончаке, где небо отражается до горизонта.",
        year: 2025,
    },
];

const PHOTOS: [&[u8]; WORKS.len()] = [
    include_bytes!("../assets/coast.jpg"),
    include_bytes!("../assets/facade.jpg"),
    include_bytes!("../assets/tram.jpg"),
    include_bytes!("../assets/stairs.jpg"),
    include_bytes!("../assets/greenhouse.jpg"),
    include_bytes!("../assets/salt.jpg"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlay {
    Actions,
    Preview,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Preview,
    Mark,
    Undo,
    Settings,
}

struct App {
    edition: Edition,
    query: String,
    selected: Option<usize>,
    marked: [bool; WORKS.len()],
    textures: Vec<egui::TextureHandle>,
    compact: bool,
    comparing: bool,
    azimuth_section: usize,
    herbarium: herbarium::State,
    pult: pult::State,
    orbit: orbit::State,
    lozha_light: bool,
    kalka: kalka::State,
    semaphore_marked: bool,
    search_terms: [String; WORKS.len()],
    overlay: Option<Overlay>,
    focus_search: bool,
    focus_action: bool,
    action_query: String,
    action_index: usize,
    history: Vec<usize>,
    status: Option<(&'static str, f64)>,
}

fn search_id() -> Id {
    Id::new("collection-search")
}

fn normalize(text: &str) -> String {
    text.to_lowercase().replace('ё', "е")
}

fn consume_key(ctx: &egui::Context, key: Key, ctrl: bool) -> usize {
    ctx.input_mut(|input| {
        let mut count = 0;
        input.events.retain(|event| {
            let found = matches!(
                event,
                egui::Event::Key {
                    key: logical,
                    physical_key,
                    pressed: true,
                    modifiers,
                    repeat,
                } if modifiers.ctrl == ctrl
                    && !modifiers.alt && !modifiers.shift && !modifiers.mac_cmd
                    && (*logical == key || ctrl && *physical_key == Some(key))
                    && (!repeat || matches!(key, Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight))
            );
            count += usize::from(found);
            !found
        });
        count
    })
}

impl App {
    fn new(ctx: &egui::Context, edition: Edition) -> Self {
        install_fonts(ctx);
        match edition {
            Edition::Verstka
            | Edition::Herbarium
            | Edition::Pult
            | Edition::Kalka
            | Edition::Semaphore => {
                ctx.set_visuals(egui::Visuals::light());
            }
            _ => ctx.set_visuals(egui::Visuals::dark()),
        }
        let textures = PHOTOS
            .iter()
            .zip(WORKS.iter())
            .map(|(bytes, work)| {
                let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Jpeg)
                    .expect("embedded collection image must be a valid JPEG")
                    .into_rgba8();
                let size = [image.width() as usize, image.height() as usize];
                ctx.load_texture(
                    work.title,
                    egui::ColorImage::from_rgba_unmultiplied(size, &image),
                    egui::TextureOptions::LINEAR,
                )
            })
            .collect();
        Self {
            edition,
            textures,
            query: String::new(),
            selected: Some(0),
            marked: [false; WORKS.len()],
            compact: false,
            comparing: false,
            azimuth_section: 0,
            herbarium: herbarium::State::default(),
            pult: pult::State::default(),
            orbit: orbit::State::default(),
            lozha_light: false,
            kalka: kalka::State::default(),
            semaphore_marked: false,
            search_terms: WORKS.map(|work| {
                normalize(&format!(
                    "{} {} {} {}",
                    work.title, work.place, work.category, work.year
                ))
            }),
            overlay: None,
            focus_search: true,
            focus_action: false,
            action_query: String::new(),
            action_index: 0,
            history: Vec::new(),
            status: None,
        }
    }

    fn matches(&self) -> Vec<usize> {
        let query = normalize(self.query.trim());
        let category = match self.edition {
            Edition::Azimuth => azimuth::SECTIONS[self.azimuth_section],
            _ => None,
        };
        self.search_terms
            .iter()
            .enumerate()
            .filter_map(|(index, haystack)| {
                let category_matches = category.is_none_or(|name| WORKS[index].category == name);
                let text_matches = query.split_whitespace().all(|word| {
                    haystack.contains(word)
                        || self.marked[index] && "избранное отмечено".contains(word)
                });
                (category_matches && text_matches && (!self.semaphore_marked || self.marked[index]))
                    .then_some(index)
            })
            .collect()
    }

    fn sync_selection(&mut self) {
        let visible = self.matches();
        if !self.selected.is_some_and(|index| visible.contains(&index)) {
            self.selected = visible.first().copied();
        }
    }

    fn search(&mut self, ui: &mut egui::Ui, width: f32) -> egui::Response {
        if self.overlay.is_none() && self.focus_search {
            if ui.memory(|memory| memory.is_above_modal_layer(ui.layer_id())) {
                self.focus_search = false;
                ui.memory_mut(|memory| memory.request_focus(search_id()));
            } else {
                ui.ctx().request_repaint();
            }
        }
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .id(search_id())
                .desired_width(width.max(0.0))
                .hint_text("Поиск по коллекции…"),
        );
        if response.changed() {
            self.sync_selection();
        }
        response
    }

    fn select(&mut self, index: usize) {
        self.selected = Some(index);
    }

    fn move_selection(&mut self, offset: isize) {
        let results = self.matches();
        self.selected = results
            .iter()
            .position(|index| Some(*index) == self.selected)
            .map(|position| {
                position
                    .saturating_add_signed(offset)
                    .min(results.len() - 1)
            })
            .and_then(|position| results.get(position).copied())
            .or_else(|| results.first().copied());
    }

    fn open_preview(&mut self) {
        if self.selected.is_some() {
            self.orbit.close_radial();
            self.overlay = Some(Overlay::Preview);
        }
    }

    fn open_actions(&mut self) {
        self.orbit.close_radial();
        if matches!(self.edition, Edition::Orbit) {
            self.orbit.open_center(self.selected.is_some());
        }
        self.action_query.clear();
        self.action_index = 0;
        self.focus_action = true;
        self.overlay = Some(Overlay::Actions);
    }

    fn open_settings(&mut self) {
        self.orbit.close_radial();
        self.overlay = Some(Overlay::Settings);
    }

    fn close_overlay(&mut self) {
        self.orbit.close_radial();
        self.overlay = None;
        self.focus_search = true;
    }

    fn action_label(&self, action: Action) -> &'static str {
        match action {
            Action::Preview => "Открыть просмотр",
            Action::Mark if self.selected.is_some_and(|index| self.marked[index]) => {
                "Снять отметку"
            }
            Action::Mark => "Отметить работу",
            Action::Undo => "Отменить последнюю отметку",
            Action::Settings => "Открыть настройки",
        }
    }

    fn actions(&self) -> Vec<Action> {
        let query = normalize(&self.action_query);
        [
            Action::Preview,
            Action::Mark,
            Action::Undo,
            Action::Settings,
        ]
        .into_iter()
        .filter(|action| match action {
            Action::Preview | Action::Mark => self.selected.is_some(),
            Action::Undo => !self.history.is_empty(),
            Action::Settings => true,
        })
        .filter(|action| {
            let label = normalize(self.action_label(*action));
            query.split_whitespace().all(|word| label.contains(word))
        })
        .collect()
    }

    fn notify(&mut self, ctx: &egui::Context, message: &'static str) {
        self.status = Some((message, ctx.input(|input| input.time) + 2.0));
        ctx.request_repaint_after(std::time::Duration::from_secs(2));
    }

    fn undo(&mut self, ctx: &egui::Context) {
        if let Some(index) = self.history.pop() {
            self.marked[index] = !self.marked[index];
            self.sync_selection();
            if self.selected.is_none() && self.overlay == Some(Overlay::Preview) {
                self.close_overlay();
            }
            self.notify(ctx, "Отметка отменена");
        }
    }

    fn execute(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::Preview => self.open_preview(),
            Action::Settings => self.open_settings(),
            Action::Undo => {
                self.undo(ctx);
                self.close_overlay();
            }
            Action::Mark => {
                if let Some(index) = self.selected {
                    self.marked[index] = !self.marked[index];
                    self.history.push(index);
                    self.notify(
                        ctx,
                        if self.marked[index] {
                            "Работа отмечена"
                        } else {
                            "Отметка снята"
                        },
                    );
                    self.sync_selection();
                }
                self.close_overlay();
            }
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if consume_key(ctx, Key::Comma, true) > 0 {
            if self.overlay == Some(Overlay::Settings) {
                self.close_overlay();
            } else {
                self.open_settings();
            }
            return;
        }
        if consume_key(ctx, Key::K, true) > 0 {
            if self.overlay == Some(Overlay::Actions) || self.orbit.radial_open() {
                self.close_overlay();
            } else {
                self.open_actions();
            }
            return;
        }
        if consume_key(ctx, Key::F, true) > 0 {
            self.close_overlay();
            return;
        }
        if consume_key(ctx, Key::Z, true) > 0 {
            self.undo(ctx);
            return;
        }
        if consume_key(ctx, Key::Escape, false) > 0 {
            if self.orbit.radial_open() || self.overlay.is_some() {
                self.close_overlay();
            } else {
                self.query.clear();
                self.sync_selection();
                self.focus_search = true;
            }
            return;
        }

        if self.orbit.radial_open() {
            let next = consume_key(ctx, Key::ArrowRight, false);
            let prev = consume_key(ctx, Key::ArrowLeft, false);
            self.orbit.step(
                next as isize - prev as isize,
                self.selected.is_some(),
                !self.history.is_empty(),
            );
            if consume_key(ctx, Key::Enter, false) > 0 {
                self.execute(ctx, self.orbit.action());
            }
            return;
        }
        match self.overlay {
            Some(Overlay::Settings) => {}
            Some(Overlay::Preview) => {
                let next = consume_key(ctx, Key::ArrowRight, false)
                    + consume_key(ctx, Key::ArrowDown, false);
                let prev =
                    consume_key(ctx, Key::ArrowLeft, false) + consume_key(ctx, Key::ArrowUp, false);
                self.move_selection(next as isize - prev as isize);
            }
            Some(Overlay::Actions) => {
                if matches!(
                    self.edition,
                    Edition::Pult | Edition::Lozha | Edition::Kalka | Edition::Semaphore
                ) {
                    let (forward, back) = match self.edition {
                        Edition::Pult | Edition::Semaphore => (Key::ArrowRight, Key::ArrowLeft),
                        _ => (Key::ArrowDown, Key::ArrowUp),
                    };
                    let next = consume_key(ctx, forward, false);
                    let prev = consume_key(ctx, back, false);
                    let actions = self.actions();
                    self.action_index = (self.action_index as isize + next as isize - prev as isize)
                        .rem_euclid(actions.len() as isize)
                        as usize;
                    if consume_key(ctx, Key::Enter, false) > 0 {
                        self.execute(ctx, actions[self.action_index]);
                    }
                    return;
                }
                let down = consume_key(ctx, Key::ArrowDown, false) as isize;
                let up = consume_key(ctx, Key::ArrowUp, false) as isize;
                let actions = self.actions();
                self.action_index = self
                    .action_index
                    .saturating_add_signed(down - up)
                    .min(actions.len().saturating_sub(1));
                let search_focused = ctx.memory(|mem| {
                    mem.focused()
                        .is_none_or(|id| id == overlays::action_search_id())
                });
                if search_focused
                    && consume_key(ctx, Key::Enter, false) > 0
                    && let Some(&action) = actions.get(self.action_index)
                {
                    self.execute(ctx, action);
                }
            }
            None => {
                let down = consume_key(ctx, Key::ArrowDown, false) as isize;
                let up = consume_key(ctx, Key::ArrowUp, false) as isize;
                if up != down {
                    self.move_selection(down - up);
                    self.focus_search = true;
                }
                let search_focused =
                    ctx.memory(|mem| mem.focused().is_none_or(|id| id == search_id()));
                if matches!(self.edition, Edition::Orbit | Edition::Semaphore)
                    && (self.query.is_empty() || !search_focused)
                {
                    let right = consume_key(ctx, Key::ArrowRight, false);
                    let left = consume_key(ctx, Key::ArrowLeft, false);
                    if right != left {
                        self.move_selection(right as isize - left as isize);
                    }
                }
                if matches!(self.edition, Edition::Azimuth) && self.query.is_empty() {
                    let right = consume_key(ctx, Key::ArrowRight, false) as isize;
                    let left = consume_key(ctx, Key::ArrowLeft, false) as isize;
                    if right != left {
                        self.azimuth_section = self
                            .azimuth_section
                            .saturating_add_signed(right - left)
                            .min(azimuth::SECTIONS.len() - 1);
                        self.sync_selection();
                    }
                }
                if search_focused && consume_key(ctx, Key::Enter, false) > 0 {
                    self.open_preview();
                }
            }
        }
    }
}

impl App {
    fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.shortcuts(&ctx);
        if self.edition.docks_actions() && self.overlay == Some(Overlay::Actions) {
            self.show_docked_actions(ui);
        }
        match self.edition {
            Edition::Lumen => lumen::show(ui, self),
            Edition::Verstka => verstka::show(ui, self),
            Edition::Contact => contact::show(ui, self),
            Edition::Azimuth => azimuth::show(ui, self),
            Edition::Herbarium => herbarium::show(ui, self),
            Edition::Pult => pult::show(ui, self),
            Edition::Orbit => orbit::show(ui, self),
            Edition::Lozha => lozha::show(ui, self),
            Edition::Kalka => kalka::show(ui, self),
            Edition::Semaphore => semaphore::show(ui, self),
        }
        self.show_overlay(&ctx);
        if let Some((message, expires)) = self.status
            && ctx.input(|input| input.time) < expires
        {
            egui::Area::new(Id::new("feedback"))
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -58.0])
                .order(egui::Order::Tooltip)
                .interactable(false)
                .show(&ctx, |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(34, 42, 43))
                        .corner_radius(8.0)
                        .inner_margin(9)
                        .show(ui, |ui| {
                            ui.label(RichText::new(message).color(Color32::WHITE));
                        });
                });
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

fn install_fonts(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
}

fn font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let serif = FontFamily::Name(SERIF.into());
    let mut serif_fonts = fonts.families[&FontFamily::Proportional].clone();
    if let Some(windows) = std::env::var_os("WINDIR") {
        let directory = std::path::PathBuf::from(windows).join("Fonts");
        if let Ok(bytes) = std::fs::read(directory.join("georgia.ttf")) {
            fonts
                .font_data
                .insert("georgia".into(), FontData::from_owned(bytes).into());
            serif_fonts.insert(0, "georgia".into());
        }
        if let Ok(bytes) = std::fs::read(directory.join("segoeui.ttf")) {
            fonts
                .font_data
                .insert("segoe".into(), FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "segoe".into());
        }
    }
    fonts.families.insert(serif, serif_fonts);
    fonts
}

pub fn run(edition: Edition) -> eframe::Result {
    let (title, size, minimum) = match edition {
        Edition::Lumen => ("Люмен · Мира", [760.0, 545.0], [660.0, 480.0]),
        Edition::Verstka => ("Вёрстка · Мира · проход 2", [1080.0, 760.0], [780.0, 570.0]),
        Edition::Contact => ("Контакт · Мира", [960.0, 720.0], [740.0, 550.0]),
        Edition::Azimuth => ("Азимут · Мира", [900.0, 650.0], [740.0, 530.0]),
        Edition::Herbarium => ("Гербарий · Мира", [1080.0, 760.0], [800.0, 570.0]),
        Edition::Pult => (
            "Пульт · Мира · Приборные клавиши",
            [1020.0, 700.0],
            [800.0, 570.0],
        ),
        Edition::Orbit => (
            "Орбита · Мира · Круг действий",
            [1080.0, 740.0],
            [800.0, 570.0],
        ),
        Edition::Lozha => ("Ложа · Мира", [1130.0, 760.0], [800.0, 590.0]),
        Edition::Kalka => ("Калька · Мира", [1080.0, 740.0], [800.0, 590.0]),
        Edition::Semaphore => ("Семафор · Мира", [1080.0, 720.0], [800.0, 590.0]),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size(minimum),
        ..Default::default()
    };
    eframe::run_native(
        title,
        options,
        Box::new(move |cc| Ok(Box::new(App::new(&cc.egui_ctx, edition)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &egui::Context, app: &mut App, events: Vec<egui::Event>) {
        let size = match app.edition {
            Edition::Herbarium | Edition::Pult | Edition::Orbit => egui::vec2(800.0, 570.0),
            Edition::Lozha | Edition::Kalka | Edition::Semaphore => egui::vec2(800.0, 590.0),
            _ => egui::vec2(1050.0, 750.0),
        };
        let events = events
            .into_iter()
            .flat_map(|event| {
                let mut released = event.clone();
                if let egui::Event::Key { pressed, .. } = &mut released {
                    *pressed = false;
                    [Some(event), Some(released)]
                } else {
                    [Some(event), None]
                }
            })
            .flatten()
            .collect();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            events,
            ..Default::default()
        };
        ctx.run_ui(input, |ui| app.show(ui)).textures_delta.clear();
    }

    fn key(key: Key, ctrl: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers: if ctrl {
                egui::Modifiers::CTRL
            } else {
                egui::Modifiers::NONE
            },
        }
    }

    #[test]
    fn editions_support_the_same_search_action_and_undo_flow() {
        for edition in [
            Edition::Lumen,
            Edition::Verstka,
            Edition::Contact,
            Edition::Azimuth,
            Edition::Herbarium,
            Edition::Pult,
            Edition::Orbit,
            Edition::Lozha,
            Edition::Kalka,
            Edition::Semaphore,
        ] {
            let ctx = egui::Context::default();
            let mut app = App::new(&ctx, edition);
            frame(&ctx, &mut app, vec![]);
            assert!(ctx.memory(|memory| memory.has_focus(search_id())));

            frame(
                &ctx,
                &mut app,
                vec![egui::Event::Text("архитектура".to_owned())],
            );
            assert_eq!(app.matches(), [1, 3]);
            assert_eq!(app.selected, Some(1));
            frame(&ctx, &mut app, vec![key(Key::ArrowDown, false)]);
            assert_eq!(app.selected, Some(3));

            frame(&ctx, &mut app, vec![key(Key::Enter, false)]);
            assert!(app.overlay == Some(Overlay::Preview));
            frame(&ctx, &mut app, vec![key(Key::ArrowLeft, false)]);
            assert_eq!(app.selected, Some(1));
            frame(&ctx, &mut app, vec![key(Key::Escape, false)]);
            assert!(app.overlay.is_none());
            assert_eq!(app.query, "архитектура");
            frame(&ctx, &mut app, vec![]);
            assert!(ctx.memory(|memory| memory.has_focus(search_id())));

            frame(&ctx, &mut app, vec![key(Key::K, true)]);
            assert!(app.overlay == Some(Overlay::Actions));
            if matches!(edition, Edition::Orbit | Edition::Pult | Edition::Semaphore) {
                frame(&ctx, &mut app, vec![key(Key::ArrowRight, false)]);
            } else if matches!(edition, Edition::Lozha | Edition::Kalka) {
                frame(&ctx, &mut app, vec![key(Key::ArrowDown, false)]);
            } else {
                assert!(ctx.memory(|memory| memory.has_focus(overlays::action_search_id())));
                frame(
                    &ctx,
                    &mut app,
                    vec![egui::Event::Text("отметить".to_owned())],
                );
                assert!(app.actions() == [Action::Mark]);
            }
            frame(&ctx, &mut app, vec![key(Key::Enter, false)]);
            assert!(
                app.marked[1],
                "selection: {:?}, still in actions: {}",
                app.selected,
                app.overlay == Some(Overlay::Actions)
            );
            frame(&ctx, &mut app, vec![key(Key::Z, true)]);
            assert!(!app.marked[1]);
            frame(&ctx, &mut app, vec![key(Key::Comma, true)]);
            assert!(app.overlay == Some(Overlay::Settings));
            frame(&ctx, &mut app, vec![key(Key::Escape, false)]);
            assert!(app.overlay.is_none());
            assert_eq!(app.selected, Some(1));
            assert_eq!(app.query, "архитектура");
        }
    }

    #[test]
    fn orbit_horizontal_navigation_moves_frames_and_keeps_the_filtered_query() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, Edition::Orbit);
        frame(&ctx, &mut app, vec![]);
        frame(&ctx, &mut app, vec![key(Key::ArrowRight, false)]);
        assert_eq!(app.selected, Some(1));
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("архитектура".to_owned())],
        );
        frame(&ctx, &mut app, vec![key(Key::ArrowRight, false)]);
        assert_eq!(app.selected, Some(1));
        frame(&ctx, &mut app, vec![key(Key::ArrowDown, false)]);
        assert_eq!(app.selected, Some(3));
        assert_eq!(app.query, "архитектура");
    }

    #[test]
    fn lozha_changes_lighting_without_discarding_selection_or_actions() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, Edition::Lozha);
        frame(&ctx, &mut app, vec![]);
        app.select(3);
        frame(&ctx, &mut app, vec![key(Key::K, true)]);
        let day = egui::pos2(745.0, 37.0);
        for pressed in [true, false] {
            frame(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(day),
                    egui::Event::PointerButton {
                        pos: day,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(app.lozha_light);
        assert_eq!(app.selected, Some(3));
        assert!(app.overlay == Some(Overlay::Actions));
    }

    #[test]
    fn semaphore_marked_route_tracks_the_action_and_its_undo() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, Edition::Semaphore);
        frame(&ctx, &mut app, vec![]);
        frame(&ctx, &mut app, vec![key(Key::ArrowRight, false)]);
        assert_eq!(app.selected, Some(1));
        frame(&ctx, &mut app, vec![key(Key::K, true)]);
        frame(&ctx, &mut app, vec![key(Key::ArrowRight, false)]);
        frame(&ctx, &mut app, vec![key(Key::Enter, false)]);

        let station_filter = egui::pos2(188.0, 448.0);
        for pressed in [true, false] {
            frame(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(station_filter),
                    egui::Event::PointerButton {
                        pos: station_filter,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(app.semaphore_marked);
        assert_eq!(app.matches(), [1]);
        assert_eq!(app.selected, Some(1));
        frame(&ctx, &mut app, vec![key(Key::Z, true)]);
        assert!(app.matches().is_empty());
        assert_eq!(app.selected, None);
    }

    #[test]
    fn azimuth_uses_horizontal_arrows_for_sections_with_an_empty_query() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx, Edition::Azimuth);
        frame(&ctx, &mut app, vec![]);
        frame(&ctx, &mut app, vec![key(Key::ArrowRight, false)]);
        assert_eq!(app.matches(), [0, 5]);
        frame(&ctx, &mut app, vec![key(Key::ArrowDown, false)]);
        assert_eq!(app.selected, Some(5));
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("розовая".to_owned())],
        );
        frame(&ctx, &mut app, vec![key(Key::ArrowLeft, false)]);
        assert_eq!(app.azimuth_section, 1);
        assert_eq!(app.matches(), [5]);
    }
}
