mod instruction;
mod memory;
mod organism;
mod cpu;
mod scheduler;
mod stats;
mod simulator;
mod ui;

use ui::TierraApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Tierra - Artificial Life Simulation",
        options,
        Box::new(|_cc| Ok(Box::<TierraApp>::default())),
    )
}
