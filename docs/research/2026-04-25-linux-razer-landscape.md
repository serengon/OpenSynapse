# Linux Razer Landscape — Investigación inicial OpenSynapse

**Fecha:** 2026-04-25
**Objetivo:** mapear el soporte OSS actual para hardware Razer en Linux y cruzarlo contra el catálogo vigente de Razer (2025-2026), para decidir scope del MVP de OpenSynapse.

> **Método:** datos de actividad de repos consultados via API GitHub el 2026-04-25. El conteo de PIDs/modelos sale de los headers de drivers en `openrazer/openrazer@master` (`razerkbd_driver.h`, `razermouse_driver.h`, `razerkraken_driver.h`, `razeraccessory_driver.h`). Donde no encontré dato exacto lo digo.

---

## A) Stack OSS existente

| Proyecto | Rol | Última actividad | Última release | Estado |
|---|---|---|---|---|
| **openrazer/openrazer** | driver kernel + DBus daemon Python | push 2026-04-24 | `v3.12.2` (2026-04-18) | **muy activo**, base de facto, 4.3k stars |
| **polychromatic/polychromatic** | GUI Qt + tray + efectos | push 2026-04-14 | `v0.9.7` (2026-04-14) | **activo**, frontend recomendado |
| **z3ntu/RazerGenie** | GUI Qt alternativa | push 2026-04-07 | `v1.3.0` (2025-04-29) | mantenimiento ligero (1 release/año) |
| **z3ntu/razer_test** | reimplementación userspace | último commit 2020-12-08 | nunca estable | **abandonado** ("Started: Dec 2020, finished: Sep 2025 :)") |
| **mbuesch/razer (razercfg)** | driver+daemon paralelo, otro autor | push 2026-01-23 | — | mantenido pero cobertura HW limitada |
| **CalcProgrammer1/OpenRGB** | universal RGB control | push 2026-04-24 | nightly + 0.9 | **muy activo**, cubre Razer parcialmente vía protocolo Chroma propio |
| **libratbag/libratbag + piper** | mice (DPI/perfiles, no RGB) | libratbag 2026-01-11 / piper 2025-12-15 | — | mantenido pero soporte Razer reciente rezagado |
| **sezanzeb/input-remapper** | remapeo teclas/macros (X11+Wayland) | push 2026-04-19 | — | activo, ortogonal a Razer |
| **rvaiya/keyd** | remapeo a nivel evdev | push 2025-12-19 | — | activo |
| **xremap** | remapeo Wayland-friendly | activo | — | complemento |
| **razer-cli** + forks | wrapper CLI sobre openrazer | varios forks; ninguno canónico | — | utilitario menor |
| **razerCommander, Snake, chroma-feedback** | GUIs/utilidades menores listados en README de openrazer | bajo perfil | — | nicho |

**Qué hace cada uno:**

- **openrazer**: kernel module + `openrazer-daemon` (Python) que expone DBus. Cubre teclados, mice, mousepads RGB, headsets Kraken (no BlackShark/Barracuda), keypads Tartarus/Orbweaver, dock. **No** hace EQ audio, **no** firmware update, **no** per-app profiles, **no** Hypershift como abstracción (sí key remapping limitado). Conteo actual: **119** PIDs en kbd, **113** en mouse, **7** en kraken, **28** en accessories.
- **Polychromatic**: editor de efectos por dispositivo, tray, scripts ("Software Effects"), también hace de backend para OpenRGB. Sin Chroma Studio cross-device por layout físico.
- **RazerGenie**: equivalente más simple, menos pulido, menos efectos.
- **OpenRGB**: cubre periféricos Razer vía protocolo Chroma USB; también motherboards/RAM/GPU. Para Razer es subset de openrazer en cobertura, pero permite efectos sincronizados *cross-vendor*.
- **libratbag/piper**: configura DPI stages, polling rate, botones/macros. El soporte de mice Razer recientes (V3 Pro flagships) está incompleto.
- **input-remapper / keyd / xremap**: cubren la parte "keybinds y macros" de Synapse a nivel OS, sin saber del HW Razer. Útiles cuando el firmware no expone macros nativas en Linux.

