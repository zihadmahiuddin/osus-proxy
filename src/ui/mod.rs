use std::sync::Arc;
use tokio::sync::Mutex;
use crate::preferences::{BeatmapMirror, Preferences};

pub fn run(
    preferences: Arc<Mutex<Preferences>>,
) -> eframe::Result<()> {
    let tokio_rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        ..Default::default()
    };

    eframe::run_simple_native(
        "osus Proxy",
        options,
        move |ctx, _frame| {
            let mut preferences = tokio_rt.block_on(preferences.lock());
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("General purpose proxy for osu!bancho server");
                ui.checkbox(&mut preferences.fake_supporter, "Fake osu!supporter");
                ui.vertical(|ui| {
                    let label = ui.label("Server Address");
                    ui.text_edit_singleline(&mut preferences.server_address)
                        .labelled_by(label.id);
                });

                egui::ComboBox::from_label("Beatmap Download Mirror")
                    .selected_text(format!("{:?}", &preferences.beatmap_mirror))
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut preferences.beatmap_mirror,
                            BeatmapMirror::Chimu,
                            format!("{} (recommended, probably fastest for most people)", &BeatmapMirror::Chimu),
                        );
                        ui.selectable_value(
                            &mut preferences.beatmap_mirror,
                            BeatmapMirror::BeatConnect,
                            "BeatConnect",
                        );
                        ui.selectable_value(
                            &mut preferences.beatmap_mirror,
                            BeatmapMirror::Nerinyan,
                            "nerinyan.moe",
                        );
                        ui.selectable_value(
                            &mut preferences.beatmap_mirror,
                            BeatmapMirror::ServerDefault,
                            format!("{} (not recommended with 'Fake osu!supporter', they might be able to detect it)", &BeatmapMirror::ServerDefault),
                        );
                    })
            });
        })
}