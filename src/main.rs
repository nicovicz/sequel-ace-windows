// Windows-subsystem menyembunyikan console di build release. Debug build tetap
// console-supporting sehingga panic/error saat startup (mis. OpenGL gagal di
// VM tanpa GPU) terlihat ketika dijalankan dari PowerShell.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod database;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([882.0, 759.0])
            .with_min_inner_size([760.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sequel Ace",
        options,
        Box::new(|_creation_context| Ok(Box::new(app::SequelAceApp::new()))),
    )
}
