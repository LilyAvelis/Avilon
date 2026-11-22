//! General-purpose egui presets and a responsive desktop shell from the Mira atlas.
//! Full layouts and their distinct gestures are demonstrated in gui/mira/showcase.

use egui::{
    self, Align, Color32, FontFamily, FontId, Frame, Layout, RichText, Stroke, Ui, Visuals,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Verstka,
    Lumen,
    Contact,
    Azimuth,
    Herbarium,
    Pult,
    Orbit,
    Lozha,
    Kalka,
    Semaphore,
}

impl Theme {
    pub const ALL: [Self; 10] = [
        Self::Verstka,
        Self::Lumen,
        Self::Contact,
        Self::Azimuth,
        Self::Herbarium,
        Self::Pult,
        Self::Orbit,
        Self::Lozha,
        Self::Kalka,
        Self::Semaphore,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Verstka => "Вёрстка",
            Self::Lumen => "Люмен",
            Self::Contact => "Контакт",
            Self::Azimuth => "Азимут",
            Self::Herbarium => "Гербарий",
            Self::Pult => "Пульт",
            Self::Orbit => "Орбита",
            Self::Lozha => "Ложа",
            Self::Kalka => "Калька",
            Self::Semaphore => "Семафор",
        }
    }

    pub const fn palette(self) -> Palette {
        let (dark, canvas, surface, ink, subdued, accent, line, selected, radius) = match self {
            Self::Verstka => (
                false, 0xf6f2e8, 0xede8db, 0x222624, 0x5e5d51, 0x973723, 0xcac3b2, 0xe8dfce, 0,
            ),
            Self::Lumen => (
                true, 0x0e1118, 0x171c26, 0xedf1f8, 0x9caabf, 0x97e7ce, 0x48566b, 0x22333c, 8,
            ),
            Self::Contact => (
                true, 0x221e1a, 0x2e2822, 0xf7e9d1, 0xbba88e, 0xf2b76c, 0x927956, 0x57432d, 14,
            ),
            Self::Azimuth => (
                true, 0x0e1c56, 0x162756, 0xe7ecff, 0xa0b9ff, 0xe2fa80, 0x586da6, 0x30467d, 0,
            ),
            Self::Herbarium => (
                false, 0xf3efda, 0xdfe2c8, 0x283425, 0x566642, 0x405936, 0x9fab80, 0xcfdbb5, 12,
            ),
            Self::Pult => (
                false, 0xdde2d5, 0xc8d1c0, 0x20322a, 0x425949, 0x24563a, 0x889d8d, 0xaad6ad, 5,
            ),
            Self::Orbit => (
                true, 0x12101c, 0x20182e, 0xede6ff, 0xaa9fc0, 0xcdb4ff, 0x705a90, 0x413467, 16,
            ),
            Self::Lozha => (
                true, 0x251920, 0x2c1d25, 0xfae9cc, 0xbea08f, 0xedbc70, 0x835d4c, 0x593735, 2,
            ),
            Self::Kalka => (
                false, 0xe9ece3, 0xf8f7ee, 0x1c3545, 0x49636c, 0x204f91, 0x9ab6b7, 0xf8de8e, 1,
            ),
            Self::Semaphore => (
                false, 0xf5edd9, 0xeae0c6, 0x1e2726, 0x5d6355, 0xb4341d, 0xc1b697, 0xf3cebc, 0,
            ),
        };
        Palette {
            dark,
            canvas: rgb(canvas),
            surface: rgb(surface),
            ink: rgb(ink),
            subdued: rgb(subdued),
            accent: rgb(accent),
            line: rgb(line),
            selected: rgb(selected),
            radius,
        }
    }

    /// Apply this theme to one UI subtree. Host font definitions and other UI scopes stay independent.
    pub fn apply(self, ui: &mut Ui) {
        let palette = self.palette();
        let mut visuals = if palette.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        visuals.window_fill = palette.canvas;
        visuals.panel_fill = palette.canvas;
        visuals.extreme_bg_color = palette.canvas;
        visuals.faint_bg_color = palette.surface;
        visuals.weak_text_color = Some(palette.subdued);
        visuals.text_edit_bg_color = Some(palette.surface);
        visuals.selection.bg_fill = palette.selected;
        visuals.selection.stroke = Stroke::new(1.0, palette.accent);
        visuals.override_text_color = Some(palette.ink);
        for state in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            state.bg_fill = palette.surface;
            state.weak_bg_fill = palette.surface;
            state.fg_stroke.color = palette.ink;
            state.bg_stroke = Stroke::new(1.0, palette.line);
            state.corner_radius = palette.radius.into();
            state.expansion = 0.0;
        }
        for state in [
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            state.bg_fill = palette.selected;
            state.weak_bg_fill = palette.selected;
            state.bg_stroke = Stroke::new(1.0, palette.accent);
        }
        let style = ui.style_mut();
        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.interact_size.y = 32.0;
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub dark: bool,
    pub canvas: Color32,
    pub surface: Color32,
    pub ink: Color32,
    pub subdued: Color32,
    pub accent: Color32,
    pub line: Color32,
    pub selected: Color32,
    pub radius: u8,
}

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// Work surface with a header and independent navigation and body. The host supplies data,
/// navigation state, callbacks, and scrolling policy for the body.
pub fn workspace<T>(
    ui: &mut Ui,
    theme: Theme,
    title: &str,
    heading_family: FontFamily,
    state: &mut T,
    navigation: impl FnOnce(&mut Ui, &mut T),
    body: impl FnOnce(&mut Ui, &mut T),
) {
    let palette = theme.palette();
    let size = ui.available_size();
    Frame::new()
        .fill(palette.canvas)
        .inner_margin(24)
        .show(ui, |ui| {
            theme.apply(ui);
            ui.set_min_size((size - egui::vec2(48.0, 48.0)).max(egui::Vec2::ZERO));
            ui.label(RichText::new(title).font(FontId::new(39.0, heading_family)));
            ui.separator();
            ui.add_space(9.0);
            if ui.available_width() < 650.0 {
                ui.horizontal_wrapped(|ui| navigation(ui, state));
                ui.separator();
                body(ui, state);
            } else {
                let height = ui.available_height();
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(174.0, height),
                        Layout::top_down(Align::Min),
                        |ui| navigation(ui, state),
                    );
                    ui.separator();
                    ui.allocate_ui_with_layout(
                        ui.available_size_before_wrap(),
                        Layout::top_down(Align::Min),
                        |ui| body(ui, state),
                    );
                });
            }
        });
}

/// Header and controls for a compact, keyboard-driven command or lookup window.
pub fn launcher(ui: &mut Ui, theme: Theme, title: &str, content: impl FnOnce(&mut Ui)) {
    let size = ui.available_size();
    Frame::new()
        .fill(theme.palette().canvas)
        .inner_margin(16)
        .show(ui, |ui| {
            theme.apply(ui);
            ui.set_min_size((size - egui::vec2(32.0, 32.0)).max(egui::Vec2::ZERO));
            ui.heading(title);
            content(ui);
        });
}
