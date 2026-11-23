use avilon_egui::facets::{Form, Light, View};
use eframe::egui;

struct Preview(View);

impl eframe::App for Preview {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        ui.painter()
            .rect_filled(ui.max_rect(), 0, egui::Color32::from_rgb(16, 24, 43));
        self.0.show(ui, ui.available_size());
    }
}

fn main() -> eframe::Result {
    let form = Form::from_mesh(
        &[
            [0.42, 1.4, 0.18],
            [-1.0, -0.42, -0.68],
            [0.84, -0.34, -0.35],
            [-0.2, -0.73, 1.1],
        ],
        &[&[0, 1, 2], &[0, 2, 3], &[0, 3, 1], &[1, 2, 3]],
    )
    .expect("the asymmetric talisman is a convex tetrahedron");
    let view = View::new(form)
        .with_light(Light::Pearl)
        .with_spin_speed(0.6);
    eframe::run_native(
        "Асимметричный талисман",
        eframe::NativeOptions::default(),
        Box::new(move |_| Ok(Box::new(Preview(view)))),
    )
}
