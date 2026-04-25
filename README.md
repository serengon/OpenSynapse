# OpenSynapse

Linux equivalent of Razer Synapse: una sola GUI para RGB, macros, remapeo de teclas y perfiles por aplicación, sobre dispositivos Razer (y potencialmente otros).

## Estado

Diseño inicial. Sin código todavía.

## Premisas

- **Userspace-first**: hablamos al hardware vía libusb/hidapi + reglas udev. Sin módulos kernel ni DKMS para el flujo por defecto. Ver [ADR 0001](docs/adr/0001-userspace-first.md).
- **Empaquetado simple**: GUI distribuida como Flatpak/AppImage, daemon como `.deb`/`.rpm` con `postinst` que deja todo listo (udev + grupo).
- **Reusar lo que existe**: integramos openrazer, input-remapper y herramientas propias (`nari-ctl`) en vez de reimplementar drivers.

## Alcance inicial

- Razer Nari Ultimate (1532:051a)
- Razer Tartarus Pro (1532:0244)

## Estructura

- `docs/adr/` — decisiones de arquitectura
