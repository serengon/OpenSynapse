use anyhow::Result;
use openrazer_adapter::Adapter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let adapter = Adapter::connect().await?;
    let devices = adapter.list_devices().await?;

    if devices.is_empty() {
        println!("no devices found");
        return Ok(());
    }

    for device in &devices {
        let info = device.info().await?;
        println!(
            "{} ({:04x}:{:04x}) — serial {}",
            info.name, info.vid, info.pid, info.serial
        );
        println!("  type: {}", info.kind);
        match device.battery().await? {
            Some(b) => println!(
                "  battery: {:.0}%{}",
                b.level,
                if b.charging { " (charging)" } else { "" }
            ),
            None => println!("  battery: n/a"),
        }
    }

    Ok(())
}
