# Roadmap

Basado en [`docs/research/2026-04-25-linux-razer-landscape.md`](research/2026-04-25-linux-razer-landscape.md) y [`docs/adr/0001-userspace-first.md`](adr/0001-userspace-first.md).

## Principios de scope

- **No reinventar driver.** Backend principal = openrazer (DBus). Cubrimos gaps con PRs upstream, no con un fork.
- **Diferenciador = lo que nadie hace en Linux**, no "otra GUI más linda". Foreground-aware per-app profiles, audio integrado al perfil, Chroma Studio cross-device.
- **Dogfood primero**: el user tiene Nari Ultimate + Tartarus Pro. Todo lo que no se pueda probar acá se posterga.
- **Userspace-first** (ADR 0001).

---

## v0.1 — MVP "una sola GUI que vale la pena"

**Objetivo**: reemplazar a Polychromatic en el flujo diario del user, y agregar 1 feature que Polychromatic no tiene.

Scope HW:
- Razer Nari Ultimate (vía nuestro fork openrazer `add-nari-ultimate`)
- Razer Tartarus Pro (openrazer mainline)

Features:
1. **GUI Qt/QML** que lista dispositivos via openrazer DBus, muestra batería, edita RGB básico, y dispara macros.
2. **Per-app profile switcher** (X11 primero, vía `_NET_ACTIVE_WINDOW`). Cambio automático de perfil cuando la ventana en foco coincide con regla. Esta es **la** apuesta de identidad. Wayland se difiere a v0.2.
3. **Importador de perfiles Polychromatic** — no fragmentar comunidad.
4. **Empaquetado**: AppImage o Flatpak para la GUI; el daemon openrazer queda como dependencia documentada (no lo redistribuimos).

Out of scope v0.1: audio, Chroma Studio cross-device, firmware update, wireless flagships, Wayland.

Criterio de "done":
- Cambio de perfil automático funciona al cambiar entre 3 apps de prueba (terminal, navegador, juego).
- Macro del Tartarus se dispara correctamente.
- RGB del Nari se cambia desde la GUI.
- Importa al menos 1 perfil Polychromatic existente del user.

---

## v0.2 — Audio per-perfil

Scope HW: sumar BlackShark V2 Pro o Kraken V4 (depende de qué se consiga prestado para testear; el Nari del user no tiene EQ vía HID).

Features:
1. **Integración PipeWire/WirePlumber** — cada perfil OpenSynapse incluye preset de EQ + sidetone + sink default. Audio Windows-style: un solo default, todo lo sigue (alineado con preferencia del user).
2. **Soporte Wayland** del per-app switcher (KWin script + portal `org.freedesktop.portal.Inhibit`/`ScreenCast`).
3. **Battery dashboard** unificado con notificaciones de low battery.

---

## v0.3+ — Extender alcance

Lista de candidatos, prioridad a definir según uso real:

- **Chroma Studio cross-device** (editor visual de efectos sincronizados sobre layout físico).
- **HyperSpeed multi-device dongle** support con detección y battery sync.
- **Mouse calibration / DPI stages** vía libratbag para Basilisk/DeathAdder.
- **Hypershift** como abstracción (layer secundaria por hold-key).
- **Contribuciones upstream openrazer**: Kraken V4 PID en `razerkraken_driver.h`, gaps de wireless flagships.

Excluido por costo/beneficio:
- **Firmware update**: bloqueado sin cooperación de Razer / LVFS.
- **Chroma Apps SDK** (efectos reactivos a juegos).
- **Cámaras Kiyo / mics Seiren / Audio Mixer**: requieren reversing HID propio; no hay base OSS.

---

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Razer cambia protocolo cada generación → deuda de PIDs perpetua | Backend abstracto; openrazer absorbe la deuda, nosotros consumimos su DBus |
| BlackShark/Barracuda/Kraken V4 sin soporte HID Razer-aware | v0.2 trabaja sobre UAC + PipeWire, sin tocar HID |
| Fragmentación con Polychromatic | Importador de perfiles + diálogo temprano con maintainer (lah7) |
| Wayland fricción con foreground tracking | X11 primero; KWin script como puente para KDE (entorno del user) |
