#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use avilon_actions::palette::{Direction, Entry, Palette};
use avilon_egui::{
    mira::{self, Theme},
    settings_layout::SettingsRow,
};
use eframe::egui::{self, FontFamily, Key, Modifiers, RichText};

const COMMANDS: [Entry<'static>; 4] = [
    Entry {
        id: "general",
        label: "Общие настройки",
        group: "Раздел",
        search_terms: "запуск очередь",
        hidden_when_empty: false,
    },
    Entry {
        id: "appearance",
        label: "Оформление",
        group: "Раздел",
        search_terms: "тема мира верстка люмен",
        hidden_when_empty: false,
    },
    Entry {
        id: "apply",
        label: "Применить изменения",
        group: "Настройки",
        search_terms: "сохранить",
        hidden_when_empty: false,
    },
    Entry {
        id: "revert",
        label: "Вернуть применённые значения",
        group: "Настройки",
        search_terms: "отмена сброс",
        hidden_when_empty: false,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Preferences {
    jobs: u8,
    auto_start: bool,
    theme: Theme,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            jobs: 3,
            auto_start: false,
            theme: Theme::Verstka,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Page {
    #[default]
    General,
    Appearance,
}

#[derive(Default)]
struct Settings {
    compact: bool,
    page: Page,
    applied: Preferences,
    draft: Preferences,
    show_commands: bool,
    focus_search: bool,
    query: String,
    palette: Palette,
}

impl Settings {
    fn execute(&mut self, id: &str) {
        match id {
            "general" => self.page = Page::General,
            "appearance" => self.page = Page::Appearance,
            "apply" => self.applied = self.draft,
            "revert" => self.draft = self.applied,
            _ => {}
        }
        self.show_commands = false;
        self.query.clear();
        self.palette.set_query("");
    }

    fn command_list(&mut self, ui: &mut egui::Ui) {
        let search = egui::Id::new("settings-command-search");
        if std::mem::take(&mut self.focus_search) {
            ui.memory_mut(|memory| memory.request_focus(search));
        }
        let forward = ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowDown));
        let backward = ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowUp));
        let activate = ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter));
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .id(search)
                .desired_width(ui.available_width())
                .hint_text("Найти команду…"),
        );
        if response.changed() {
            self.palette.set_query(&self.query);
        }
        let visible = self.palette.visible(&COMMANDS);
        if forward {
            self.palette.step(visible.len(), Direction::Next);
        }
        if backward {
            self.palette.step(visible.len(), Direction::Previous);
        }
        let selected = self.palette.selected_id(&COMMANDS, &visible);
        let mut chosen = if activate { selected } else { None };
        for index in visible {
            let entry = &COMMANDS[index];
            if ui
                .selectable_label(Some(entry.id) == selected, entry.label)
                .clicked()
            {
                chosen = Some(entry.id);
            }
        }
        if let Some(id) = chosen {
            self.execute(id);
        }
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        for (page, label) in [(Page::General, "Общие"), (Page::Appearance, "Оформление")]
        {
            if ui.selectable_label(self.page == page, label).clicked() {
                self.page = page;
            }
        }
        if ui.button("Действия · Ctrl+K").clicked() {
            self.show_commands = !self.show_commands;
            self.focus_search = self.show_commands;
        }
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        if self.show_commands {
            ui.heading("Действия");
            self.command_list(ui);
            ui.separator();
        }
        match self.page {
            Page::General => {
                ui.heading("Общие");
                ui.add_space(6.0);
                SettingsRow::show(
                    ui,
                    "Параллельные задачи",
                    "Сколько операций утилита выполняет одновременно.",
                    |ui| {
                        ui.add(egui::Slider::new(&mut self.draft.jobs, 1..=12));
                    },
                );
                ui.separator();
                SettingsRow::show(
                    ui,
                    "Очередь",
                    "Запуск новых задач после добавления в очередь.",
                    |ui| {
                        ui.checkbox(&mut self.draft.auto_start, "Запускать автоматически");
                    },
                );
                ui.separator();
            }
            Page::Appearance => {
                ui.heading("Оформление");
                ui.add_space(6.0);
                SettingsRow::show(
                    ui,
                    "Тема окна",
                    "Применяется сразу к этому окну. «Вернуть» отменяет выбор.",
                    |ui| {
                        egui::ComboBox::from_id_salt("mira-preference-theme")
                            .selected_text(self.draft.theme.title())
                            .show_ui(ui, |ui| {
                                for theme in Theme::ALL {
                                    ui.selectable_value(
                                        &mut self.draft.theme,
                                        theme,
                                        theme.title(),
                                    );
                                }
                            });
                    },
                );
                ui.separator();
            }
        }
        ui.horizontal(|ui| {
            let dirty = self.draft != self.applied;
            if ui
                .add_enabled(dirty, egui::Button::new("Применить"))
                .clicked()
            {
                self.applied = self.draft;
            }
            if ui
                .add_enabled(dirty, egui::Button::new("Вернуть"))
                .clicked()
            {
                self.draft = self.applied;
            }
        });
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "Применено: задач одновременно — {}, автозапуск — {}, тема — {}",
                self.applied.jobs,
                if self.applied.auto_start {
                    "да"
                } else {
                    "нет"
                },
                self.applied.theme.title(),
            ))
            .weak(),
        );
    }
}

