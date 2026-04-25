# Adapter traits v0

Contrato entre el orchestrator y cada backend (openrazer, input-remapper, PipeWire, libratbag, foreground-watcher). Define qué tiene que poder hacer un adapter, **no cómo lo hace**. Los spikes #2-#6 implementan estas traits contra HW real y validan si los backends realmente las soportan.

Estado: **draft**. Cualquier cambio incompatible bumpea `trait_version` (constante en `opensynapse-core`).

## Goals

- **Capability-oriented, no backend-oriented**: el orchestrator pide "aplicá este lighting", no "llamá a openrazer". Permite cambiar el backend de una capability sin tocar el orchestrator.
- **Per-capability, no monolítico**: un adapter implementa solo las capabilities que cubre (openrazer ≠ PipeWire). Composición vía traits, no jerarquía.
- **Async tokio-first**: alineado con la decisión del workspace. Métodos async, errores con `Result`.
- **Sin estado global**: cada adapter es `Send + Sync`, instanciable múltiples veces. El orchestrator decide la lifetime.

## Non-goals (v0)

- **Rollback transaccional**. v0 = best-effort: si la mitad de un scene apply falla, se loguea y se sigue. Spike #2 valida si openrazer permite rollback; si sí, se introduce un trait `Transactional` opcional.
- **Read-back de estado actual del HW**. El orchestrator mantiene en memoria qué scene aplicó; no pregunta al HW "¿qué color tenés ahora?".
- **Capability introspection rica** (¿este device soporta breathing?). v0: el adapter intenta y falla con warning. La GUI v0.1 muestra todas las opciones sin grey-out; v1 sumará introspection.
- **Hot-plug**. Discovery se llama on-demand y al startup; suscripción a eventos = v0.2.

## Identidad de devices

```rust
/// Identifica un device físico de forma estable entre runs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId {
    pub vid: u16,
    pub pid: u16,
    /// Some si hay >1 unidad del mismo modelo y necesitamos distinguir.
    /// None si la scene es ambivalente (cualquier unidad sirve).
    pub serial: Option<String>,
}
```

`DeviceId` se construye desde `device_match` del Profile schema, y desde `discover()` del adapter. Igualdad: `serial.is_some()` requiere match exacto; `None` matchea cualquier serial con mismo vid/pid.

## Trait base: Discovery

```rust
#[async_trait]
pub trait DeviceDiscovery: Send + Sync {
    /// Enumera devices que este adapter puede manejar AHORA.
    /// No cachea: el orchestrator decide cuándo llamar.
    async fn discover(&self) -> Result<Vec<DiscoveredDevice>>;
}

pub struct DiscoveredDevice {
    pub id: DeviceId,
    pub name: String,
    pub kind: DeviceKind,    // Keypad | Keyboard | Mouse | Headset | Other
}
```

Adapters globales (PipeWire, foreground-watcher) **no implementan** `DeviceDiscovery`.

## Capability traits

Cada capability es una trait separada. Un adapter implementa las que cubre.

### Lighting

```rust
#[async_trait]
pub trait LightingAdapter: Send + Sync {
    async fn apply(&self, device: &DeviceId, spec: &LightingSpec) -> Result<()>;
}

pub struct LightingSpec {
    pub mode: LightingMode,           // Static, Breathing, Spectrum, Wave, Reactive, None
    pub color: Option<Color>,         // requerido para modes que lo usan
    pub brightness: Option<u8>,       // 0..=100
}

pub struct Color { pub r: u8, pub g: u8, pub b: u8 }
```

Si el device no soporta el `mode` pedido, el adapter retorna `AdapterError::UnsupportedCapability { detail }`. El orchestrator loguea y sigue con el resto de capabilities.

### Macros

```rust
#[async_trait]
pub trait MacroAdapter: Send + Sync {
    async fn apply(&self, device: &DeviceId, macros: &[MacroSpec]) -> Result<()>;
}

pub struct MacroSpec {
    pub key: String,                  // identifier de tecla, ej "M1", "thumb_up"
    pub sequence: Vec<MacroAction>,
}

pub enum MacroAction {
    Key { value: String },            // "ctrl+w", "enter"
    Text { value: String },           // typed literal
    Delay { ms: u32 },
}
```

### Mouse / DPI

```rust
#[async_trait]
pub trait DpiAdapter: Send + Sync {
    async fn apply(&self, device: &DeviceId, spec: &DpiSpec) -> Result<()>;
}

pub struct DpiSpec {
    pub stages: Vec<u32>,
    pub active_stage: u8,
}
```

### Audio (global, no device)

```rust
#[async_trait]
pub trait AudioAdapter: Send + Sync {
    async fn apply(&self, spec: &AudioSpec) -> Result<()>;
}

pub struct AudioSpec {
    pub default_sink: Option<String>,    // PipeWire node name; None = no cambiar
    pub sidetone_db: Option<f32>,
    pub eq_preset: Option<String>,       // referencia opaca; resolución es del adapter
}
```

### Foreground (global, read-only)

Distinto en forma: no es "apply", es "subscribe".

```rust
#[async_trait]
pub trait ForegroundWatcher: Send + Sync {
    /// Stream de eventos. Vivo mientras el handle exista.
    async fn watch(&self) -> Result<BoxStream<'static, ForegroundEvent>>;
}

pub struct ForegroundEvent {
    pub wm_class: String,
    pub title: String,
    pub timestamp: Instant,
}
```

El orchestrator suscribe una vez al startup y reacciona a cada evento. Spike #6 valida latencia y estabilidad.

## Error model

