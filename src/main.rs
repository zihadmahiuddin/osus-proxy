#![windows_subsystem = "windows"]

use crate::preferences::Preferences;
use color_eyre::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::metadata::LevelFilter;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

mod osus_proxy;
mod preferences;
mod ui;

fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::never("./", "osus-proxy.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(non_blocking)
                .with_filter(LevelFilter::from(Level::DEBUG)),
        )
        .with(tracing_subscriber::fmt::layer().with_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        ))
        .init();

    // TODO: implement preferences saving and loading?
    let preferences = Arc::new(Mutex::new(Preferences::default()));

    let preferences_clone = preferences.clone();
    let _proxy_thread = std::thread::spawn(|| {
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

    Ok(())

    // let packet_file = include_bytes!("../packets/1.bin");
    // process_bancho_packet(packet_file).await;
    // Ok();
}