impl eframe::App for Settings {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.input_mut(|input| input.consume_key(Modifiers::CTRL, Key::K)) && !self.compact {
            self.show_commands = !self.show_commands;
            self.focus_search = self.show_commands;
        }
        if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
            self.show_commands = false;
        }
        if self.compact {
            mira::launcher(ui, Theme::Lumen, "Команды утилиты", |ui| {
                self.command_list(ui);
                ui.label(match self.page {
                    Page::General => "Текущий раздел: общие настройки",
                    Page::Appearance => "Текущий раздел: оформление",
                });
                ui.label(format!(
                    "Применено: {} задач одновременно",
                    self.applied.jobs
                ));
            });
        } else {
            let theme = self.draft.theme;
            let family = if theme == Theme::Verstka {
                editorial(ui.ctx())
            } else {
                FontFamily::Proportional
            };
            mira::workspace(
                ui,
                theme,
                "Настройки утилиты",
                family,
                self,
                |ui, host| host.header(ui),
                |ui, host| {
                    egui::ScrollArea::vertical().show(ui, |ui| host.body(ui));
                },
            );
        }
    }
}

fn editorial(ctx: &egui::Context) -> FontFamily {
    let family = FontFamily::Name("mira-editorial".into());
    if ctx.fonts(|fonts| fonts.families().contains(&family)) {
        family
    } else {
        FontFamily::Proportional
    }
}

fn install_editorial(ctx: &egui::Context) {
    let Some(windir) = std::env::var_os("WINDIR") else {
        return;
    };
    let path = std::path::PathBuf::from(windir)
        .join("Fonts")
        .join("georgia.ttf");
    if let Ok(bytes) = std::fs::read(path) {
        let mut fonts = egui::FontDefinitions::default();
        let mut fallback = fonts.families[&FontFamily::Proportional].clone();
        fallback.insert(0, "mira-georgia".into());
        fonts.font_data.insert(
            "mira-georgia".into(),
            egui::FontData::from_owned(bytes).into(),
        );
        fonts
            .families
            .insert(FontFamily::Name("mira-editorial".into()), fallback);
        ctx.set_fonts(fonts);
    }
}

fn main() -> eframe::Result {
    let compact = std::env::args_os().any(|arg| arg == "--compact");
    let (title, size) = if compact {
        ("Avilon · Люмен", [570.0, 350.0])
    } else {
        ("Avilon · Настройки утилиты", [910.0, 630.0])
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size([420.0, 310.0]),
        ..Default::default()
    };
    eframe::run_native(
        title,
        options,
        Box::new(move |cc| {
            install_editorial(&cc.egui_ctx);
            Ok(Box::new(Settings {
                compact,
                focus_search: compact,
                ..Default::default()
            }))
        }),
    )
}
