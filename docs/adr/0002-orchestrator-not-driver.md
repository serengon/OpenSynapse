# ADR 0002 — OpenSynapse es orquestador, no driver

- Fecha: 2026-04-25
- Estado: Aceptado

## Contexto

La investigación inicial ([landscape](../research/2026-04-25-linux-razer-landscape.md)) muestra que openrazer es el stack de facto para HW Razer en Linux: activo (`v3.12.2` 2026-04-18), 119 PIDs de teclados / 113 de mice / 7 de Kraken / 28 de accessories, con DBus daemon estable. Polychromatic y RazerGenie ya cubren la GUI de RGB sobre openrazer.

Construir otro driver, otro daemon, u otra GUI-de-RGB-más sería duplicar trabajo y fragmentar comunidad chica.

Lo que sí no existe en Linux y Synapse hace nativo en Windows:
- Perfiles que cambian solo según app activa (foreground-aware).
- Un perfil = bundle atómico de RGB + macros + DPI + audio + sidetone.
- Sincronización entre componentes que hoy en Linux viven aislados (openrazer, input-remapper/keyd, libratbag, PipeWire).

## Decisión

OpenSynapse es **un orquestador de perfiles cross-componente** con GUI Qt encima. Concretamente:

- **No** escribimos driver. Backend HW = openrazer DBus (y libratbag para mice cuando sume valor).
- **No** competimos con Polychromatic en "editor de RGB". Importamos sus perfiles y, si el feature ya existe ahí, lo delegamos o lo replicamos sin reinvención.
- **Sí** somos dueños de:
  1. **Profile engine**: define qué es un perfil OpenSynapse (RGB + macros + audio + DPI + reglas de activación) y persiste/importa/exporta.
  2. **Foreground watcher**: detecta ventana activa (X11 `_NET_ACTIVE_WINDOW`, Wayland vía portal/KWin) y dispara cambios de perfil.
  3. **Adaptadores**: openrazer (DBus), input-remapper o keyd (IPC/CLI), PipeWire/WirePlumber (presets de EQ + sink default + sidetone), libratbag (DPI).
  4. **GUI Qt/QML** que expone el modelo unificado.

## Consecuencias

Positivas:
- Scope acotado: el código nuestro es lógica de orquestación + UI, no kernel ni protocolo USB.
- Beneficio inmediato para usuarios que ya tienen openrazer instalado.
- Cualquier mejora en openrazer (PIDs nuevos, features nuevas) nos llega gratis.
- Identidad clara: "el Synapse que faltaba" en lugar de "otra GUI Razer".

Negativas:
- Dependencia dura de openrazer. Si openrazer no soporta un dispositivo o feature, nosotros tampoco — el camino es contribuir upstream, no parchear local.
- Cualquier breaking change en DBus de openrazer nos rompe. Mitigación: pinear versión mínima soportada y testear contra `master`.
- Necesitamos relación sana con maintainers (lah7 en Polychromatic, terrycain/z3ntu en openrazer) para no fragmentar.

## No-objetivos explícitos

OpenSynapse **no** va a:
- Mantener un fork propio del kernel module openrazer (excepto el branch experimental Nari, que apunta a mergear upstream).
- Reimplementar control RGB que Polychromatic ya hace bien.
- Hacer firmware updates (bloqueado sin LVFS/Razer cooperation).
- Soportar HW no-Razer en v0.x (aunque la arquitectura de adaptadores no lo impide a futuro).
