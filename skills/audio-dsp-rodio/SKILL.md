---
name: audio-dsp-rodio
description: "Trigger: rodio, symphonia, audio, dsp, biquad, ecualizador, decodificación, motor de audio. Patrones para inicialización del motor con Rodio, decodificación con Symphonia y filtros biquad para ecualizador paramétrico en tiempo real."
license: Apache-2.0
metadata:
  author: gentle-ai-orchestrator
  version: "1.0"
---

## Activation Contract

Activa esta skill cuando se implemente o toque el motor de audio del reproductor Rust: inicialización de Rodio, decodificación con Symphonia, filtros biquad, ecualizador paramétrico de bandas en tiempo real y gestión de dispositivos de salida.

## Hard Rules

- El motor de audio se inicializa con `rodio::default_output_device()` y `rodio::Sink` o `rodio::OutputStream`.
- La decodificación de archivos usa `symphonia` (no rodio decoders solos) para obtener PCM crudo cuando se necesita acceso a muestras.
- Los filtros biquad se implementan como estructuras propias con coeficientes calculados por tipo de filtro (lowpass, highpass, peaking, lowshelf, highshelf) y frecuencia, Q, gain.
- El ecualizador paramétrico aplica cascada de biquads por banda; cada banda es independiente y se puede activar/desactivar en runtime.
- Todo el procesamiento de audio corre en el hilo de salida de Rodio o en un hilo dedicado de DSP; nunca bloquear el hilo principal de la UI.
- Las frecuencias de corte, Q y gain se validan antes de aplicar (rango audibles, Q positivo, gain dB acotado).

## Decision Gates

| Necesidad | Acción |
|-----------|--------|
| Reproducir un archivo | `rodio::Decoder` + `rodio::Sink` sobre el dispositivo de salida por defecto. |
| Acceder a muestras crudas / DSP | Decodificar con `symphonia` a PCM, aplicar biquads, enviar a Rodio. |
| Ecualizador paramétrico | Una instancia de biquad por banda, coeficientes recalculados al cambiar frecuencia/Q/gain. |
| Cambio de dispositivo de salida | Volver a crear `OutputStream` y reasignar el `Sink`. |

## Execution Steps

1. Añadir `rodio` y `symphonia` a `Cargo.toml` con versiones compatibles.
2. Inicializar `rodio::OutputStream` y `rodio::Sink` al arrancar el motor.
3. Implementar el struct `BiquadFilter` con métodos `set_lowpass`, `set_highpass`, `set_peaking`, `set_lowshelf`, `set_highshelf`.
4. Implementar `Equalizer` que gestione un `Vec<BiquadFilter>` y procese muestras PCM en bloque.
5. Conectar el pipeline: Symphonia decode → biquad cascade → rodio sink.
6. Ejecutar `cargo check` y `cargo test` para el módulo de audio.

## Output Contract

- `Cargo.toml` con `rodio` y `symphonia`.
- Struct `BiquadFilter` y `Equalizer` implementados.
- Pipeline de audio funcionando sin bloquear la UI.
- Pruebas de los coeficientes biquad y procesamiento de bloques.

## References

- `AGENT.md` — reglas del orquestador y estilo de respuesta.
