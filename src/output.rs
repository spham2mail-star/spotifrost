//! The last stage before the real sink: applies the volume when this stage
//! owns it, and keeps the result under full scale with the final limiter.

use std::sync::Arc;

use librespot_playback::SAMPLE_RATE;
use librespot_playback::audio_backend::{Sink, SinkResult};
use librespot_playback::convert::Converter;
use librespot_playback::decoder::AudioPacket;
use librespot_playback::mixer::VolumeGetter;

/// Applies volume (when asked) and the limiter before the real sink.
pub struct OutputStage {
    inner: Box<dyn Sink>,
    control: Arc<crate::sink::AudioControl>,
    /// Player volume, used to calculate the limiter ceiling.
    volume: Box<dyn VolumeGetter + Send>,
    /// Whether this wrapper applies volume. Otherwise the inner sink applies
    /// it, to already queued audio.
    applies_volume: bool,
    /// Final limiter, placed here because this stage knows the output volume.
    limiter: crate::limiter::Limiter,
}

impl OutputStage {
    pub fn new(
        inner: Box<dyn Sink>,
        control: Arc<crate::sink::AudioControl>,
        volume: Box<dyn VolumeGetter + Send>,
        applies_volume: bool,
    ) -> Self {
        Self {
            inner,
            control,
            volume,
            applies_volume,
            limiter: crate::limiter::Limiter::new(f64::from(SAMPLE_RATE)),
        }
    }
}

/// Full-scale level for samples leaving the stage.
///
/// This is 1.0 after volume is applied. Before volume, it is the level that
/// becomes 1.0 after the inner sink applies volume.
fn full_scale(volume: f64, applied: bool) -> Option<f64> {
    if applied {
        Some(1.0)
    } else if volume > f64::EPSILON {
        Some(1.0 / volume)
    } else {
        // At zero volume, no finite pre-volume ceiling is needed.
        None
    }
}

impl Sink for OutputStage {
    fn start(&mut self) -> SinkResult<()> {
        self.inner.start()
    }

    fn stop(&mut self) -> SinkResult<()> {
        self.inner.stop()
    }

    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        if self.control.take_processing_reset() {
            self.limiter = crate::limiter::Limiter::new(f64::from(SAMPLE_RATE));
        }
        let packet = match packet {
            AudioPacket::Samples(mut samples) => {
                let attenuation = self.volume.attenuation_factor();
                if self.applies_volume {
                    for sample in &mut samples {
                        *sample *= attenuation;
                    }
                }
                if let Some(full_scale) = full_scale(attenuation, self.applies_volume) {
                    self.limiter.process(&mut samples, full_scale);
                }
                AudioPacket::Samples(samples)
            }
            raw => raw,
        };
        self.inner.write(packet, converter)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn full_scale_follows_who_applies_volume() {
        assert_eq!(super::full_scale(0.5, true), Some(1.0));
        assert_eq!(super::full_scale(0.5, false), Some(2.0));
        assert_eq!(super::full_scale(0.0, false), None);
    }
}
