use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingMode {
    None,
    Static,
    Breathing,
    Spectrum,
    Wave,
    Reactive,
}

#[derive(Debug, Clone)]
pub struct LightingSpec {
    pub mode: LightingMode,
    pub color: Option<Color>,
    pub brightness: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct MacroSpec {
    pub key: String,
    pub sequence: Vec<MacroAction>,
}

#[derive(Debug, Clone)]
pub enum MacroAction {
    Key { value: String },
    Text { value: String },
    Delay { ms: u32 },
}

#[derive(Debug, Clone)]
pub struct DpiSpec {
    pub stages: Vec<u32>,
    pub active_stage: u8,
}

#[derive(Debug, Clone, Default)]
pub struct AudioSpec {
    pub default_sink: Option<String>,
    pub sidetone_db: Option<f32>,
    pub eq_preset: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ForegroundEvent {
    pub wm_class: String,
    pub title: String,
    pub timestamp: Instant,
}
