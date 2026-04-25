# ADR 0001 — Userspace-first, kernel module opcional

- Fecha: 2026-04-25
- Estado: Aceptado

## Contexto

OpenSynapse busca ser un reemplazo de Razer Synapse en Linux. La pregunta de arranque: ¿cómo hablamos al hardware?

Hay dos caminos:

1. **Kernel module** (lo que hace openrazer): expone el dispositivo vía sysfs/hidraw, integra botones como input devices "nativos" del sistema, batería por UPower, etc. Requiere DKMS, kernel headers, y se puede romper en cada actualización de kernel. Instalación frágil para usuarios no técnicos.
2. **Userspace** (libusb/hidapi + udev rules): el daemon habla USB directo en user-space. Solo requiere una regla udev que dé permisos al grupo correspondiente. Cero DKMS, cero headers, cero compilación contra kernel. Es lo que hace OpenRGB.

## Decisión

**Userspace-first.** El daemon por defecto usa hidapi/libusb. El kernel module queda como camino opcional para quienes quieran integración nativa profunda (mapeo de teclas como input device, integración UPower).

## Consecuencias

Positivas:

- Instalación trivial: Flatpak/AppImage + un `.deb` chico que solo deja una regla udev.
- No se rompe en kernel updates.
- Portabilidad entre distros mucho mayor.
- Cubre el 90% del caso de uso (RGB, macros, perfiles, lectura de batería por HID feature reports).

Negativas:

- Sin kernel module, las teclas del Tartarus no aparecen como input device nativo. Hay que emitir eventos vía `uinput` desde el daemon (que requiere su propia regla udev / capability).
- Batería visible solo dentro de OpenSynapse, no en el indicador estándar del DE — salvo que hagamos puente a UPower vía DBus.
- Para algunos efectos de iluminación reactivos al sistema podemos necesitar más viajes USB que con sysfs.

## Camino de migración

Si en el futuro queremos integración kernel: el daemon abstrae el backend (`HidapiBackend`, `SysfsBackend`), así que un usuario con openrazer instalado puede usar el sysfs backend automáticamente sin cambios en la GUI.
