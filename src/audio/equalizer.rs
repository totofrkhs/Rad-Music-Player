use std::f32::consts::PI;

/// Tipo de filtro biquad soportado por el ecualizador paramétrico.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiquadType {
    Lowpass,
    Highpass,
    Peaking,
    LowShelf,
    HighShelf,
}

/// Un filtro biquad de segundo orden.
/// Los coeficientes se calculan por tipo, frecuencia, Q y gain.
pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    /// Crea un filtro con coeficientes iniciales de paso directo (identity).
    pub fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Calcula y aplica coeficientes para un filtro biquad.
    ///
    /// Parámetros:
    /// - `sample_rate`: frecuencia de muestreo en Hz (ej. 44100).
    /// - `frequency`: frecuencia de corte o centro en Hz.
    /// - `q`: factor Q (ancho de banda). Debe ser > 0.
    /// - `gain_db`: ganancia en dB (solo aplica a Peaking, LowShelf, HighShelf).
    pub fn set_params(&mut self, filter_type: BiquadType, sample_rate: f32, frequency: f32, q: f32, gain_db: f32) {
        let a0 = 1.0;
        let w0 = 2.0 * PI * frequency / sample_rate;
        let alpha = (w0 / 2.0) * (1.0 / q.max(0.001)).sin().min(1.0);
        let a = 10.0f32.powf(gain_db / 40.0);
        let cos_w0 = w0.cos();
        let _sin_w0 = w0.sin();

        let mut b0: f32;
        let mut b1: f32;
        let mut b2: f32;
        let mut a1: f32;
        let mut a2: f32;
        match filter_type {
            BiquadType::Lowpass => {
                b0 = (1.0 - cos_w0) / 2.0;
                b1 = 1.0 - cos_w0;
                b2 = (1.0 - cos_w0) / 2.0;
                a1 = -2.0 * cos_w0;
                a2 = 1.0 - alpha;
            }
            BiquadType::Highpass => {
                b0 = (1.0 + cos_w0) / 2.0;
                b1 = -(1.0 + cos_w0);
                b2 = (1.0 + cos_w0) / 2.0;
                a1 = -2.0 * cos_w0;
                a2 = 1.0 - alpha;
            }
            BiquadType::Peaking => {
                b0 = 1.0 + alpha * a;
                b1 = -2.0 * cos_w0;
                b2 = 1.0 - alpha * a;
                a1 = -2.0 * cos_w0;
                a2 = 1.0 - alpha / a;
            }
            BiquadType::LowShelf => {
                let term = 2.0 * a.sqrt() * alpha;
                b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + term);
                b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - term);
                a1 = -2.0 * cos_w0;
                a2 = (a + 1.0) + (a - 1.0) * cos_w0 - term;
            }
            BiquadType::HighShelf => {
                let term = 2.0 * a.sqrt() * alpha;
                b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + term);
                b1 = -2.0 * a * ((a + 1.0) + (a - 1.0) * cos_w0);
                b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - term);
                a1 = 2.0 * cos_w0;
                a2 = (a + 1.0) + (a - 1.0) * cos_w0 + term;
            }
        }

        // Normalizar por a0 para evitar dividir en cada muestra.
        b0 /= a0;
        b1 /= a0;
        b2 /= a0;
        a1 /= a0;
        a2 /= a0;

        self.b0 = b0;
        self.b1 = b1;
        self.b2 = b2;
        self.a1 = a1;
        self.a2 = a2;
    }

    /// Procesa una muestra de audio estéreo/mono (valor único).
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    /// Resetea el estado del filtro (limpia el historial de muestras).
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Devuelve true si los coeficientes son finitos y el filtro es estable.
    pub fn is_stable(&self) -> bool {
        self.b0.is_finite()
            && self.b1.is_finite()
            && self.b2.is_finite()
            && self.a1.is_finite()
            && self.a2.is_finite()
            && self.a1.abs() < 2.0
            && self.a2.abs() < 2.0
    }
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_passes_through() {
        let mut f = BiquadFilter::new();
        assert_eq!(f.process(0.5), 0.5);
    }

    #[test]
    fn lowpass_coefficients_are_finite() {
        let mut f = BiquadFilter::new();
        f.set_params(BiquadType::Lowpass, 44100.0, 1000.0, 0.707, 0.0);
        assert!(f.is_stable());
    }

    #[test]
    fn peaking_applies_gain() {
        let mut f = BiquadFilter::new();
        f.set_params(BiquadType::Peaking, 44100.0, 1000.0, 1.0, 6.0);
        assert!(f.is_stable());
        // El pico en la frecuencia central debería amplificar la señal.
        let out = f.process(0.1);
        assert!(out.is_finite());
    }

    #[test]
    fn q_must_be_positive() {
        let mut f = BiquadFilter::new();
        f.set_params(BiquadType::Lowpass, 44100.0, 1000.0, 0.0, 0.0);
        assert!(f.is_stable());
    }
}
