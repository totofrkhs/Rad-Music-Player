---
name: engram-state-sync
description: "Trigger: engram, memoria, hitos, arquitectura, contexto persistente, trazabilidad. Protocolo para registrar hitos de código, decisiones de arquitectura y contexto persistente en Engram entre sesiones."
license: Apache-2.0
metadata:
  author: gentle-ai-orchestrator
  version: "1.0"
---

## Activation Contract

Activa esta skill cuando se necesite persistir contexto, hitos de código o decisiones de arquitectura en la memoria Engram del proyecto `rad#`, para mantener trazabilidad entre sesiones de trabajo del reproductor de música.

## Hard Rules

- Cada hito o decisión se registra con `mem_save` inmediatamente después de ocurrir, no al final de la sesión.
- El `topic_key` es estable y evoluciona con upsert; distintos temas no se sobrescriben entre sí.
- El contenido de `mem_save` sigue la estructura: What / Why / Where / Learned.
- Antes de buscar memoria previa, llamar `mem_context`; si no hay resultado, `mem_search` con palabras clave; si hay coincidencia, `mem_get_observation` para contenido completo.
- Al cerrar sesión, llamar `mem_session_summary` con Goal / Instructions / Discoveries / Accomplished / Next Steps / Relevant Files.
- Nunca se confía en memoria marcada `needs_review` sin verificar contra evidencia actual.

## Decision Gates

| Necesidad | Acción |
|-----------|--------|
| Guardar decisión de arquitectura | `mem_save` con type `architecture`, topic_key estable. |
| Guardar bug fix | `mem_save` con type `bugfix`, incluyendo root cause. |
| Recordar trabajo previo | `mem_context` → `mem_search` → `mem_get_observation`. |
| Cerrar sesión | `mem_session_summary` completo. |
| Proyecto ambiguo | No adivinar; pedir al usuario elegir uno de `available_projects`. |

## Execution Steps

1. Al iniciar sesión, resolver proyecto con `mem_current_project`.
2. Durante el trabajo, tras cada decisión o fix, `mem_save` con título verb+qué y contenido estructurado.
3. Al retomar trabajo previo, buscar con `mem_context` y `mem_search`.
4. Al finalizar, `mem_session_summary` con los seis campos obligatorios.
5. Si Engram falla, preservar progreso local y marcar el mirror como pendiente; no bloquear trabajo seguro.

## Output Contract

- Observaciones en Engram asociadas al proyecto `rad#`.
- Estructura What/Why/Where/Learned en cada save.
- `mem_session_summary` al cierre de sesión.
- Trazabilidad de hitos y decisiones entre sesiones.

## References

- `AGENT.md` — reglas del orquestador y estilo de respuesta.
- Protocolo Engram del sistema (persistente, sobreviviente a compaction).