**Embudo real:** si openrazer no agregó tu PID, el resto del stack no te ve. razer_test y muchos forks pequeños están muertos.

---

## B) Catálogo Razer 2025-2026 vigente

Modelos activos en razer.com a la fecha. No exhaustivo a nivel SKU.

### Teclados
- **BlackWidow**: V4 Pro (PID `0x028D`), V4 (`0x0287`), V4 X (`0x0293`), V4 75% (`0x02A5`), V4 Mini HyperSpeed wired/wireless (`0x02B9/0x02BA`), V4 TKL HyperSpeed wired/wireless (`0x02D7/0x02D5`)
- **Huntsman**: V3 Pro full (`0x02A6`), V3 Pro TKL (`0x02A7`), V3 Pro Mini (`0x02B0`), V3 Pro 8KHz (`0x02CF`), Huntsman Mini Analog (`0x0282`)
- **Ornata**: V3 (`0x02A1`), V3 X (`0x0294`), V3 X Alt (`0x02A2`), V3 TKL (`0x02A3`)
- **DeathStalker**: V2 (`0x0295`), V2 Pro wired/wireless (`0x0292/0x0290`), V2 Pro TKL wired/wireless (`0x0298/0x0296`)

### Mice
- **DeathAdder**: V3 (`0x00B2`), V3 Pro wired/wireless (`0x00B6/0x00B7`), V3 HyperSpeed wired/wireless (`0x00C4/0x00C5`), **V4 Pro** wired/wireless (`0x00BE/0x00BF`)
- **Basilisk**: V3 (lineage previo soportado), V3 X HyperSpeed (`0x00B9`), V3 35K (`0x00CB`), V3 Pro 35K wired/wireless (`0x00CC/0x00CD`), Basilisk Mobile (`0x00D3`)
- **Viper**: V3 HyperSpeed (`0x00B8`), V3 Pro wired/wireless (`0x00C0/0x00C1`)
- **Naga**: V2 HyperSpeed receiver (`0x00B4`), V2 Pro (en lineage)
- **Cobra**: Pro wired/wireless (`0x00AF/0x00B0`)
- **Pro Click V2** y V2 Vertical (`0x00C7..0x00D1`)
- HyperPolling Wireless Dongle (`0x00B3`)

### Headsets
- BlackShark V2 / V2 Pro (2023) / V2 X / V2 HyperSpeed
- Kraken V4 / V4 Pro / V4 X / Kraken Kitty V2 (`0x0560` en kraken driver)
- Barracuda / X / Pro
- Nari Ultimate (descontinuado, aún en uso — incluido el del propio user)

### Keypads
- Tartarus Pro (`0x0244`), Tartarus V2 (`0x022B`)

### Mousepads / accesorios RGB
- Firefly V2 (`0x0C04`), Firefly V2 Pro (`0x0C08`), Goliathus Chroma 3XL (`0x0C06`), Goliathus Extended (`0x0C02`), Strider Chroma (`0x0C05`)
- Mouse Bungee V3 Chroma (`0x0F1D`), Base Station V2 Chroma (`0x0F20`), Charging Pad Chroma (`0x0F26`), Laptop Stand Chroma V2 (`0x0F2B`), Thunderbolt 4 Dock Chroma (`0x0F21`)
- **Chroma Addressable RGB Controller** (`0x0F1F`) — ARGB hub

### Otros (Synapse-managed) — NO presentes en openrazer
- Kiyo Pro / Pro Ultra / X (cámaras UVC)
- Seiren V3 Chroma / V3 Mini / V3 Pro (mics)
- Razer Audio Mixer
- Edge / Kishi V2 (mobile, descartar para MVP)

---

## C) Cruce de cobertura (Razer vigente vs OSS)

