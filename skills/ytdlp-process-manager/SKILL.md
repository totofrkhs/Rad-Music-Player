---
name: ytdlp-process-manager
description: "Trigger: yt-dlp, tokio process, descarga, cola, carátula, id3, etiquetas. Instrucciones para ejecutar yt-dlp de forma asíncrona con tokio::process, gestionar colas de descarga simultáneas, extraer carátulas y leer etiquetas con id3."
license: Apache-2.0
metadata:
  author: gentle-ai-orchestrator
  version: "1.0"
---

## Activation Contract

Activa esta skill cuando se implemente descarga de audio mediante yt-dlp en el reproductor Rust: ejecución asíncrona con `tokio::process::Command`, cola de descargas concurrentes, extracción de carátulas (thumbnails) y lectura/escritura de etiquetas ID3.

## Hard Rules

- yt-dlp se invoca como proceso externo con `tokio::process::Command`, nunca como biblioteca embebida.
- Cada descarga es un `tokio::task` propio; se respeta un límite de concurrencia configurable (semáforo o canal acotado) para no saturar red/CPU.
- La cola de descargas se implementa con `tokio::sync::mpsc` o similar; los jobs encolados tienen estado: pending, running, completed, failed.
- Las carátulas se extraen con la opción `--write-thumbnail` o `--embed-thumbnail` de yt-dlp y se guardan junto al archivo de audio.
- Las etiquetas ID3 se leen/escriben con el crate `id3` (no se parsea el archivo manualmente).
- Se capturan stdout/stderr de yt-dlp para parsear progreso y errores; se respetan los códigos de salida.
- Las rutas de salida se sanitizan para evitar path traversal.

## Decision Gates

| Necesidad | Acción |
|-----------|--------|
| Descargar una canción | Encolar job con URL y destino; `tokio::process::Command` con `yt-dlp`. |
| Muchas descargas a la vez | Límite de concurrencia con semáforo; canal de jobs. |
| Obtener carátula | Pasar `--write-thumbnail` a yt-dlp; mover el thumbnail generado. |
| Leer etiquidas ID3 | Usar `id3::Tag::read_from_path` sobre el archivo descargado. |
| Escribir etiquidas ID3 | Usar `id3::Tag` y `write_to_path` tras la descarga. |

## Execution Steps

1. Añadir `tokio`, `id3` y cualquier crate de utilidad a `Cargo.toml`.
2. Implementar `DownloadJob` con URL, destino, estado y metadatos.
3. Implementar `DownloadQueue` con canal `mpsc` y workers limitados por semáforo.
4. Implementar `run_ytdlp` que lance `tokio::process::Command` y capture salida.
5. Tras descarga, extraer thumbnail y escribir etiquidas ID3 con `id3`.
6. Exponer eventos de progreso/estado a la UI vía canales o callbacks.
7. Ejecutar `cargo check` y pruebas de la cola con URLs falsas/mockeadas.

## Output Contract

- `DownloadQueue` con límite de concurrencia.
- Ejecución asíncrona de yt-dlp con captura de salida.
- Extracción de carátulas y escritura/lectura de ID3.
- Manejo de errores y estados de job.

## References

- `AGENT.md` — reglas del orquestador y estilo de respuesta.
