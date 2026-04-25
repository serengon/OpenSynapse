# OpenSynapse

Linux equivalent of Razer Synapse: una sola GUI para RGB, macros, remapeo de teclas y perfiles por aplicación, sobre dispositivos Razer (y potencialmente otros).

## Estado

v0.1 MVP funcional. Daemon CLI sin GUI todavía.

Lo que ya funciona:

- Detecta dispositivos Razer vía openrazer DBus.
- Cambia automáticamente el RGB del Tartarus Pro según la aplicación en foreground (X11).
- Configs en TOML editables a mano (`~/.config/opensynapse/`).
- Genera config inicial al primer arranque (`_aliases.toml` con tus devices descubiertos + scene fallback).

## Premisas

- **Userspace-first**: hablamos al hardware vía openrazer (DBus). Sin módulos kernel ni DKMS para el flujo por defecto. Ver [ADR 0001](docs/adr/0001-userspace-first.md) y [ADR 0002](docs/adr/0002-orchestrator-not-driver.md).
- **Empaquetado simple** (futuro): GUI distribuida como Flatpak/AppImage, daemon como `.deb`/`.rpm`.
- **Reusar lo que existe**: openrazer, input-remapper, PipeWire — orquestamos, no reimplementamos.

## Quickstart

Pre-requisitos:
- `openrazer-daemon` instalado y corriendo (`systemctl --user status openrazer-daemon`).
- Sesión X11 (Wayland llega en v0.2).
- Toolchain Rust stable.

```bash
git clone https://github.com/serengon/OpenSynapse.git
cd OpenSynapse
cargo run -p opensynapse-daemon --bin opensynapsed
```

Primer arranque: detecta tus devices, escribe `~/.config/opensynapse/{devices,scenes}/` con un alias por device y una scene fallback `default` (cyan estático). Cambiá de ventana → no pasa nada porque solo hay una scene.

Para ver el switching real, agregá scenes per-app. Ejemplo (`~/.config/opensynapse/`):

```toml
# devices/tartarus-coding.toml
schema_version = 1
[profile]
name = "tartarus-coding"
[device_match]
vid = 0x1532
pid = 0x0244
[lighting]
mode = "static"
color = "#00ff00"
brightness = 80
```

```toml
# scenes/coding.toml
schema_version = 1
[scene]
name = "coding"
priority = 100

[[match]]
wm_class = "kitty"
[[match]]
wm_class = "Code"

[devices]
tartarus = { profile = "tartarus-coding", required = false }
```

Reiniciá el daemon. Cambiá entre kitty y otra ventana — el Tartarus debería ponerse verde y volver a cyan.

Logs verbosos: `RUST_LOG=opensynapsed=debug cargo run -p opensynapse-daemon --bin opensynapsed`.

## Binarios incluidos

| Binario | Para qué |
|---|---|
| `opensynapsed` | El daemon orquestador (lo que querés correr) |
| `openrazer-probe` | Lista devices vía openrazer y muestra batería |
| `openrazer-apply` | Aplica un mode RGB ad-hoc a un device. `openrazer-apply 1532 0244 static 00ff00` |
| `fg-watch` | Loguea cambios de ventana activa (para debug del watcher X11) |

## Arquitectura y decisiones

- [`docs/spikes.md`](docs/spikes.md) — mapa vivo de spikes y dependencias.
- [`docs/profile-schema.md`](docs/profile-schema.md) — schema de perfiles en disco (TOML, dos niveles).
- [`docs/adapter-traits.md`](docs/adapter-traits.md) — contrato entre orchestrator y backends.
- [`docs/adr/`](docs/adr/) — decisiones arquitectónicas (userspace-first, orchestrator-not-driver, profile model, adapter contract).

## Roadmap

Ver [`docs/roadmap.md`](docs/roadmap.md). Próximo: macros (input-remapper o keyd), importador de perfiles Polychromatic, GUI Qt/QML, soporte Wayland, audio per-perfil (PipeWire).

## Estructura del repo

```
crates/
├── opensynapse-core/         # traits + specs compartidos, sin deps de backend
├── openrazer-adapter/        # adapter sobre openrazer DBus
├── foreground-watcher-x11/   # watcher de _NET_ACTIVE_WINDOW
└── opensynapse-daemon/       # binario opensynapsed: orchestrator + loader + bootstrap
docs/
├── adr/                      # decisiones de arquitectura
├── research/                 # notas de investigación
└── *.md                      # spikes, schema, traits, roadmap
```

## Hardware testeado

- Razer Nari Ultimate (1532:051a) — vía fork [openrazer/add-nari-ultimate](https://github.com/openrazer/openrazer/issues/974).
- Razer Tartarus Pro (1532:0244) — openrazer mainline.
