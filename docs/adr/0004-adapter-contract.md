# ADR 0004 — Adapter contract: traits per-capability, sin rollback en v0

- Fecha: 2026-04-25
- Estado: Aceptado

## Contexto

ADR 0002 definió que OpenSynapse delega el control de HW a backends existentes (openrazer, libratbag, PipeWire, input-remapper). Falta definir el **contrato programático** entre el orchestrator y esos backends — es decir, las traits Rust que cada adapter implementa.

Dos preguntas centrales:

1. **¿Una trait monolítica o varias por capability?**
   - **Monolítica**: una trait `Adapter` con todos los métodos (`apply_lighting`, `apply_macros`, `apply_audio`, ...). Cada adapter implementa los que cubre y devuelve `UnsupportedCapability` en el resto.
   - **Per-capability**: una trait por dominio (`LightingAdapter`, `MacroAdapter`, `AudioAdapter`, ...). Cada adapter implementa solo las traits que cubre.

2. **¿Cómo se manejan apply parciales fallidos?**
   - **Atómico con rollback**: si una capability falla a la mitad de un scene apply, revertir las ya aplicadas.
   - **Best-effort**: aplicar lo que se pueda, loguear lo que falló, no revertir.

Restricciones reales:
- openrazer **no expone primitivas transaccionales** sobre DBus. Cualquier "rollback" que implementemos sería emulado guardando estado previo y re-aplicándolo, lo cual es frágil (el estado previo puede ya no ser válido si otro proceso tocó el HW o cambió el device).
- En el spike #1 ya validamos que cada call DBus es independiente y puede fallar individualmente sin afectar a las otras. No hay batch.
- Los backends son heterogéneos: PipeWire no tiene noción de "device" como openrazer; input-remapper solo cubre macros; libratbag solo cubre mice. Una trait monolítica forzaría `unimplemented!()` por todos lados.

## Decisión

**Traits per-capability**, ortogonales y composables.

```rust
pub trait DeviceDiscovery { ... }
pub trait LightingAdapter { ... }
pub trait MacroAdapter   { ... }
pub trait DpiAdapter     { ... }
pub trait AudioAdapter   { ... }
pub trait ForegroundWatcher { ... }
```

Un adapter implementa solo las que cubre:
- `OpenrazerAdapter`: `DeviceDiscovery + LightingAdapter + MacroAdapter + DpiAdapter`
- `PipewireAdapter`: `AudioAdapter`
- `InputRemapperAdapter`: `MacroAdapter`
- `LibratbagAdapter`: `DpiAdapter`
- `ForegroundWatcherX11`: `ForegroundWatcher`

Las traits y los tipos compartidos (`DeviceId`, `LightingSpec`, `MacroSpec`, etc.) viven en un crate nuevo `opensynapse-core` **sin dependencias de backends**. Cada adapter crate depende de `opensynapse-core` y aporta su backend.

**Sin rollback en v0.** El orchestrator aplica capabilities en orden determinístico. Si una falla:
- **Transient** (timeout, busy): un retry. Si vuelve a fallar, log warning, seguir con las siguientes.
- **Fatal** (parse error, backend muerto): log error, seguir con las siguientes.
- **UnsupportedCapability**: log warning, skipear esa capability.
- El estado HW queda donde quedó. El orchestrator marca la scene como "aplicada con errores" en su estado interno.

El spike #2 (openrazer write) valida si esto es aceptable en la práctica. Si los apply parciales generan estados HW inconsistentes que confunden al usuario, se introduce un trait opcional `Transactional` que adapters capaces puedan implementar.

Decisiones secundarias dentro de este contrato (ver [`docs/adapter-traits.md`](../adapter-traits.md) para detalle):

- **Sin read-back de estado HW**. El orchestrator es la fuente de verdad de "qué scene está activa". Pedirle al HW "¿qué color tenés?" es lento y muchas veces miente.
- **Discovery on-demand con cache TTL** (~5s). Hot-plug = v0.2.
- **Registry como `Vec<Arc<dyn Trait>>` ordenado** (no `HashMap`). El orden es significativo: el primer adapter que pueda manejar el device gana. Permite preferencias del usuario ("macros via input-remapper antes que openrazer") sin código nuevo.
- **`AudioAdapter` y `ForegroundWatcher` como `Option`, no `Vec`**. Por definición no hay "dos backends de audio activos a la vez".
- **Errores con distinción `Transient` vs `Fatal`**, para que el orchestrator pueda decidir reintentos sin meter retry policies en cada adapter.

## Consecuencias

Positivas:

- Adapters honestos: un crate solo implementa lo que sabe, el compilador valida el wiring.
- Agregar una capability nueva (ej: macropad layers) = una trait nueva en `opensynapse-core` + implementaciones donde corresponda. No hay un mega-trait para ampliar.
- Backends intercambiables por capability: input-remapper o openrazer para macros, según cuál soporte mejor el HW del usuario, sin tocar el orchestrator.
- Ordenamiento del registry como mecanismo de preferencia evita reinventar un sistema de prioridades.
- Sin rollback = código v0 mucho más simple. Cuando spike #2 nos diga si vale la pena, se introduce con dato real.

Negativas:

- Más boilerplate al implementar un adapter que cubre varias capabilities: un `impl Trait for X` por trait. Mitigación: trivial; los métodos son pocos.
- "Best-effort apply" puede dejar el HW en estado mezcla (RGB nuevo, macros viejas) si algo falla. Mitigación: log claro al usuario + el orchestrator marca la scene como "parcial". El próximo scene change lo va a normalizar.
- Sin read-back, si algo externo cambia el HW (otro proceso, el usuario apretando un botón físico), el orchestrator no se entera. Mitigación: aceptado como tradeoff; v0 asume "OpenSynapse es el único que toca el HW". Las situaciones reales que rompen esto se evalúan post-spike #2.
- Crate `opensynapse-core` extra para mantener. Mitigación: vive y muere con el workspace; no se publica independiente.

## Camino de migración

`pub const TRAIT_VERSION: u32 = 1;` en `opensynapse-core`. Si una trait cambia de forma incompatible:

- **Adición de método con default impl**: minor, no bumpea version.
- **Cambio de signature o nuevo método sin default**: major, bumpea version. Los adapters tienen que recompilarse contra el nuevo core. Como hoy son `cargo` deps directas en el workspace, el compile fail es inmediato; cuando haya adapters externos, ese check se vuelve runtime.

Si el spike #2 demuestra que rollback es viable y necesario, se introduce trait `Transactional` opcional sin tocar las traits existentes — adapters capaces lo implementan, el orchestrator lo detecta vía downcast y usa transactional path cuando todos los adapters involucrados lo soportan; fallback a best-effort cuando no.
