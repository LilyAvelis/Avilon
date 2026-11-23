use std::io;
use std::path::Path;
use std::process::Command;

use eframe::egui::{self, FontId, RichText, Stroke, TextureHandle, Vec2, vec2};

use crate::graphics::{INK, MUTED, PAPER, RULE};

use super::{complexity_badge, demo_executable};

struct Project {
    title: &'static str,
    description: &'static str,
    complexity: usize,
    executable: &'static str,
    preview: &'static [u8],
}

const PROJECTS: [Project; 3] = [
    Project {
        title: "Наклонный сад",
        description: "Наклони поднос и докати шарик до светящейся лунки.",
        complexity: 4,
        executable: "target/unity-kinetka-v2/NaklonnySad.exe",
        preview: include_bytes!("../../assets/animation/garden.jpg"),
    },
    Project {
        title: "Площадь от рассвета до ночи",
        description: "Поверни диск. Тени вытянутся по мостовой, а ночью зажгутся окна.",
        complexity: 3,
        executable: "target/unity-kinetka/ClockSquare/ClockSquare.exe",
        preview: include_bytes!("../../assets/animation/square.jpg"),
    },
    Project {
        title: "Дорожка домино",
        description: "Толкни первую плашку. Последняя ударит по колокольчику.",
        complexity: 4,
        executable: "target/unity-kinetka/DominoRun/DominoRun.exe",
        preview: include_bytes!("../../assets/animation/domino.jpg"),
    },
];

#[derive(Default)]
pub(super) struct Gallery {
    previews: Option<[TextureHandle; PROJECTS.len()]>,
    errors: [Option<String>; PROJECTS.len()],
}

impl Gallery {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, heading: FontId, jump: bool) {
        ui.separator();
        ui.add_space(10.0);
        let title = ui.label(
            RichText::new("Проекты Unity")
                .font(heading.clone())
                .color(INK),
        );
        if jump {
            title.scroll_to_me(Some(egui::Align::Min));
        }
        ui.add_space(12.0);

        let previews = self.previews.get_or_insert_with(|| {
            std::array::from_fn(|index| {
                let project = &PROJECTS[index];
                let image =
                    image::load_from_memory_with_format(project.preview, image::ImageFormat::Jpeg)
                        .expect("embedded project preview must be a valid JPEG")
                        .into_rgba8();
                ui.ctx().load_texture(
                    project.title,
                    egui::ColorImage::from_rgba_unmultiplied(
                        [image.width() as usize, image.height() as usize],
                        image.as_raw(),
                    ),
                    egui::TextureOptions::LINEAR,
                )
            })
        });
        let project_heading = FontId::new(20.0, heading.family);
        if ui.available_width() >= 800.0 {
            ui.columns(PROJECTS.len(), |columns| {
                for (index, ui) in columns.iter_mut().enumerate() {
                    show_card(
                        ui,
                        &PROJECTS[index],
                        &previews[index],
                        &project_heading,
                        &mut self.errors[index],
                        true,
                    );
                }
            });
        } else {
            for (index, project) in PROJECTS.iter().enumerate() {
                show_card(
                    ui,
                    project,
                    &previews[index],
                    &project_heading,
                    &mut self.errors[index],
                    false,
                );
                ui.add_space(9.0);
            }
        }
    }
}

fn show_card(
    ui: &mut egui::Ui,
    project: &Project,
    preview: &TextureHandle,
    heading: &FontId,
    error: &mut Option<String>,
    column: bool,
) {
    ui.push_id(project.title, |ui| {
        egui::Frame::new()
            .fill(PAPER.gamma_multiply(0.98))
            .stroke(Stroke::new(1.0, RULE))
            .inner_margin(12)
            .show(ui, |ui| {
                let width = ui.available_width();
                if column {
                    ui.set_min_width(width);
                    draw_preview(ui, preview, width);
                    ui.add_space(8.0);
                    show_details(ui, project, heading, error, true);
                } else {
                    ui.horizontal_top(|ui| {
                        draw_preview(ui, preview, (width * 0.34).min(240.0));
                        ui.add_space(9.0);
                        ui.vertical(|ui| show_details(ui, project, heading, error, false));
                    });
                }
            });
    });
}

fn draw_preview(ui: &mut egui::Ui, preview: &TextureHandle, width: f32) {
    let size = vec2(width, width * preview.aspect_ratio().recip());
    ui.add(egui::Image::new((preview.id(), size)).fit_to_exact_size(size));
}

fn show_details(
    ui: &mut egui::Ui,
    project: &Project,
    heading: &FontId,
    error: &mut Option<String>,
    column: bool,
) {
    let width = ui.available_width();
    ui.add_sized(
        vec2(width, if column { 47.0 } else { 0.0 }),
        egui::Label::new(
            RichText::new(project.title)
                .font(heading.clone())
                .color(INK),
        )
        .wrap(),
    );
    ui.add_sized(
        vec2(width, if column { 40.0 } else { 0.0 }),
        egui::Label::new(RichText::new(project.description).size(14.0).color(MUTED)).wrap(),
    );
    ui.add_space(4.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(165.0, 23.0), egui::Sense::hover());
    complexity_badge(ui, rect, project.complexity);
    ui.add_space(5.0);
    let response = ui.add_enabled(
        cfg!(windows),
        egui::Button::new(RichText::new("Запустить ↗").color(INK)).min_size(vec2(116.0, 30.0)),
    );
    let response = if cfg!(windows) {
        response.on_hover_text(format!("Открыть «{}» в отдельном окне", project.title))
    } else {
        response.on_disabled_hover_text("Эта сборка предназначена для Windows")
    };
    if response.clicked() {
        *error = launch(project).err().map(|err| err.to_string());
    }
    if let Some(error) = error {
        ui.label(RichText::new(error.as_str()).color(egui::Color32::from_rgb(141, 50, 34)));
    }
}

fn launch(project: &Project) -> io::Result<()> {
    let path = demo_executable(Path::new(project.executable))?;
    Command::new(&path)
        .current_dir(path.parent().expect("executable has a directory"))
        .spawn()
        .map(drop)
}
