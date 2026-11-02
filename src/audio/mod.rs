use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub mod equalizer;

use equalizer::{BiquadFilter, BiquadType};

/// Banda del ecualizador: tipo de filtro biquad y sus parámetros.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqBand {
    pub filter_type: BiquadType,
    pub frequency: f32,
    pub q: f32,
    pub gain_db: f32,
}

/// Motor de reproducción de audio basado en Rodio (salida) y Symphonia (decodificación).
///
/// Mantiene la configuración del ecualizador y la aplica a cada pista nueva
/// envolviendo el decodificador en `EqSource`.
pub struct AudioEngine {
    _stream: OutputStream,
    sink: Sink,
    eq_bands: Vec<EqBand>,
}

impl AudioEngine {
    /// Inicializa el dispositivo de salida por defecto y un Sink de Rodio.
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("no se pudo crear el stream de salida: {}", e))?;
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("no se pudo crear el sink de audio: {}", e))?;
        Ok(Self {
            _stream,
            sink,
            eq_bands: Vec::new(),
        })
    }

    /// Carga y reproduce un archivo aplicando las bandas del ecualizador activas.
    pub fn play<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = File::open(path.as_ref()).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| e.to_string())?;
        let sample_rate = decoder.sample_rate() as f32;

        let mut eq = EqSource::new(decoder.convert_samples::<f32>(), sample_rate);
        for band in &self.eq_bands {
            eq.add_band(band.filter_type, band.frequency, band.q, band.gain_db);
        }

        self.sink.append(eq);
        self.sink.play();
        Ok(())
    }

    /// Activa o actualiza una banda. Reemplaza la existente del mismo tipo de filtro.
    pub fn set_eq_band(&mut self, band: EqBand) {
        upsert_band(&mut self.eq_bands, band);
    }

    /// Desactiva todas las bandas del ecualizador.
    #[allow(dead_code)]
    pub fn clear_eq(&mut self) {
        self.eq_bands.clear();
    }

    /// Devuelve las bandas activas.
    #[allow(dead_code)]
    pub fn eq_bands(&self) -> &[EqBand] {
        &self.eq_bands
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

/// Inserta una banda o reemplaza la existente del mismo tipo de filtro.
pub fn upsert_band(bands: &mut Vec<EqBand>, band: EqBand) {
    match bands.iter_mut().find(|b| b.filter_type == band.filter_type) {
        Some(existing) => *existing = band,
        None => bands.push(band),
    }
}

/// Envuelve una fuente de Rodio y aplica una cadena de filtros biquad (ecualizador).
pub struct EqSource<S: Source<Item = f32>> {
    inner: S,
    filters: Vec<BiquadFilter>,
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

    /// Añade una banda usando el sample rate de la fuente.
    pub fn add_band(&mut self, filter_type: BiquadType, frequency: f32, q: f32, gain_db: f32) {
        let mut filter = BiquadFilter::new();
        filter.set_params(filter_type, self.sample_rate, frequency, q, gain_db);
        self.filters.push(filter);
    }

    pub fn add_filter(&mut self, filter: BiquadFilter) {
        self.filters.push(filter);
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct TestSource {
        samples: std::vec::IntoIter<f32>,
        rate: u32,
    }

    impl TestSource {
        fn new(samples: Vec<f32>, rate: u32) -> Self {
            Self {
                samples: samples.into_iter(),
                rate,
            }
        }

        fn constant(value: f32, len: usize, rate: u32) -> Self {
            Self::new(vec![value; len], rate)
        }

        fn alternating(len: usize, rate: u32) -> Self {
            let samples = (0..len)
                .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
                .collect();
            Self::new(samples, rate)
        }
    }

    impl Iterator for TestSource {
        type Item = f32;

        fn next(&mut self) -> Option<f32> {
            self.samples.next()
        }
    }

    impl Source for TestSource {
        fn current_frame_len(&self) -> Option<usize> {
            None
        }

        fn channels(&self) -> u16 {
            1
        }

        fn sample_rate(&self) -> u32 {
            self.rate
        }

        fn total_duration(&self) -> Option<Duration> {
            None
        }
    }

    #[test]
    fn eq_source_without_filters_passes_samples_through() {
        let source = TestSource::constant(0.5, 8, 44_100);
        let eq = EqSource::new(source, 44_100.0);
        let out: Vec<f32> = eq.collect();
        assert_eq!(out, vec![0.5; 8]);
    }

    #[test]
    fn eq_source_zero_gain_peaking_is_identity() {
        let source = TestSource::constant(0.25, 32, 44_100);
        let mut eq = EqSource::new(source, 44_100.0);
        eq.add_band(BiquadType::Peaking, 1_000.0, 1.0, 0.0);
        let out: Vec<f32> = eq.collect();
        for sample in out {
            assert!((sample - 0.25).abs() < 1e-4, "salida inesperada: {sample}");
        }
    }

    #[test]
    fn eq_source_lowpass_attenuates_high_frequencies() {
        // Alternancia +1/-1 a 44.1 kHz equivale a la frecuencia de Nyquist.
        let source = TestSource::alternating(512, 44_100);
        let mut eq = EqSource::new(source, 44_100.0);
        eq.add_band(BiquadType::Lowpass, 200.0, 0.707, 0.0);
        let out: Vec<f32> = eq.collect();
        let tail = &out[256..];
        let peak = tail.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
        assert!(peak < 0.5, "el lowpass no atenuó la señal: pico {peak}");
    }

    #[test]
    fn upsert_band_replaces_same_filter_type() {
        let mut bands: Vec<EqBand> = Vec::new();
        upsert_band(
            &mut bands,
            EqBand {
                filter_type: BiquadType::Peaking,
                frequency: 1_000.0,
                q: 1.0,
                gain_db: 0.0,
            },
        );
        upsert_band(
            &mut bands,
            EqBand {
                filter_type: BiquadType::Peaking,
                frequency: 2_000.0,
                q: 1.0,
                gain_db: 6.0,
            },
        );
        assert_eq!(bands.len(), 1);
        assert_eq!(bands[0].frequency, 2_000.0);
    }

    #[test]
    fn upsert_band_keeps_distinct_filter_types() {
        let mut bands: Vec<EqBand> = Vec::new();
        upsert_band(
            &mut bands,
            EqBand {
                filter_type: BiquadType::Lowpass,
                frequency: 200.0,
                q: 0.707,
                gain_db: 0.0,
            },
        );
        upsert_band(
            &mut bands,
            EqBand {
                filter_type: BiquadType::Peaking,
                frequency: 1_000.0,
                q: 1.0,
                gain_db: 3.0,
            },
        );
        assert_eq!(bands.len(), 2);
    }

    fn write_test_wav(path: &std::path::Path, sample_rate: u32, samples: &[i16]) {
        use std::io::Write;
        let mut file = std::fs::File::create(path).unwrap();
        let data_len = (samples.len() * 2) as u32;
        let byte_rate = sample_rate * 2;

        file.write_all(b"RIFF").unwrap();
        file.write_all(&(36 + data_len).to_le_bytes()).unwrap();
        file.write_all(b"WAVE").unwrap();
        file.write_all(b"fmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap();
        file.write_all(&1u16.to_le_bytes()).unwrap(); // PCM
        file.write_all(&1u16.to_le_bytes()).unwrap(); // mono
        file.write_all(&sample_rate.to_le_bytes()).unwrap();
        file.write_all(&byte_rate.to_le_bytes()).unwrap();
        file.write_all(&2u16.to_le_bytes()).unwrap(); // block align
        file.write_all(&16u16.to_le_bytes()).unwrap(); // bits por muestra
        file.write_all(b"data").unwrap();
        file.write_all(&data_len.to_le_bytes()).unwrap();
        for sample in samples {
            file.write_all(&sample.to_le_bytes()).unwrap();
        }
    }

    #[test]
    fn eq_source_filters_decoded_wav() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rad_eq_{nanos}.wav"));

        // Alternancia +16000/-16000 a 44.1 kHz: contenido en la frecuencia de Nyquist.
        let samples: Vec<i16> = (0..2048)
            .map(|i| if i % 2 == 0 { 16_000 } else { -16_000 })
            .collect();
        write_test_wav(&path, 44_100, &samples);

        let decoder = Decoder::new(BufReader::new(File::open(&path).unwrap())).unwrap();
        let rate = decoder.sample_rate() as f32;
        let mut eq = EqSource::new(decoder.convert_samples::<f32>(), rate);
        eq.add_band(BiquadType::Lowpass, 200.0, 0.707, 0.0);
        let out: Vec<f32> = eq.collect();

        let _ = std::fs::remove_file(&path);

        assert!(out.len() >= 1024, "muestras decodificadas: {}", out.len());
        let tail = &out[out.len() / 2..];
        let peak = tail.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
        assert!(peak < 0.2, "el lowpass no atenuó el WAV: pico {peak}");
    }
}
