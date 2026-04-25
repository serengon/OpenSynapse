# Mapa de spikes

Lista viva. Cada spike responde **una** pregunta de viabilidad sobre una dependencia o capacidad. Si la respuesta es "no", se replantea el roadmap antes de seguir. No incluye trabajo de producto (eso vive en `roadmap.md`).

## Convenciones

- **Pregunta**: lo que el spike valida. Si no se puede formular como sí/no, el spike está mal acotado.
- **Bloqueo upstream**: lo que tiene que estar definido antes de empezar (schema, trait, otro spike).
- **Salida**: artefacto que deja (binario que corre, doc con hallazgos, no código en producción).
- **Estado**: `done` / `in-progress` / `blocked` / `pending`.

## Spikes

### #1 — openrazer read-only ✅ done

- **Pregunta**: ¿zbus + tokio sobre `org.razer` permite enumerar devices y leer metadata/batería de forma estable?
- **Bloqueo**: ninguno.
- **Salida**: `crates/openrazer-adapter` + `openrazer-probe`. Hallazgos en `MEMORY.md` (camelCase, battery=`razer.device.power`, errores como `MethodError`).
- **Resultado**: sí. Nari + Tartarus enumerados; battery handling probado vía path de "n/a".

### #2 — openrazer write atómico

- **Pregunta**: ¿podemos aplicar un bundle (RGB efecto + brightness + DPI/macro relevante) sobre un device y, si una llamada falla a la mitad, volver atrás de forma determinística?
- **Bloqueo**: Profile schema v0 (saber qué writes tiene que soportar el adapter).
- **Salida**: extensión del crate `openrazer-adapter` con métodos write + binario `openrazer-apply` que toma un mini-perfil JSON y lo aplica al Tartarus.
- **Por qué importa**: si el rollback no es viable (openrazer no expone "transacciones"), el profile engine tiene que cambiar de semántica — mejor saberlo antes de diseñarlo.

### #3 — input-remapper o keyd

- **Pregunta**: ¿podemos cargar/descargar un mapping de teclas para un device específico sin afectar a otros y sin requerir reload global?
- **Bloqueo**: schema (sección de macros/remap), y elegir entre input-remapper (DBus) y keyd (config file + signal).
- **Salida**: spike-doc comparando ambos + adapter mínimo del ganador.
- **Riesgo**: ambos pueden pisar al usuario que ya configuró cosas a mano. Discovery: ¿coexistimos o tomamos ownership?

### #4 — PipeWire/WirePlumber bundle audio (v0.2)

- **Pregunta**: ¿podemos setear `default sink` + EQ preset + sidetone como una operación atómica observable, y revertir al cambiar de perfil?
- **Bloqueo**: schema (sección audio); spike #2 resuelto (mismas preguntas de atomicidad).
- **Salida**: crate `pipewire-adapter` + binario `audio-apply`.
- **Diferido a v0.2** según roadmap.

### #5 — libratbag DPI (v0.3)

- **Pregunta**: ¿libratbag DBus permite leer/escribir DPI stages en mice Razer sin colisionar con openrazer?
- **Bloqueo**: schema (sección mouse). Probablemente HW prestado (no tenemos mouse Razer en dogfood).
- **Diferido**: solo si v0.3 lo retoma.

### #6 — Foreground watcher X11

- **Pregunta**: ¿podemos detectar cambios de `_NET_ACTIVE_WINDOW` con latencia <100ms y obtener WM_CLASS / título de forma confiable bajo KDE+X11?
- **Bloqueo**: ninguno (independiente de schema y adapters).
- **Salida**: crate `foreground-watcher` + binario `fg-watch` que loguea `[ts] class=foo title=bar`.
- **Por qué importa**: es **el** diferenciador del producto. Si la latencia es mala o el evento es inestable, todo el pitch de "perfiles per-app" se cae.

### #7 — Foreground watcher Wayland (v0.2)

- **Pregunta**: ¿KWin script + portal `org.freedesktop.portal.*` cubren foreground tracking sin requerir extensiones por compositor?
- **Bloqueo**: spike #6 resuelto (para tener una baseline X11 contra la cual comparar).
- **Diferido a v0.2**.

### #8 — Importador Polychromatic

- **Pregunta**: ¿el formato de perfiles Polychromatic mapea de forma lossless a nuestro Profile schema?
- **Bloqueo**: Profile schema v0 estable.
- **Salida**: parser + binario `import-poly` que toma un perfil de Polychromatic y emite uno nuestro.
- **Por qué importa**: criterio de "done" del v0.1 lo exige (no fragmentar comunidad).

## Dependencias entre spikes

```
Profile schema v0
  ├─→ #2 openrazer write
  ├─→ #3 remapper
  ├─→ #4 pipewire   (v0.2)
  ├─→ #5 libratbag  (v0.3)
  └─→ #8 importador polychromatic

#6 foreground watcher X11   (independiente)
  └─→ #7 wayland   (v0.2)
```

## Orden propuesto para v0.1

1. **Profile schema v0** (no es spike — es el contrato)
2. **Adapter traits** (idem)
3. Spike #6 foreground watcher X11 — **paralelizable** con lo de abajo, no depende de schema
4. Spike #2 openrazer write
5. Spike #3 remapper
6. Spike #8 importador Polychromatic
7. Wiring: profile engine + orchestrator (deja de ser spike, ya es producto)

## No-spikes (decisiones que NO ameritan spike)

- **Lenguaje GUI**: ya decidido Qt/QML por ADR (cuando arranque la GUI, será código directo, no spike).
- **Persistencia de perfiles** (TOML vs JSON vs SQLite): decidir en el schema, no spikear.
- **Empaquetado** (.deb / Flatpak): irrelevante hasta que haya algo que empaquetar.
