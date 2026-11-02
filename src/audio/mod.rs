use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub mod equalizer;

/// Motor de reproducción de audio basado en Rodio (salida) y Symphonia (decodificación/metadatos).
///
/// Symphonia se integra plenamente en M3 (ecualizador con acceso a PCM crudo).
/// En M2, Rodio maneja la decodificación y reproducción; Symphonia está listo como dependencia.
pub struct AudioEngine {
    _stream: OutputStream,
    sink: Sink,
}

impl AudioEngine {
    /// Inicializa el dispositivo de salida por defecto y un Sink de Rodio.
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("no se pudo crear el stream de salida: {}", e))?;
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("no se pudo crear el sink de audio: {}", e))?;
        Ok(Self { _stream, sink })
    }

    /// Carga y reproduce un archivo de audio.
    pub fn play<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = File::open(path.as_ref()).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| e.to_string())?;
        self.sink.append(source);
        self.sink.play();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn pause(&self) {
        self.sink.pause();
    }

    #[allow(dead_code)]
    pub fn resume(&self) {
        self.sink.play();
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.sink.stop();
    }

    #[allow(dead_code)]
    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new().expect("no se pudo inicializar el motor de audio")
    }
}

/// Envuelve una fuente de Rodio y aplica una cadena de filtros biquad (ecualizador).
pub struct EqSource<S: Source<Item = f32>> {
    inner: S,
    filters: Vec<equalizer::BiquadFilter>,
    sample_rate: f32,
}

impl<S: Source<Item = f32>> EqSource<S> {
    pub fn new(inner: S, sample_rate: f32) -> Self {
        Self {
            inner,
            filters: Vec::new(),
            sample_rate,
        }
    }

    pub fn add_filter(&mut self, filter: equalizer::BiquadFilter) {
        self.filters.push(filter);
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }
}

impl<S: Source<Item = f32>> Source for EqSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
}

impl<S: Source<Item = f32>> Iterator for EqSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        let mut out = sample;
        for f in &mut self.filters {
            out = f.process(out);
        }
        Some(out)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
