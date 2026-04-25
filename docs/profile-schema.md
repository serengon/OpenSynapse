# Profile schema v0

Contrato central de OpenSynapse: define qué se persiste en disco, qué consumen los adapters y qué edita la GUI. Todo lo demás (adapter traits, orchestrator, importador Polychromatic, GUI) se diseña *contra* este schema.

Estado: **draft**. Cualquier cambio incompatible bumpea `schema_version`.

## Modelo conceptual

Dos entidades, dos niveles:

- **Device Profile** — bundle reusable acotado a *un* device (RGB, macros, DPI). Vive aislado: editar uno no afecta otros. Análogo al "Profile" de Razer Synapse.
- **Scene** — bundle global que dice *"cuando matchea esta condición, aplicá estos device profiles + esta config de audio"*. Análogo al "Config" de Razer Synapse 3.

El usuario puede tener:
- **Modo global**: una sola scene activa siempre (no usa matching).
- **Modo per-app**: múltiples scenes con reglas de match, orchestrator pickea según ventana activa.

Los dos modos coexisten sin conflicto: el modo global es solo una scene con `priority = 0` y `match = []` (matchea todo, último recurso).

## Layout en disco

```
~/.config/opensynapse/
├── devices/
│   ├── tartarus-coding.toml
│   ├── tartarus-gaming.toml
│   ├── nari-quiet.toml
│   └── nari-loud.toml
├── scenes/
│   ├── default.toml
│   ├── coding.toml
│   └── gaming.toml
└── opensynapse.toml          # config global del daemon (no es un perfil)
```

Naming: kebab-case, `name` interno = filename sin extensión. El orchestrator valida que `name` interno y filename coincidan al cargar.

Formato: TOML (decisión de esta sesión: editable a mano, diff-friendly, idiomático Rust/serde, comentarios soportados).

## Device Profile

```toml
schema_version = 1

[profile]
name = "tartarus-coding"           # debe coincidir con el filename
description = "Verde estático + macros vim"

# Constrain a qué device(s) este profile aplica.
# vid+pid son requeridos (filter inicial).
# serial es opcional (si presente, restringe a ESA unidad concreta).
[device_match]
vid = 0x1532
pid = 0x0244
# serial = "PM1940F36403322"        # opcional

# Cada sección es OPCIONAL. Solo las que estén presentes se aplican.
# Las ausentes = "no toques eso".

[lighting]
mode = "static"                    # static | breathing | spectrum | wave | reactive | none
color = "#00ff00"                  # solo si mode requiere color
brightness = 80                    # 0..100, opcional
# regiones (logo, scroll, backlight) se modelan en v1 cuando exista un device que las distinga; v0 trata el device como una sola región.

[[macros]]
key = "M1"
sequence = [
  { type = "key", value = "ctrl+w" },
  { type = "delay", ms = 50 },
  { type = "key", value = "h" },
]

[[macros]]
key = "M2"
sequence = [{ type = "text", value = "git status\n" }]

[dpi]                              # solo aplicable a mice; ignorado en otros device kinds
stages = [800, 1600, 3200]
active_stage = 1                   # índice 0..N
```

### Reglas

- `device_match.vid` + `device_match.pid` requeridos.
- Una sección ausente significa "no aplicar" (NO "resetear a default"). El orchestrator no toca lo que el profile no menciona.
- Si una sección está presente pero el device no la soporta (ej: `[dpi]` en un teclado), el adapter loguea warning y skipea esa sección — no falla todo el profile.

## Scene

```toml
schema_version = 1

[scene]
name = "coding"
description = "VS Code, Alacritty"
priority = 100                     # mayor = más específico; default = 0

# Si match es vacío o ausente, la scene matchea siempre (fallback).
# Múltiples [[match]] = OR. Dentro de un match, todos los campos = AND.
[[match]]
wm_class = "Code"                  # match exacto (case-sensitive como X11)

[[match]]
wm_class = "Alacritty"

# Bindings device → device_profile. Key = alias lógico (string libre);
# el orchestrator resuelve el alias a un device físico vía device_aliases.toml
# (ver "Aliases" abajo).
[devices]
tartarus = { profile = "tartarus-coding", required = false }
nari     = { profile = "nari-quiet",      required = true  }

# Audio: cualquier campo ausente = "no cambiar".
[audio]
default_sink = "alsa_output.usb-Razer_Razer_Nari_Ultimate-00.analog-stereo"
sidetone_db = -20.0
# eq_preset = "voice"              # nombre de preset definido aparte (v0.2)
```

