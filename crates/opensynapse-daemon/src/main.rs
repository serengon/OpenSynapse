use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use foreground_watcher_x11::X11ForegroundWatcher;
use openrazer_adapter::OpenrazerAdapter;
use opensynapse_core::ForegroundWatcher;
use tokio::signal;
use tokio_stream::StreamExt;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod bootstrap;
mod config;
mod engine;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config_dir = config_dir()?;
    info!(config = %config_dir.display(), "starting opensynapsed");

    let openrazer = OpenrazerAdapter::connect()
        .await
        .context("connecting to openrazer DBus")?;

    let bootstrapped = bootstrap::bootstrap_if_empty(&config_dir, &openrazer).await?;
    if bootstrapped {
        info!(
            "first-run bootstrap done; edit configs in {} to customize",
            config_dir.display()
        );
    }

    let loaded = config::LoadedConfig::load(&config_dir).context("loading configs")?;
    info!(
        scenes = loaded.scenes.len(),
        device_profiles = loaded.device_profiles.len(),
        aliases = loaded.aliases.len(),
        "config ready"
    );

    let lighting: Arc<dyn opensynapse_core::LightingAdapter> = Arc::new(openrazer);
    let mut engine = engine::Engine::new(loaded, lighting);

    let watcher = X11ForegroundWatcher::start().context("starting X11 foreground watcher")?;
    let mut events = watcher
        .watch()
        .await
        .context("subscribing to foreground events")?;

    info!("watching foreground; switch windows to trigger scene changes. Ctrl+C to exit.");

    loop {
        tokio::select! {
            ev = events.next() => match ev {
                Some(ev) => {
                    if let Err(e) = engine.handle_event(&ev).await {
                        error!(error = %e, "engine handle_event failed");
                    }
                }
                None => {
                    error!("foreground stream ended; exiting");
                    return Ok(());
                }
            },
            _ = signal::ctrl_c() => {
                info!("ctrl+c received; exiting");
                return Ok(());
            }
        }
    }
}

fn config_dir() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("OPENSYNAPSE_CONFIG_DIR") {
        return Ok(PathBuf::from(custom));
    }
    let pd = ProjectDirs::from("", "", "opensynapse").context("could not determine config dir")?;
    Ok(pd.config_dir().to_path_buf())
}
