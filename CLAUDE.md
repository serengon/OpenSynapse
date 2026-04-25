# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Estado del repositorio

Diseño inicial, **sin código todavía**. Solo hay documentación (`README.md`, `docs/adr/`, `docs/roadmap.md`, `docs/research/`). No hay build, tests, ni lint configurados aún. Cuando se agregue código, actualizar esta sección.

## Qué es OpenSynapse

Equivalente Linux de Razer Synapse: una sola GUI para RGB, macros, remapeo, perfiles por aplicación y audio sobre dispositivos Razer.

**Posicionamiento clave (ADR 0002):** OpenSynapse es un **orquestador de perfiles cross-componente**, no un driver ni "otra GUI de RGB". El backend HW es **openrazer (DBus)**; no escribimos protocolo USB propio salvo casos puntuales (`nari-ctl`). No competimos con Polychromatic — importamos sus perfiles.

## Decisiones arquitectónicas que condicionan cualquier código futuro

Leer los ADR antes de proponer cambios estructurales:

- **`docs/adr/0001-userspace-first.md`** — daemon habla HW vía hidapi/libusb + udev. Sin DKMS ni kernel module en el flujo por defecto. El backend está abstraído (`HidapiBackend`, `SysfsBackend`) por si después se quiere integración sysfs.
- **`docs/adr/0002-orchestrator-not-driver.md`** — *supersede en parte* la dirección de 0001: el backend HW real es **openrazer DBus** (más libratbag para mice cuando aporte). Lo nuestro es lógica de orquestación + UI Qt/QML, no USB. Cualquier feature que openrazer no soporta → PR upstream, no fork local (excepción: branch `add-nari-ultimate`).

Componentes que el código debe orquestar (no reimplementar):
- **openrazer** (DBus) — RGB, macros, batería Razer.
- **input-remapper / keyd** (IPC/CLI) — remapeo de teclas.
- **PipeWire / WirePlumber** — presets de EQ, sink default, sidetone.
- **libratbag** — DPI en mice.

Lo que **sí** es propio:
1. **Profile engine** — un perfil = bundle atómico de RGB + macros + DPI + audio + reglas de activación; persistencia, import/export.
2. **Foreground watcher** — detecta ventana activa (X11 `_NET_ACTIVE_WINDOW` en v0.1; Wayland vía portal/KWin script en v0.2) y dispara cambios de perfil. Esta es la apuesta de identidad — Synapse en Windows lo hace, nadie en Linux lo hace bien.
3. **Adaptadores** hacia los componentes de arriba.
4. **GUI Qt/QML**.

## Hardware de referencia (dogfood)

- Razer Nari Ultimate (`1532:051a`) — vía fork openrazer `add-nari-ultimate`.
- Razer Tartarus Pro (`1532:0244`) — openrazer mainline.

Todo lo que no se pueda probar en este HW se posterga. Ver `docs/roadmap.md` para el plan v0.1 → v0.3.

## No-objetivos explícitos

No proponer trabajo en estas áreas (ADR 0002 §No-objetivos):
- Fork propio del kernel module openrazer (excepto branch Nari).
- Reimplementar lo que Polychromatic ya hace bien (editor RGB).
- Firmware updates (bloqueado sin LVFS / cooperación Razer).
- HW no-Razer en v0.x.
- Chroma Apps SDK, cámaras Kiyo, mics Seiren, Audio Mixer (sin base OSS).

## Convenciones

- Documentación en español (README, ADR, roadmap están en español; mantener consistencia).
- ADRs numerados secuencialmente en `docs/adr/NNNN-slug.md` con campos `Fecha` y `Estado`.
- Research notes fechadas en `docs/research/YYYY-MM-DD-slug.md`.
