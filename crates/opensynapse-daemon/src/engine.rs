use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use opensynapse_core::{DeviceId, ForegroundEvent, LightingAdapter};
use tracing::{debug, info, warn};

use crate::config::{DeviceProfileFile, LoadedConfig, SceneFile};

pub struct Engine {
    pub lighting: Arc<dyn LightingAdapter>,
    pub aliases: HashMap<String, DeviceId>,
    pub device_profiles: HashMap<String, DeviceProfileFile>,
    pub scenes: Vec<SceneFile>,
    last_scene: Option<String>,
    last_class: Option<String>,
}

impl Engine {
    pub fn new(config: LoadedConfig, lighting: Arc<dyn LightingAdapter>) -> Self {
        Self {
            lighting,
            aliases: config.aliases,
            device_profiles: config.device_profiles,
            scenes: config.scenes,
            last_scene: None,
            last_class: None,
        }
    }

    pub async fn handle_event(&mut self, event: &ForegroundEvent) -> Result<()> {
        debug!(wm_class = %event.wm_class, title = %event.title, "received event");
        // Dedup a nivel de evento: misma clase consecutiva = no repetimos match.
        // (KDE emite cada PropertyNotify duplicado).
        if self.last_class.as_deref() == Some(event.wm_class.as_str()) {
            return Ok(());
        }
        self.last_class = Some(event.wm_class.clone());

        let chosen = self.pick_scene(event);
        match chosen {
            Some(scene) => {
                let name = scene.scene.name.clone();
                if self.last_scene.as_deref() == Some(name.as_str()) {
                    return Ok(());
                }
                info!(scene = %name, wm_class = %event.wm_class, "switching scene");
                self.apply_scene_by_name(&name).await;
                self.last_scene = Some(name);
            }
            None => {
                warn!(wm_class = %event.wm_class, "no matching scene");
            }
        }
        Ok(())
    }

    fn pick_scene(&self, event: &ForegroundEvent) -> Option<&SceneFile> {
        let mut candidates: Vec<&SceneFile> = self
            .scenes
            .iter()
            .filter(|s| {
                s.match_rules.is_empty() || s.match_rules.iter().any(|r| matches_rule(r, event))
            })
            .collect();
        // mayor prioridad gana; empate por nombre lex (determinístico)
        candidates.sort_by(|a, b| {
            b.scene
                .priority
                .cmp(&a.scene.priority)
                .then_with(|| a.scene.name.cmp(&b.scene.name))
        });
        candidates
            .into_iter()
            .find(|s| self.required_devices_present(s))
    }

    fn required_devices_present(&self, scene: &SceneFile) -> bool {
        scene
            .devices
            .iter()
            .filter(|(_, b)| b.required)
            .all(|(alias, _)| self.aliases.contains_key(alias))
    }

    async fn apply_scene_by_name(&self, name: &str) {
        // Buscar la scene por nombre (no podemos guardar &SceneFile mientras llamamos
        // métodos que toman &mut self; clonamos el Arc/refs necesarios).
        let scene = match self.scenes.iter().find(|s| s.scene.name == name) {
            Some(s) => s,
            None => return,
        };
        for (alias, binding) in &scene.devices {
            let Some(id) = self.aliases.get(alias) else {
                warn!(alias, "alias missing at apply time");
                continue;
            };
            let Some(profile) = self.device_profiles.get(&binding.profile) else {
                warn!(profile = %binding.profile, "device profile missing at apply time");
                continue;
            };
            if let Some(spec) = &profile.lighting {
                if let Err(e) = self.lighting.apply_lighting(id, spec).await {
                    warn!(error = %e, alias, "lighting apply failed");
                }
            }
            // macros / dpi: out of scope v0.1-MVP
        }
    }
}

fn matches_rule(rule: &crate::config::MatchRule, event: &ForegroundEvent) -> bool {
    if let Some(class) = &rule.wm_class {
        if class != &event.wm_class {
            return false;
        }
    }
    true
}