```rust
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("device not found: {0:?}")]
    DeviceNotFound(DeviceId),

    #[error("capability not supported on this device: {detail}")]
    UnsupportedCapability { detail: String },

    #[error("backend transient failure: {source}")]
    Transient { #[source] source: BoxError },

    #[error("backend fatal failure: {source}")]
    Fatal { #[source] source: BoxError },
}
```

Distinción **transient vs fatal** importa al orchestrator:
- **Transient** (timeout, device temporariamente busy): el orchestrator puede reintentar 1 vez antes de logar como warning.
- **Fatal** (parse error, schema mismatch, backend muerto): no reintentar; loguear como error y skipear esta capability para esta scene.
- **DeviceNotFound**: no es error si el device del scene era `optional`; sí lo es si era `required` (lo decide el orchestrator, no el adapter).
- **UnsupportedCapability**: warning, skip esa capability, seguir con el resto.

## Composición: cómo un adapter implementa varias capabilities

```rust
pub struct OpenrazerAdapter { /* zbus connection, etc */ }

impl DeviceDiscovery for OpenrazerAdapter { /* ... */ }
impl LightingAdapter for OpenrazerAdapter { /* ... */ }
impl MacroAdapter for OpenrazerAdapter { /* ... */ }
impl DpiAdapter for OpenrazerAdapter { /* ... */ }
```

`PipewireAdapter` implementa solo `AudioAdapter`. `InputRemapperAdapter` implementa solo `MacroAdapter` (alternativa al MacroAdapter de openrazer si éste no alcanza). `LibratbagAdapter` implementa solo `DpiAdapter`.

## Registry / wiring en el orchestrator

```rust
pub struct AdapterRegistry {
    pub discovery: Vec<Arc<dyn DeviceDiscovery>>,
    pub lighting: Vec<Arc<dyn LightingAdapter>>,
    pub macros:   Vec<Arc<dyn MacroAdapter>>,
    pub dpi:      Vec<Arc<dyn DpiAdapter>>,
    pub audio:    Option<Arc<dyn AudioAdapter>>,         // single
    pub foreground: Option<Arc<dyn ForegroundWatcher>>,  // single
}
```

Resolución per-device cuando el orchestrator aplica una scene:
1. Resolver alias → `DeviceId` desde `_aliases.toml`.
2. Para cada capability del Device Profile, iterar `registry.<capability>` y llamar `apply(device_id, spec)` al primer adapter cuyo `discover()` haya reportado ese device.
3. Si ningún adapter lo reportó, tratar como `DeviceNotFound`.

Discovery se cachea en el orchestrator con TTL (ej: 5s) para no hammerear DBus en cada scene change. Refresh manual vía CLI.

**Orden de adapters en el `Vec`**: significativo. El primero que pueda gana. Esto permite que el usuario configure "para macros, preferí input-remapper sobre openrazer" sin código nuevo.

## Mapping crates → adapters (v0.1 target)

| Crate | Implementa | Spike |
|---|---|---|
| `openrazer-adapter` | `DeviceDiscovery`, `LightingAdapter`, `MacroAdapter`, `DpiAdapter` | #1 ✅, #2 |
| `pipewire-adapter` | `AudioAdapter` | #4 (v0.2) |
| `input-remapper-adapter` | `MacroAdapter` | #3 |
| `libratbag-adapter` | `DpiAdapter` | #5 (v0.3) |
| `foreground-watcher-x11` | `ForegroundWatcher` | #6 |
| `foreground-watcher-wayland` | `ForegroundWatcher` | #7 (v0.2) |

Las traits y los tipos compartidos (`DeviceId`, `LightingSpec`, etc.) viven en un crate nuevo: `opensynapse-core` (sin dependencias de backend).

## `opensynapse-core` (crate a crear)

```
crates/opensynapse-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── id.rs           # DeviceId, DeviceKind
    ├── spec.rs         # LightingSpec, MacroSpec, DpiSpec, AudioSpec
    ├── adapter.rs      # las traits
    └── error.rs        # AdapterError
```

Sin deps fuera de `tokio`, `async-trait`, `thiserror`, `futures` (para `BoxStream`). **No** depende de zbus ni de ningún backend.

Cada adapter crate depende de `opensynapse-core` y aporta su backend específico.

## Versionado

`pub const TRAIT_VERSION: u32 = 1;` en `opensynapse-core`. Cada adapter crate exporta el `TRAIT_VERSION` con el que fue compilado (constante asociada a una struct). El orchestrator lo verifica al cargar adapters (relevante si en algún momento adapters se cargan dinámicamente; hoy son `cargo` deps directas, así que la verificación es compile-time).

## Decisiones explícitas

| Decisión | Por qué |
|---|---|
| Per-capability traits, no `Adapter` monolítico | Permite que un crate cubra solo lo que sabe (PipeWire ≠ openrazer); evita métodos `unimplemented!()` por todos lados |
| Sin rollback en v0 | Spike #2 valida si es viable; introducirlo ahora a ciegas es over-engineering |
| Sin read-back de estado HW | El orchestrator es la fuente de verdad; pedirle al HW "¿qué tenés?" es lento, ruidoso y muchas veces el HW miente |
| Discovery on-demand + cache TTL | Hot-plug = v0.2; en v0.1 el costo de re-discover en cada scene change es trivial pero acumulable |
| Distinción transient vs fatal | Sin esto, el orchestrator no sabe si reintentar; con esto, decide sin meter retry policies en cada adapter |
| Orden de adapters significativo (`Vec`, no `HashMap`) | Permite preferencias del usuario ("para macros, primero input-remapper") sin código nuevo |
| `BoxStream` para foreground events | Único caso de "subscribe", abstrae la implementación; el costo de Box es irrelevante a esta tasa de eventos |
| `AudioAdapter`/`ForegroundWatcher` como `Option`, no `Vec` | Por definición no tiene sentido tener dos backends de audio activos a la vez |
