use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use opensynapse_core::{DeviceId, DpiSpec, LightingSpec, MacroSpec};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceProfileFile {
    pub schema_version: u32,
    pub profile: ProfileMeta,
    pub device_match: DeviceMatch,
    #[serde(default)]
    pub lighting: Option<LightingSpec>,
    #[serde(default)]
    pub macros: Option<Vec<MacroSpec>>,
    #[serde(default)]
    pub dpi: Option<DpiSpec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceMatch {
    pub vid: u16,
    pub pid: u16,
    #[serde(default)]
    pub serial: Option<String>,
}

impl From<&DeviceMatch> for DeviceId {
    fn from(m: &DeviceMatch) -> Self {
        DeviceId {
            vid: m.vid,
            pid: m.pid,
            serial: m.serial.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SceneFile {
    pub schema_version: u32,
    pub scene: SceneMeta,
    #[serde(default, rename = "match")]
    pub match_rules: Vec<MatchRule>,
    #[serde(default)]
    pub devices: HashMap<String, DeviceBinding>,
    #[serde(default)]
    pub audio: Option<opensynapse_core::AudioSpec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SceneMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatchRule {
    #[serde(default)]
    pub wm_class: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceBinding {
    pub profile: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct AliasesFile {
    pub aliases: HashMap<String, AliasEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AliasEntry {
    pub vid: u16,
    pub pid: u16,
    #[serde(default)]
    pub serial: Option<String>,
}

impl From<&AliasEntry> for DeviceId {
    fn from(a: &AliasEntry) -> Self {
        DeviceId {
            vid: a.vid,
            pid: a.pid,
            serial: a.serial.clone(),
        }
    }
}

pub struct LoadedConfig {
    pub aliases: HashMap<String, DeviceId>,
    pub device_profiles: HashMap<String, DeviceProfileFile>,
    pub scenes: Vec<SceneFile>,
}

impl LoadedConfig {
    pub fn load(root: &Path) -> Result<Self> {
        let aliases_path = root.join("devices/_aliases.toml");
        let aliases_raw: AliasesFile = read_toml(&aliases_path)?;
        let aliases: HashMap<String, DeviceId> = aliases_raw
            .aliases
            .iter()
            .map(|(k, v)| (k.clone(), v.into()))
            .collect();

        let mut device_profiles = HashMap::new();
        let devices_dir = root.join("devices");
        for entry in read_dir_toml(&devices_dir)? {
            let filename = entry
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("bad filename: {entry:?}"))?
                .to_string();
            if filename.starts_with('_') {
                continue;
            }
            let parsed: DeviceProfileFile =
                read_toml(&entry).with_context(|| format!("loading device profile {entry:?}"))?;
            check_version(parsed.schema_version, &entry)?;
            if parsed.profile.name != filename {
                bail!(
                    "device profile name {:?} does not match filename {:?}",
                    parsed.profile.name,
                    filename
                );
            }
            device_profiles.insert(filename, parsed);
        }

        let mut scenes = Vec::new();
        let scenes_dir = root.join("scenes");
        for entry in read_dir_toml(&scenes_dir)? {
            let filename = entry
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("bad filename: {entry:?}"))?
                .to_string();
            let parsed: SceneFile =
                read_toml(&entry).with_context(|| format!("loading scene {entry:?}"))?;
            check_version(parsed.schema_version, &entry)?;
            if parsed.scene.name != filename {
                bail!(
                    "scene name {:?} does not match filename {:?}",
                    parsed.scene.name,
                    filename
                );
            }
            scenes.push(parsed);
        }

        // Validar referencias cruzadas; warnings, no abort.
        for s in &scenes {
            for (alias, binding) in &s.devices {
                if !aliases.contains_key(alias) {
                    warn!(scene = %s.scene.name, alias, "scene references unknown alias");
                }
                if !device_profiles.contains_key(&binding.profile) {
                    warn!(scene = %s.scene.name, profile = %binding.profile,
                          "scene references unknown device profile");
                }
            }
        }

        debug!(
            aliases = aliases.len(),
            device_profiles = device_profiles.len(),
            scenes = scenes.len(),
            "config loaded"
        );

        Ok(Self {
            aliases,
            device_profiles,
            scenes,
        })
    }
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {path:?}"))?;
    toml::from_str::<T>(&raw).with_context(|| format!("parsing {path:?}"))
}

fn read_dir_toml(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("reading dir {dir:?}"))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            // skip files starting with '_' here — those son metadata (aliases)
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.starts_with('_') {
                    continue;
                }
            }
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

fn check_version(v: u32, path: &Path) -> Result<()> {
    if v == 0 || v > SCHEMA_VERSION {
        bail!("{path:?} schema_version {v} not supported (this build supports {SCHEMA_VERSION})");
    }
    Ok(())
}