Niveles: **(1)** detección + RGB básico; **(2)** RGB completo + efectos por zona; **(3)** macros/keybinds; **(4)** DPI/perfiles persistentes; **(5)** firmware update; **(6)** features avanzadas (EQ, Hypershift, sync cross-device).

| Categoría | Cobertura openrazer modelos vigentes | Mejor nivel | Comentario |
|---|---|---|---|
| Teclados V3/V4 wired | Bien: V4 Pro/X, Huntsman V3 Pro full/TKL/Mini, Ornata V3 family, BlackWidow V3 TK | 2-3 | Macros vía openrazer son limitadas; mejor combinar con keyd. |
| Teclados V4 HyperSpeed wireless | V4 Mini HyperSpeed y V4 TKL HyperSpeed presentes en master 2026; review issues abiertas para gaps de features | 1-2 | Detección sí; battery/wakeup parcial. |
| Mice wired V3 | DeathAdder V3 (no Pro) sí, Basilisk V3 35K sí, Viper V3 HyperSpeed sí, Cobra wired (no Pro) limitado | 2-4 con limitaciones | DPI sí; perfiles HW persistentes parciales. |
| Mice wireless flagships | DeathAdder V3 Pro / V4 Pro, Basilisk V3 Pro 35K, Viper V3 Pro, Cobra Pro, Naga V2 Pro: PIDs presentes pero **features avanzadas (HyperPolling 8K, profile sync, charging dock)** rezagadas | 1-3 | libratbag aún más rezagado. |
| Headsets Kraken | Kraken V2/TE/Ultimate/Kitty V2 vía `razerkraken_driver` (7 PIDs). **Kraken V4/V4 Pro: faltan en master** según headers consultados | 1-2 | RGB básico; sin EQ, sin THX Spatial, sin sidetone. |
| Headsets BlackShark | Sin soporte específico — UAC class. EQ y mic gain quedan al usuario via PipeWire/EasyEffects | 0-1 | **Gap grande.** |
| Headsets Barracuda | Idem; muchos features dependen del dongle HyperSpeed | 0-1 | Gap grande. |
| Keypads | Tartarus Pro y V2 soportados | 2-3 | Macros idealmente vía input-remapper. |
| Mousepads RGB | Firefly V2 / V2 Pro / Goliathus / Strider soportados | 2 | OK. |
| Mouse Bungee / Base Station / Charging Pad / Laptop Stand V2 / TB4 Dock | Todos con PID en master | 1-2 | Cobertura amplia. |
| Chroma ARGB Controller | PID `0x0F1F` presente | 1-2 | Sin la app de Synapse para mapear strips por zona. |
| Cámaras Kiyo | UVC standard. Sin control Razer-specific (HDR toggles, perfiles autofocus) | 0 (Razer-side) | UVC funciona. |
| Mics Seiren / Audio Mixer | Audio USB estándar. Sin gain/mute LED/efectos vía OSS Razer-aware | 0 (Razer-side) | Gap. |

**Estimación de cobertura por categoría (orientativa):**

- Teclados vigentes: **~85% detectados, ~60-70% con RGB nivel 2 completo**.
- Mice vigentes: **~80% detectados, wireless flagships con features parciales**.
- Headsets: **<30% con cualquier feature útil más allá de RGB básico (Kraken)**.
- Mousepads/accesorios RGB: **~90%** de los vigentes.
- Cámaras / mics / mixer: **~0% Razer-aware**.

---

## D) Gaps grandes — qué hace Synapse que NADIE hace en OSS