### Required vs optional

- `required = true` + device ausente → la scene **no se activa** y el orchestrator pasa a la siguiente scene de menor prioridad que matchee.
- `required = false` (o ausente) + device ausente → la scene se activa, ese device se skipea con un log warning.
- Si la scene no tiene devices `required` y ninguno está presente, igual se activa (puede aplicar solo audio, por ej.).

## Aliases de devices

Para que las scenes sean portables (no hardcodean un serial específico), los nombres como `tartarus` o `nari` se resuelven vía un archivo aparte:

```toml
# ~/.config/opensynapse/devices/_aliases.toml
[tartarus]
vid = 0x1532
pid = 0x0244
# serial = "PM1940F36403322"        # opcional, para distinguir si hay 2

[nari]
vid = 0x1532
pid = 0x051a
```

El orchestrator carga aliases al startup. Una scene que referencia un alias no resuelto = error de validación (loguear, no aplicar la scene).

## Matching: algoritmo

Pseudocódigo del orchestrator cuando cambia la ventana activa:

```
on foreground_change(wm_class, title):
    candidates = scenes
        .filter(s => s.match.empty() || s.match.any(m => matches(m, wm_class, title)))
        .sort_by(s => -s.priority)        # desc

    for scene in candidates:
        if all(d.required => d.alias.is_present() for d in scene.devices):
            apply(scene)
            return

    # Ningún candidate aplicable: dejar el estado anterior.
    log("no scene applied")
```

Match dentro de un `[[match]]` block:
- Todos los campos presentes deben coincidir (AND).
- Campos soportados en v0: `wm_class` (string exacto), `wm_class_regex` (regex, lazy compile).
- `title` y otros se suman cuando alguien los pida (no hay caso de uso real ahora).

Empate de prioridad: el de menor `name` lex order gana (determinístico, sin sorpresas).

## Validación

Al cargar un perfil/scene, el daemon valida:
1. Schema bien formado (serde).
2. `name` interno = filename.
3. `device_match.vid/pid` presentes en device profiles.
4. `priority >= 0`.
5. Aliases referenciados en `[devices]` existen en `_aliases.toml`.
6. `device_profile` referenciado en `[devices]` existe en disco.
7. `mode` en `[lighting]` es uno de los enums conocidos.

Errores → log estructurado, perfil/scene descartado de la lista activa, no abort del daemon.

## Versionado

`schema_version = 1` en cada archivo. El daemon reconoce solo su versión actual + 1 anterior (con migración automática). Bumps:
- **Minor** (campos nuevos opcionales): no bumpea version. Daemon viejo ignora campos que no conoce.
- **Major** (cambio de semántica o campos requeridos nuevos): bumpea version, daemon viejo rechaza con error claro.

## Out of scope v0 (explícito)

- Layered profiles (stacks composables) — modelo "scene + device profile" cubre el caso real.
- Per-region lighting (logo vs scroll vs backlight) — agregar cuando un device dogfooded lo demande.
- Time-based matching (`when: hour > 18`) — sin caso de uso real.
- Hot-reload de archivos — el daemon recarga al recibir signal SIGUSR1 o vía CLI; auto-watch de FS = v0.2.
- Encrypted profiles — sin caso de uso.
- EQ presets concretos — esquema declara el campo `eq_preset` como string opaco; los presets viven aparte (v0.2 con PipeWire adapter).

## Decisiones explícitas y por qué

| Decisión | Por qué |
|---|---|
| TOML, un archivo por entidad | Decisión de la sesión: editable, diff-friendly, idiomático serde |
| Dos niveles (device profile + scene) | Pedido explícito del usuario (Synapse-style: global + per-device) |
| `required` vs `optional` por device en scene | Decisión de la sesión: balance entre atomicidad y usabilidad |
| Match por `wm_class` + priority | Decisión de la sesión: simple, predecible, sin DSL |
| Aliases en archivo aparte | Hace las scenes portables entre máquinas; el alias mapea a HW local |
| Sección ausente = "no tocar" (no "default") | Permite componer perfiles sin que se pisen entre sí |
| Validación tolerante (skip + log, no abort) | El daemon corre desatendido; un perfil roto no debe tirar todo |
