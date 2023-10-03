#![windows_subsystem = "windows"]

use crate::preferences::Preferences;
use color_eyre::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod osus_proxy;
mod preferences;
mod ui;

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // TODO: implement preferences saving and loading?
    let preferences = Arc::new(Mutex::new(Preferences::default()));

    let preferences_clone = preferences.clone();
    let proxy_thread = std::thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                osus_proxy::start(preferences_clone)
                    .await
                    .expect("Failed to run proxy")
            })
    });

    ui::run(preferences).unwrap();
    proxy_thread
        .join()
        .expect("Could not join on proxy thread.");

    Ok(())

    // let packet_file = include_bytes!("../packets/1.bin");
    // process_bancho_packet(packet_file).await;
    // Ok();
}
