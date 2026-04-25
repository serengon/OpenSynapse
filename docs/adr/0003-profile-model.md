# ADR 0003 — Profile model: dos niveles (device profile + scene)

- Fecha: 2026-04-25
- Estado: Aceptado

## Contexto

OpenSynapse necesita un modelo de "perfil" persistible en disco que la GUI edite, los adapters consuman y el orchestrator dispare según contexto (ventana activa, hora, manual). La decisión de cómo modelar esto condiciona todo: importador Polychromatic, GUI, schema en disco, semántica del orchestrator.

Tres alternativas reales sobre la mesa:

1. **Bundle global por app**: un Profile = todo el estado deseable cuando esa app está activa (RGB de cada device + macros + DPI + audio). Atomicidad simple, mental model claro.
2. **Layers componibles**: un Profile = stack de layers que se mergean por precedencia. Flexible para compartir piezas. Costo: complejidad de merge, difícil de explicar.
3. **Per-device profiles**: un Profile vive a nivel device y se agrupan en "scenes". Parecido a Polychromatic, pero pierde el "un trigger → todo cambia".

El usuario pidió explícitamente que coexistan **modo global y modo per-device**, citando que Razer Synapse Windows ofrece ambos: un Profile per-device (RGB + macros + DPI de un device) y un Config global que asocia profiles + audio + reglas a una app.

## Decisión

Dos entidades en disco, dos niveles:

- **Device Profile** — bundle reusable acotado a un device (lighting + macros + DPI). Vive aislado.
- **Scene** — bundle global que referencia device profiles por alias + audio + reglas de match (`wm_class` + priority).

Coexistencia de modos:
- **Modo global**: una sola scene con `match = []` (matchea siempre) y `priority = 0`.
- **Modo per-app**: múltiples scenes con reglas; el orchestrator pickea la de mayor priority cuyas condiciones matcheen.

Schema completo, layout en disco, algoritmo de matching y reglas de validación: ver [`docs/profile-schema.md`](../profile-schema.md).

Decisiones secundarias que viven dentro de este modelo:
- **TOML**, un archivo por entidad. Editable a mano, diff-friendly, idiomático Rust/serde.
- **Aliases en archivo aparte** (`devices/_aliases.toml`) → scenes son portables entre máquinas sin hardcodear seriales.
- **Sección ausente = "no tocar"** (no "resetear a default") → permite componer sin pisarse.
- **`required` vs `optional`** por device en cada scene → balance entre atomicidad y usabilidad cuando un device falta.
- **Match por `wm_class` exacto + priority entero**, sin DSL → simple, predecible, suficiente para v0.1.
- **Validación tolerante**: perfil inválido se loguea y se descarta; el daemon sigue corriendo.

## Consecuencias

Positivas:

- Cubre los dos modos que el usuario quería sin ramificar el código del orchestrator (un modo es un caso particular del otro).
- Device profiles son reusables entre scenes (un `tartarus-coding` puede aparecer en `coding`, `writing`, `default`) — menos duplicación.
- Aliases hacen las scenes portables: un usuario puede compartir su scene `coding.toml` y otro la usa con sus propios devices al mapear los aliases.
- Estructura mapea bien a la GUI esperada: pestañas por device + un editor de "scenes" que combina y agrega audio/match.
- Importador Polychromatic mapea naturalmente a Device Profile (Polychromatic ya razona per-device).

Negativas:

- Dos archivos para cambiar el RGB de un device en un contexto específico (editar el Device Profile o crear uno nuevo + actualizar la Scene). Mitigación: la GUI puede ofrecer "edit inline" que crea un device profile derivado automáticamente.
- Aliases agregan un nivel de indirección que un usuario nuevo tiene que entender. Mitigación: el daemon autogenera `_aliases.toml` con los devices descubiertos al primer run; el usuario solo lo edita si quiere.
- "Sección ausente = no tocar" puede confundir ("¿por qué cuando cambio de scene el RGB se queda?"). Mitigación: documentar fuerte y, en GUI, mostrar explícitamente "esta scene no toca el RGB del Tartarus".

## Camino de migración

`schema_version = 1` en cada archivo. Si en el futuro el modelo cambia de forma incompatible:

- **Cambios menores** (campos opcionales nuevos): no bumpea versión; daemons viejos los ignoran.
- **Cambios mayores**: bump de versión + migrador automático one-shot al startup. Daemon viejo rechaza con error claro en vez de pisar archivos.
