use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use openrazer_adapter::OpenrazerAdapter;
use opensynapse_core::DeviceDiscovery;
use tracing::info;

pub async fn bootstrap_if_empty(root: &Path, openrazer: &OpenrazerAdapter) -> Result<bool> {
    let aliases_path = root.join("devices/_aliases.toml");
    if aliases_path.exists() {
        return Ok(false);
    }

    fs::create_dir_all(root.join("devices"))?;
    fs::create_dir_all(root.join("scenes"))?;

    let discovered = openrazer
        .discover()
        .await
        .context("discovering devices for first-run bootstrap")?;

    // _aliases.toml
    let mut aliases = String::from("# Generado por opensynapsed en el primer arranque.\n");
    aliases.push_str("# Editá libremente; las scenes referencian estos alias.\n\n");
    for d in &discovered {
        let alias = alias_from_name(&d.name);
        aliases.push_str(&format!(
            "[{alias}]\nvid = 0x{:04x}\npid = 0x{:04x}\n",
            d.id.vid, d.id.pid
        ));
        if let Some(s) = &d.id.serial {
            aliases.push_str(&format!("serial = \"{s}\"\n"));
        }
        aliases.push('\n');
    }
    fs::write(&aliases_path, aliases)?;

    // Un device profile + un scene "default" static cyan, para cada device descubierto.
    // Static es el modo más universalmente soportado entre HW Razer (algunos
    // headsets solo tienen iluminación de logo y rechazan spectrum/wave).
    for d in &discovered {
        let alias = alias_from_name(&d.name);
        let profile_name = format!("{alias}-default");
        let dp = format!(
            "schema_version = 1\n\n\
             [profile]\n\
             name = \"{profile_name}\"\n\
             description = \"Bootstrap: static cyan\"\n\n\
             [device_match]\n\
             vid = 0x{:04x}\n\
             pid = 0x{:04x}\n\n\
             [lighting]\n\
             mode = \"static\"\n\
             color = \"#00ffff\"\n\
             brightness = 80\n",
            d.id.vid, d.id.pid
        );
        fs::write(root.join(format!("devices/{profile_name}.toml")), dp)?;
    }

    // Scene default — fallback (priority 0, sin match).
    let mut default_scene = String::from(
        "schema_version = 1\n\n\
         [scene]\n\
         name = \"default\"\n\
         description = \"Bootstrap: static cyan en todos los devices\"\n\
         priority = 0\n\n\
         [devices]\n",
    );
    for d in &discovered {
        let alias = alias_from_name(&d.name);
        default_scene.push_str(&format!(
            "{alias} = {{ profile = \"{alias}-default\", required = false }}\n"
        ));
    }
    fs::write(root.join("scenes/default.toml"), default_scene)?;

    info!(devices = discovered.len(), root = %root.display(), "bootstrapped initial config");
    Ok(true)
}

/// "Razer Tartarus Pro" → "tartarus", "Razer Nari Ultimate" → "nari".
/// Heurística: si empieza con "Razer", tomar la segunda palabra; sino, primera.
fn alias_from_name(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let pick = if words.first().map(|s| s.eq_ignore_ascii_case("razer")) == Some(true) {
        words.get(1).copied().unwrap_or(name)
    } else {
        words.first().copied().unwrap_or(name)
    };
    pick.to_lowercase()
}
