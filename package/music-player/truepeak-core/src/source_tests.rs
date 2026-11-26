//! Unit tests for the source contract and stream descriptor.

use super::*;
use crate::meter::{
    TruePeakMeter,
    true_peak_interleaved,
};

// A finite duration is "known"; zero, negative, NaN, and infinity are not.
#[test]
fn duration_known_rejects_degenerate_values() {
    let known = AudioSpec {
        rate: 48000,
        channels: 2,
        duration_secs: 123.4,
    };
    assert!(known.duration_known());

    let zero = AudioSpec {
        rate: 48000,
        channels: 2,
        duration_secs: 0.0,
    };
    assert!(!zero.duration_known());

    let negative = AudioSpec {
        rate: 48000,
        channels: 2,
        duration_secs: -5.0,
    };
    assert!(!negative.duration_known());

    let nan = AudioSpec {
        rate: 48000,
        channels: 2,
        duration_secs: f64::NAN,
    };
    assert!(!nan.duration_known());

    let infinite = AudioSpec {
        rate: 48000,
        channels: 2,
        duration_secs: f64::INFINITY,
    };
    assert!(!infinite.duration_known());
}

// A minimal in-memory source so the trait can be exercised behind a `Box<dyn ...>`.
struct FakeSource {
    samples: Vec<f32>,
    channels: u16,
    rate: u32,
    cursor: usize, // interleaved sample index of the next chunk
    chunk: usize,  // interleaved samples per next_chunk
}

impl TruePeakSource for FakeSource {
    fn spec(&self) -> AudioSpec {
        return AudioSpec {
            rate: self.rate,
            channels: self.channels,
            duration_secs: self.samples.len() as f64 / (self.rate as f64 * self.channels as f64),
        };
    }

    fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError> {
        if self.cursor >= self.samples.len() {
            return Ok(Vec::new());
        }
        let end = (self.cursor + self.chunk).min(self.samples.len());
        let block = self.samples[self.cursor..end].to_vec();
        self.cursor = end;
        return Ok(block);
    }

    fn seek_to_frame(
        &mut self,
        frame: u64,
    ) -> Result<(), TruePeakError> {
        // Frame -> interleaved sample index lands exactly on a frame boundary.
        let target = frame as usize * self.channels as usize;
        if target > self.samples.len() {
            return Err(TruePeakError::Seek {
                message: format!("frame {frame} past end"),
            });
        }
        self.cursor = target;
        return Ok(());
    }
}

// Driving the trait through a boxed object and feeding the meter reproduces the
// whole-buffer measurement (proves the trait is object-safe and usable).
#[test]
fn boxed_source_drives_the_meter() {
    let samples = [0.0_f32, 0.9, 0.9, 0.0, -0.5, 0.3, 0.8, -0.7];
    let mut source: Box<dyn TruePeakSource> = Box::new(FakeSource {
        samples: samples.to_vec(),
        channels: 2,
        rate: 48000,
        cursor: 0,
        chunk: 3, // deliberately mid-frame chunking
    });

    let mut meter = TruePeakMeter::new(source.spec().channels as usize);
    loop {
        let chunk = source
            .next_chunk()
            .expect("in-memory source should yield a chunk");
        if chunk.is_empty() {
            break;
        }
        meter.feed(&chunk);
    }

    let expected = true_peak_interleaved(&samples, 2);
    assert!((meter.peak() - expected).abs() < 1e-6);
}

// Seeking to an exact frame repositions the stream; past-the-end seeks error.
#[test]
fn seek_to_frame_repositions_and_guards_end() {
    let mut source = FakeSource {
        samples: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6], // 3 stereo frames
        channels: 2,
        rate: 48000,
        cursor: 0,
        chunk: 2,
    };

    source
        .seek_to_frame(1)
        .expect("in-memory source should seek to an existing frame");
    let chunk = source
        .next_chunk()
        .expect("in-memory source should yield the sought chunk");
    assert_eq!(chunk, vec![0.3, 0.4]);

    assert!(source.seek_to_frame(99).is_err());
}
