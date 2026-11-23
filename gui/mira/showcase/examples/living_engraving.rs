use avilon_egui::engraving::{Settings, View};
use eframe::egui;

struct Preview(View);

impl eframe::App for Preview {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.0.show(ui, ui.available_size());
    }
}

fn main() -> eframe::Result {
    let view = View::new(Settings {
        seed: 42,
        radius: 0.12,
        zoom: 4.5,
        live: true,
        ..Settings::default()
    });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([760.0, 460.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Живая гравюра",
        options,
        Box::new(move |_| Ok(Box::new(Preview(view)))),
    )
}
