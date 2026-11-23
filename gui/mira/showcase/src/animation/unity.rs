use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use eframe::egui::{self, FontId, RichText, Stroke, TextureHandle, Vec2, vec2};

use crate::graphics::{INK, MUTED, PAPER, RULE};

use super::{complexity_badge, demo_executable};

const LEGACY_WATER_LAB: &str = "target/unity-water-lab-sky/WaterLab.exe";
const WATER_LAB_RELEASE: &str = "target/water-lab-current.txt";

struct Project {
    title: &'static str,
    description: &'static str,
    complexity: usize,
    executable: &'static str,
    preview: &'static [u8],
}

const PROJECTS: [Project; 4] = [
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
    Project {
        title: "Лаборатория воды",
        description: "Брось шарик и сравни волны, преломление и три жидкости.",
        complexity: 4,
        executable: LEGACY_WATER_LAB,
        preview: include_bytes!("../../assets/animation/water-lab.jpg"),
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
        let columns = match ui.available_width() {
            width if width >= 1050.0 => 4,
            width if width >= 800.0 => 2,
            _ => 1,
        };
        for first in (0..PROJECTS.len()).step_by(columns) {
            ui.columns(columns, |row| {
                for (offset, ui) in row.iter_mut().enumerate() {
                    let index = first + offset;
                    if let Some(project) = PROJECTS.get(index) {
                        show_card(
                            ui,
                            project,
                            &previews[index],
                            &project_heading,
                            &mut self.errors[index],
                            columns > 1,
                        );
                    }
                }
            });
            ui.add_space(9.0);
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
    let path = if project.executable == LEGACY_WATER_LAB {
        water_lab_executable()?
    } else {
        demo_executable(Path::new(project.executable))?
    };
    Command::new(&path)
        .current_dir(path.parent().expect("executable has a directory"))
        .spawn()
        .map(drop)
}

fn water_lab_executable() -> io::Result<PathBuf> {
    let manifest = match demo_executable(Path::new(WATER_LAB_RELEASE)) {
        Ok(path) => path,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return demo_executable(Path::new(LEGACY_WATER_LAB));
        }
        Err(err) => return Err(err),
    };
    let release = std::fs::read_to_string(&manifest)?;
    let release = release.trim();
    if !release.starts_with("water-lab-release-")
        || !release
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Неверный путь выпуска лаборатории",
        ));
    }
    let folder = manifest.with_file_name(release);
    let executable = folder.join("WaterLab.exe");
    if !executable.is_file()
        || !folder.join("UnityPlayer.dll").is_file()
        || !folder.join("WaterLab_Data/globalgamemanagers").is_file()
        || !folder.join("water-lab-inputs.sha256").is_file()
    {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Неполный выпуск лаборатории",
        ));
    }
    Ok(executable)
}
