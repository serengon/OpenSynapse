use anyhow::Result;
use foreground_watcher_x11::X11ForegroundWatcher;
use opensynapse_core::ForegroundWatcher;
use tokio_stream::StreamExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let watcher = X11ForegroundWatcher::start()?;
    let mut stream = watcher.watch().await?;

    println!("watching foreground; switch windows to see events. Ctrl+C to exit.");
    let start = std::time::Instant::now();
    while let Some(ev) = stream.next().await {
        let dt = ev.timestamp.duration_since(start);
        println!(
            "[+{:>6.2}s] class={:?} title={:?}",
            dt.as_secs_f64(),
            ev.wm_class,
            ev.title
        );
    }
    Ok(())
}