1. **Perfiles per-app (foreground-aware)**: cambiar DPI / RGB / macros según la app activa. Synapse lo hace nativo. Nadie en Linux tiene la pieza completa: openrazer no escucha foco, polychromatic tampoco. Hay que construirlo (X11 via `_NET_ACTIVE_WINDOW`, Wayland via portales / extensión KWin / GNOME).
2. **Chroma Studio (sync RGB cross-device por layout físico)**: pintar un "wave" que arranca en teclado y termina en mousepad. OpenRGB lo hace genérico pero sin la coreografía de Synapse. Polychromatic tiene "software effects" pero no editor visual cross-device.
3. **Audio EQ y THX Spatial / sidetone para BlackShark/Kraken V4/Barracuda**: cero soporte OSS Razer-aware. EasyEffects sirve pero no es per-device-aware ni guarda preset junto al perfil del juego.
4. **Firmware updates**: openrazer no hace flashing. fwupd LVFS — no encontré dato exacto, pero a la fecha no hay vendor stream Razer público en LVFS. Gap estructural; requiere reverse-engineering por device y/o cooperación del vendor.
5. **Hypershift (layer secundaria activada por tecla)**: openrazer no lo expone como abstracción. Implementable a nivel SO con keyd/xremap pero requiere integración con perfiles.
6. **HyperSpeed multi-device dongle**: un solo dongle para mouse + teclado wireless. Soporte OSS irregular; típicamente cada device aparece pero sincronización/battery/firmware del dongle se pierde.
7. **Chroma Apps SDK (efectos reactivos a juegos)**: HP bajo, cooldowns, killfeed. Existe `chromasdk` reverse-eng en Wine pero nada nativo Linux con catálogo de juegos.
8. **Mouse profile manager con calibración de superficie**: Synapse calibra; libratbag no lo expone para Razer.
9. **Battery dashboard unificado y notificaciones**: parcial en polychromatic, ausente en muchos modelos wireless modernos.
10. **HyperPolling 8K real**: Huntsman V3 Pro 8KHz y HyperPolling dongle tienen PID en openrazer pero el toggle de polling no siempre persiste sin el daemon de Synapse.

---

## Implicaciones para OpenSynapse — qué atacar primero

1. **No reinventar el driver. Apostar a openrazer como backend principal y contribuir PRs.** openrazer tiene 119/113/7/28 PIDs y release fresca (`v3.12.2` 2026-04-18). El camino de menor resistencia es: (a) detectar gaps de PID/feature en wireless flagships (DeathAdder V4 Pro, Basilisk V3 Pro 35K, Viper V3 Pro, Cobra Pro, V4 Mini/TKL HyperSpeed) y mandar PRs; (b) cubrir Kraken V4 que aún falta en `razerkraken_driver.h`.
2. **Construir el feature que NADIE tiene: perfiles per-app foreground-aware.** Es el diferenciador funcional más alto vs Polychromatic/RazerGenie. Diseñarlo desde el día 1 como capa independiente del driver para que sirva incluso a setups OpenRGB-only o multi-vendor — esto define la identidad de OpenSynapse.
3. **Cubrir el gap de audio sin reversear HW**: integrar EQ/sidetone para BlackShark V2 Pro, Kraken V4, Barracuda Pro vía PipeWire/WirePlumber con presets per-perfil. El audio es UAC estándar — el valor está en empaquetarlo dentro del mismo perfil que el RGB y los keybinds. Encaja con la preferencia de audio Windows-style del user (un solo default).
4. **MVP scope realista**: teclados V3/V4 wired + mice DeathAdder/Basilisk wired + Tartarus + Firefly. Dejar fuera Hypershift wireless flagships, firmware update y Chroma Studio cross-device para v0.2+. Los wireless flagships son el 80% del dolor por 20% del valor inicial.
5. **GUI Qt/QML coexistiendo con polychromatic, no compitiendo**. Comunidad pequeña; fragmentar es contraproducente. Importar perfiles polychromatic, exponer DBus compatible, hablar con el maintainer (lah7) temprano. Si OpenSynapse termina siendo "polychromatic + per-app + audio + Chroma Studio cross-device", todos ganan.

**Riesgos a vigilar**: (a) Razer cambia protocolo en cada generación — la deuda de PIDs nunca termina; (b) BlackShark/Barracuda/Kiyo/Seiren no exponen control fuera de Synapse Windows, lo que requiere reversing USB HID con Wireshark+usbmon (caro en tiempo); (c) firmware update sin vendor cooperation está esencialmente bloqueado.
