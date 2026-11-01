//! Reads one file's length from its container header, without decoding any audio.
//!
//! What: `probe_duration_secs` opens a file, probes its format, and divides the frame count by the sample rate.
//! Why: the library rows show each track's length before the slow true-peak measurement has run, and the
//! header holds it for most formats. Formats whose header omits the frame count give `None`.

use std::fs::File;
use std::path::Path;

use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

/// What:
///      `pub fn probe_duration_secs(path: &Path) -> Option<f64>`.
///  The length in seconds from the container header, or `None` when it cannot be read.
/// Why:
///       A header read is cheap enough to run for every track at queue load, while a full decode is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function probeDurationSecs(path: string): number | null {
///   const track = probeFormat(path)?.firstAudioTrack();
///   return track?.numFrames && track.sampleRate ? track.numFrames / track.sampleRate : null;
/// }
/// ```
pub fn probe_duration_secs(path: &Path) -> Option<f64> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            tracing::debug!(path = %path.display(), error = %error, "duration probe could not open the file");
            return None;
        }
    };
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|value| return value.to_str()) {
        hint.with_extension(extension);
    }
    let format = match symphonia::default::get_probe().probe(
        &hint,
        stream,
        FormatOptions::default(),
        MetadataOptions::default(),
    ) {
        Ok(format) => format,
        Err(error) => {
            tracing::debug!(path = %path.display(), error = %error, "duration probe could not read the format");
            return None;
        }
    };
    return duration_of_first_audio_track(&*format);
}

/// What:
///      `fn duration_of_first_audio_track(format: &dyn FormatReader) -> Option<f64>`.
///  The first audio track's frame count divided by its sample rate.
/// Why:
///       Each container lists its tracks, and the length belongs to the audio track alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const durationOf = (format: FormatReader) => { const t = format.firstAudioTrack(); return t && t.numFrames && t.rate ? t.numFrames / t.rate : null; };
/// ```
fn duration_of_first_audio_track(format: &dyn FormatReader) -> Option<f64> {
    let track = format.first_track_known_codec(TrackType::Audio)?;
    let rate = track
        .codec_params
        .as_ref()
        .and_then(|params| return params.audio())
        .and_then(|audio| return audio.sample_rate)?;
    let frames = track.num_frames?;
    return frames_to_secs(frames, rate);
}

/// What:
///      `fn frames_to_secs(frames: u64, rate: u32) -> Option<f64>`.
///  Converts a frame count at a sample rate into seconds, or `None` for a zero rate.
/// Why:
///       A zero rate would divide by zero, and a header with no rate has no usable length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const framesToSecs = (frames: number, rate: number) => (rate > 0 ? frames / rate : null);
/// ```
fn frames_to_secs(frames: u64, rate: u32) -> Option<f64> {
    if rate == 0 {
        return None;
    }
    return Some(frames as f64 / f64::from(rate));
}

#[cfg(test)]
#[path = "duration_probe_tests.rs"]
mod tests;
