use eframe::egui::{
    self, Align, Button, Color32, FontId, Frame, Id, Layout, RichText, Stroke, Vec2,
};

use crate::{App, Edition, Overlay, WORKS};

pub(super) fn action_search_id() -> Id {
    Id::new("action-search")
}

fn keycap(ui: &mut egui::Ui, text: &str, edition: Edition) {
    if !matches!(edition, Edition::Lumen) {
        ui.label(RichText::new(text).size(11.0).monospace().strong());
        return;
    }
    Frame::new()
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(7, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).monospace());
        });
}

impl App {
    pub(super) fn show_docked_actions(&mut self, ui: &mut egui::Ui) {
        let (surface, line, ink, focus) = match self.edition {
            Edition::Herbarium => (
                [212, 222, 195],
                [135, 155, 115],
                [37, 71, 48],
                [182, 199, 154],
            ),
            _ => (
                [229, 219, 199],
                [183, 169, 144],
                [103, 40, 24],
                [229, 219, 199],
            ),
        };
        let [surface, line, ink, focus] =
            [surface, line, ink, focus].map(|[r, g, b]| Color32::from_rgb(r, g, b));
        egui::Panel::bottom("docked-actions")
            .resizable(false)
            .exact_size(101.0)
            .frame(Frame::new().fill(surface))
            .show(ui, |ui| {
                let panel = ui.max_rect();
                ui.painter()
                    .hline(panel.x_range(), panel.top(), Stroke::new(1.0, line));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(26.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(215.0, 81.0),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_width(205.0);
                            ui.spacing_mut().item_spacing.y = 3.0;
                            ui.label(RichText::new("Действия").strong());
                            self.action_search(ui);
                            ui.label(
                                RichText::new("↑ ↓  Выбрать   ·   Enter  Выполнить").size(10.0),
                            );
                        },
                    );
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        let actions = self.actions();
                        let width = ((ui.available_width() - 44.0) * 0.5).clamp(160.0, 270.0);
                        let mut chosen = None;
                        for (row, chunk) in actions.chunks(2).enumerate() {
                            ui.horizontal(|ui| {
                                for (column, action) in chunk.iter().copied().enumerate() {
                                    let active = row * 2 + column == self.action_index;
                                    let lead = if active { "›" } else { "·" };
                                    let label = format!("{lead}  {}", self.action_label(action));
                                    let text = RichText::new(label).color(ink);
                                    let rect = match self.edition {
                                        Edition::Verstka => Button::new(text).frame(false),
                                        _ => Button::new(text)
                                            .fill(if active { focus } else { surface })
                                            .stroke(Stroke::NONE)
                                            .corner_radius(18),
                                    };
                                    if ui.add_sized([width, 35.0], rect).clicked() {
                                        chosen = Some(action);
                                    }
                                }
                            });
                        }
                        if actions.is_empty() {
                            ui.label("Действия не найдены");
                        }
                        if let Some(action) = chosen {
                            self.execute(ui.ctx(), action);
                        }
                    });
                });
            });
    }

    pub(super) fn show_overlay(&mut self, ctx: &egui::Context) {
        let Some(overlay) = self.overlay else { return };
        if overlay == Overlay::Actions
            && (self.edition.docks_actions()
                || matches!(
                    self.edition,
                    Edition::Orbit
                        | Edition::Pult
                        | Edition::Lozha
                        | Edition::Kalka
                        | Edition::Semaphore
                ))
        {
            return;
        }
        let fill = match self.edition {
            Edition::Lumen => Color32::from_rgb(23, 28, 38),
            Edition::Verstka => Color32::from_rgb(249, 245, 236),
            Edition::Contact => Color32::from_rgb(41, 36, 30),
            Edition::Azimuth => Color32::from_rgb(18, 32, 85),
            Edition::Herbarium => Color32::from_rgb(242, 239, 215),
            Edition::Pult => Color32::from_rgb(230, 232, 223),
            Edition::Orbit => Color32::from_rgb(29, 24, 46),
            Edition::Lozha if self.lozha_light => Color32::from_rgb(246, 238, 218),
            Edition::Lozha => Color32::from_rgb(47, 30, 38),
            Edition::Kalka => Color32::from_rgb(235, 244, 241),
            Edition::Semaphore => Color32::from_rgb(248, 241, 224),
        };
        let border = match self.edition {
            Edition::Lumen => Color32::from_rgb(65, 78, 93),
            Edition::Verstka => Color32::from_rgb(191, 177, 156),
            Edition::Contact => Color32::from_rgb(148, 119, 77),
            Edition::Azimuth => Color32::from_rgb(113, 146, 255),
            Edition::Herbarium => Color32::from_rgb(140, 157, 115),
            Edition::Pult => Color32::from_rgb(134, 149, 124),
            Edition::Orbit => Color32::from_rgb(115, 95, 155),
            Edition::Lozha => Color32::from_rgb(160, 114, 67),
            Edition::Kalka => Color32::from_rgb(105, 151, 146),
            Edition::Semaphore => Color32::from_rgb(155, 79, 56),
        };
        let id = Id::new("mira-overlay");
        let radius = match self.edition {
            Edition::Lumen => 12,
            Edition::Contact | Edition::Orbit => 24,
            Edition::Verstka | Edition::Azimuth | Edition::Pult => 0,
            Edition::Herbarium => 8,
            Edition::Lozha => 4,
            Edition::Kalka | Edition::Semaphore => 0,
        };
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .frame(
                Frame::new()
                    .fill(fill)
                    .stroke(Stroke::new(1.0, border))
                    .corner_radius(radius)
                    .inner_margin(20),
            )
            .backdrop_color(Color32::from_black_alpha(146))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(10.0, 8.0);
                ui.visuals_mut().extreme_bg_color = match self.edition {
                    Edition::Lumen => Color32::from_rgb(16, 20, 29),
                    Edition::Verstka => Color32::from_rgb(255, 252, 247),
                    Edition::Contact => Color32::from_rgb(25, 23, 20),
                    Edition::Azimuth => Color32::from_rgb(11, 19, 55),
                    Edition::Herbarium => Color32::from_rgb(248, 245, 225),
                    Edition::Pult => Color32::from_rgb(241, 244, 235),
                    Edition::Orbit => Color32::from_rgb(20, 16, 32),
                    Edition::Lozha if self.lozha_light => Color32::from_rgb(250, 244, 232),
                    Edition::Lozha => Color32::from_rgb(41, 26, 34),
                    Edition::Kalka => Color32::from_rgb(250, 255, 252),
                    Edition::Semaphore => Color32::from_rgb(255, 251, 237),
                };
                if !matches!(self.edition, Edition::Lumen) {
                    let button = match self.edition {
                        Edition::Contact => Color32::from_rgb(87, 67, 43),
                        Edition::Azimuth => Color32::from_rgb(51, 75, 153),
                        Edition::Herbarium => Color32::from_rgb(213, 225, 187),
                        Edition::Pult => Color32::from_rgb(190, 207, 181),
                        Edition::Orbit => Color32::from_rgb(65, 52, 103),
                        Edition::Lozha if self.lozha_light => Color32::from_rgb(224, 209, 181),
                        Edition::Lozha => Color32::from_rgb(93, 56, 57),
                        Edition::Kalka => Color32::from_rgb(179, 216, 206),
                        Edition::Semaphore => Color32::from_rgb(236, 199, 175),
                        _ => Color32::from_rgb(235, 225, 206),
                    };
                    let widgets = &mut ui.visuals_mut().widgets;
                    for state in [
                        &mut widgets.inactive,
                        &mut widgets.active,
                        &mut widgets.hovered,
                    ] {
                        state.corner_radius = radius.into();
                        state.bg_stroke = Stroke::NONE;
                    }
                    widgets.hovered.weak_bg_fill = button;
                    widgets.active.weak_bg_fill = button;
                }
                let view = ctx.content_rect();
                let width = match overlay {
                    Overlay::Preview => (view.height() - 144.0).clamp(250.0, 560.0),
                    _ => 410.0,
                }
                .min(view.width() - 64.0);
                ui.set_width(width);
                match overlay {
                    Overlay::Actions => self.show_actions(ui),
                    Overlay::Settings => self.show_settings(ui),
                    Overlay::Preview => self.show_preview(ui),
                }
            });
        if modal.backdrop_response.clicked() {
            self.close_overlay();
        }
    }

    fn overlay_heading(&mut self, ui: &mut egui::Ui, title: &str) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).size(21.0).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Закрыть").on_hover_text("Esc").clicked() {
                    self.close_overlay();
                }
            });
        });
    }

    fn show_actions(&mut self, ui: &mut egui::Ui) {
        self.overlay_heading(ui, "Действия");
        if let Some(index) = self.selected {
            ui.label(RichText::new(WORKS[index].title).weak());
        }
        self.action_search(ui);
        let actions = self.actions();
        let mut chosen = None;
        for (index, action) in actions.iter().copied().enumerate() {
            let response = ui.add_sized(
                [ui.available_width(), 34.0],
                Button::selectable(index == self.action_index, self.action_label(action)),
            );
            if response.clicked() {
                chosen = Some(action);
            }
        }
        if actions.is_empty() {
            ui.label("Действия не найдены");
        }
        ui.separator();
        ui.horizontal(|ui| {
            keycap(ui, "↑ ↓", self.edition);
            ui.label("Выбрать");
            keycap(ui, "Enter", self.edition);
            ui.label("Выполнить");
        });
        if let Some(action) = chosen {
            self.execute(ui.ctx(), action);
        }
    }

    fn action_search(&mut self, ui: &mut egui::Ui) {
        if self.focus_action {
            if ui.memory(|memory| memory.is_above_modal_layer(ui.layer_id())) {
                self.focus_action = false;
                ui.memory_mut(|memory| memory.request_focus(action_search_id()));
            } else {
                ui.ctx().request_repaint();
            }
        }
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.action_query)
                .id(action_search_id())
                .font(FontId::proportional(16.0))
                .desired_width(ui.available_width())
                .hint_text("Найти действие…"),
        );
        if response.changed() {
            self.action_index = 0;
        }
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        self.overlay_heading(ui, "Настройки");
        ui.checkbox(&mut self.compact, "Компактный список");
        ui.separator();
        let (vertical, horizontal) = match self.edition {
            Edition::Orbit => (
                "Выбрать кадр при поиске",
                "Листать кадры или выбрать сектор",
            ),
            Edition::Pult => (
                "Выбрать кадр",
                "Листать кадры в просмотре или выбрать клавишу",
            ),
            Edition::Semaphore => (
                "Выбрать кадр при поиске",
                "Листать маршрут или выбрать команду",
            ),
            _ => ("Выбрать работу или действие", "Листать работы в просмотре"),
        };
        egui::Grid::new("shared-shortcuts")
            .spacing([18.0, 5.0])
            .show(ui, |ui| {
                for (keys, label) in [
                    ("Ctrl+F", "Поиск по коллекции"),
                    ("↑ ↓", vertical),
                    ("Enter", "Открыть просмотр или выполнить действие"),
                    ("← →", horizontal),
                    ("Ctrl+K", "Действия"),
                    ("Ctrl+,", "Настройки"),
                    ("Ctrl+Z", "Отменить последнюю отметку"),
                    ("Esc", "Вернуться или очистить поиск"),
                ] {
                    keycap(ui, keys, self.edition);
                    ui.label(label);
                    ui.end_row();
                }
            });
    }

    fn show_preview(&mut self, ui: &mut egui::Ui) {
        let Some(index) = self.selected else { return };
        let work = &WORKS[index];
        self.overlay_heading(ui, work.title);
        let width = ui.available_width();
        ui.add(
            egui::Image::new(&self.textures[index])
                .fit_to_exact_size(Vec2::splat(width))
                .corner_radius(4),
        );
        ui.horizontal(|ui| {
            if ui.button("←").on_hover_text("Предыдущая работа").clicked() {
                self.move_selection(-1);
            }
            if ui.button("→").on_hover_text("Следующая работа").clicked() {
                self.move_selection(1);
            }
            ui.label(format!(
                "{} · {} · {}",
                work.place, work.category, work.year
            ));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Действия").on_hover_text("Ctrl+K").clicked() {
                    self.open_actions();
                }
            });
        });
    }
}
