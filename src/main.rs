mod config;
mod mqtt_connector;

use tokio::signal;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("timepipe is starting...");
    let cfg_path =
        std::env::var("TIMEPIPE_CONFIG").unwrap_or_else(|_| "config/config.toml".to_string());

    let cfg: config::Config = config::Config::load_from_file(&cfg_path)?;
    tracing::info!("Loading Config from '{}'", cfg_path);

    tracing::info!("Will now start MQTT Connector");
    let (mut schema_rx, mut data_rx) = mqtt_connector::run_mqtt(&cfg.mqtt).await?;

    // schema worker
    let schema_task = tokio::spawn(async move {
        while let Some(schema) = schema_rx.recv().await {
            tracing::info!(?schema, "recieved a schema");
        }
        tracing::info!("schema task ended");
    });
    // data worker
    let data_task = tokio::spawn(async move {
        while let Some(data) = data_rx.recv().await {
            tracing::info!(?data, "recieved data");
        }
        tracing::info!("data task ended");
    });

    tracing::info!("Press Ctrl-C to stop application");
    // --- graceful shutdown ---
    // On Unix (Docker/K8s): listen for SIGTERM/SIGINT
    // Elsewhere: just Ctrl-C
    #[cfg(unix)]
    {
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sigint = signal(SignalKind::interrupt())?;
        tokio::select! {
            _ = signal::ctrl_c() => {},
            _ = sigterm.recv() => {},
            _ = sigint.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        tokio::select! {
            _ = signal::ctrl_c() => {},
        }
    }

    tracing::info!("timepipe will exit now gracefully");
    Ok(())
}
