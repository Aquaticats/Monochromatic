//! Corpus collector:
//!  decode every track through the shared truepeak-core meter and emit
//! fine-grained cache bins,
//!  so the search can model dense sub-second probe windows.
//!
//! Decoding is delegated to ffmpeg (one `f32le` pipe per file at the stream's native rate
//! and channels),
//!  so no decoder is ported into the repo;
//!  the meter is the production
//! `truepeak_core::TruePeakMeter`,
//!  segmented with `take_peak` at each bin boundary.
//!  Usage:
//! `truepeak-core-collect <music-root> <out.jsonl> [bin_seconds] [workers]`.

/// Imports the work queue shared across decode threads.
use std::collections::VecDeque;
/// Imports process arguments.
use std::env;
/// Imports the output file and writer.
use std::fs::File;
/// Imports buffered writing and the stdout reader trait.
use std::io::{BufWriter, Read, Write};
/// Imports the borrowed path type and owned path buffer.
use std::path::{Path, PathBuf};
/// Imports the child-process spawner for ffprobe and ffmpeg.
use std::process::{Command, Stdio};
/// Imports the shared queue handle,
///  channel,
///  and lock.
use std::sync::{Arc, Mutex, mpsc};
/// Imports thread spawning for the decode workers.
use std::thread;

/// Imports `anyhow` helpers for application-level error returns.
use anyhow::{anyhow, bail, Context, Result};

/// Imports serde derive and JSON for the per-track output rows.
use serde::Serialize;
/// Imports the shared meter,
///  the one true-peak measurement.
use truepeak_core::TruePeakMeter;

/// Default seconds per cache bin when the argument is omitted.
const DEFAULT_BIN_SECONDS: f64 = 0.1;
/// Default decode-worker count when the argument is omitted.
const DEFAULT_WORKERS: usize = 6;
/// Audio extensions the collector decodes.
const AUDIO_EXTENSIONS: &[&str] = &[
    "flac", "wav", "wave", "mp3", "ogg", "oga", "opus", "m4a", "m4b", "mp4", "aac", "aiff", "aif",
];

/// One track's measurement:
///  shape,
///  the full true peak,
///  and the per-bin peaks.
#[derive(Serialize)]
struct TrackMetrics {
    /// Absolute path of the decoded file.
    path: String,
    /// Decoded length in seconds (frames over rate).
    duration_secs: f64,
    /// Sample rate in frames per second.
    rate: u32,
    /// Channel count.
    channels: u16,
    /// Decoded interleaved frame count.
    decoded_frames: u64,
    /// Seconds covered by each bin.
    bin_seconds: f64,
    /// Full-track true peak (linear).
    full_peak: f32,
    /// Per-bin true peaks (linear) from the shared meter.
    bin_peaks: Vec<f32>,
}

/// Read the sample rate,
///  channel count,
///  and duration of a file via ffprobe.
fn probe_spec(path: &Path) -> Result<(u32, u16, f64)> {
    // Ask ffprobe for the first audio stream's rate and channels and the container duration.
    let output = Command::new("ffprobe")
        .args([
            "-v", "error", "-select_streams", "a:0", "-show_entries",
            "stream=sample_rate,channels:format=duration", "-of", "json",
        ])
        .arg(path)
        .output()?;
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let stream = &parsed["streams"][0];
    // Pull each field, defaulting duration to zero (the decoded frame count is authoritative).
    let rate: u32 = stream["sample_rate"].as_str().and_then(|s| return s.parse().ok()).unwrap_or(0);
    let channels: u16 = stream["channels"].as_u64().unwrap_or(0) as u16;
    let duration: f64 = parsed["format"]["duration"].as_str().and_then(|s| return s.parse().ok()).unwrap_or(0.0);
    return Ok((rate, channels, duration))
}

/// Decode a file with ffmpeg and return its full peak,
///  per-bin peaks,
///  and frame count.
fn decode_and_bin(
    path: &Path,
    channels: u16,
    bin_frames: u64,
) -> Result<(f32, Vec<f32>, u64)> {
    // Spawn ffmpeg to emit native-rate interleaved f32 little-endian on stdout.
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-i"]).arg(path)
        .args(["-f", "f32le", "-"])
        .stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;
    let mut stdout = child.stdout.take().context("ffmpeg stdout missing")?;

    let samples_per_bin = (bin_frames * u64::from(channels)) as usize;
    let mut meter = TruePeakMeter::new(usize::from(channels));
    let mut bins: Vec<f32> = Vec::new();
    let mut leftover: Vec<u8> = Vec::new();
    let mut into_bin = 0usize;
    let mut total_samples: u64 = 0;
    let mut buf = [0u8; 65536];
    loop {
        let read = stdout.read(&mut buf)?;
        if read == 0 {
            break;
        }
        // Convert all complete 4-byte groups to f32, keeping any partial bytes for next read.
        leftover.extend_from_slice(&buf[..read]);
        let complete = leftover.len() / 4 * 4;
        let samples: Vec<f32> = leftover[..complete]
            .as_chunks::<4>().0.iter()
            .map(|b| return f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        leftover.drain(..complete);
        total_samples += samples.len() as u64;
        // Feed in bin-aligned chunks; take the meter's peak at each bin boundary.
        let mut slice = &samples[..];
        while !slice.is_empty() {
            let take = (samples_per_bin - into_bin).min(slice.len());
            meter.feed(&slice[..take]);
            into_bin += take;
            slice = &slice[take..];
            if into_bin == samples_per_bin {
                bins.push(meter.take_peak());
                into_bin = 0;
            }
        }
    }
    // Flush the final partial bin and the full peak is the max across bins.
    if into_bin > 0 {
        bins.push(meter.take_peak());
    }
    child.wait()?;
    let full_peak = bins.iter().fold(0.0_f32, |peak, &bin| return peak.max(bin));
    let decoded_frames = if channels == 0 { 0 } else { total_samples / u64::from(channels) };
    return Ok((full_peak, bins, decoded_frames))
}

/// Measure one track end to end (probe,
///  decode,
///  bin) into a `TrackMetrics`.
fn measure(path: &Path, bin_seconds: f64) -> Result<TrackMetrics> {
    let (rate, channels, _container_duration) = probe_spec(path)?;
    if rate == 0 || channels == 0 {
        bail!("missing rate or channels");
    }
    let bin_frames = ((bin_seconds * f64::from(rate)) as u64).max(1);
    let (full_peak, bin_peaks, decoded_frames) = decode_and_bin(path, channels, bin_frames)?;
    return Ok(TrackMetrics {
        path: path.to_string_lossy().into_owned(),
        duration_secs: decoded_frames as f64 / f64::from(rate),
        rate,
        channels,
        decoded_frames,
        bin_seconds,
        full_peak,
        bin_peaks,
    })
}

/// Recursively collect audio files under a root,
///  sorted for determinism.
fn collect_audio_files(root: &Path) -> Result<Vec<PathBuf>> {
    // Depth-first walk with an explicit stack (no recursion over the directory tree depth).
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        let metadata = std::fs::metadata(&path)?;
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path)? {
                pending.push(entry?.path());
            }
        } else if is_audio(&path) {
            files.push(path);
        }
    }
    files.sort();
    return Ok(files)
}

/// Report whether a path has a decodable audio extension and is not an AppleDouble sidecar.
fn is_audio(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| return n.to_str()).unwrap_or("");
    if name.starts_with("._") {
        return false;
    }
    match path.extension().and_then(|e| return e.to_str()) {
        Some(ext) => return AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()),
        None => return false,
    }
}

/// Entry point:
///  walk the root,
///  decode in parallel,
///  and write the JSONL corpus.
fn main() -> Result<()> {
    // Send tracing events (including truepeak-core's) to stderr; the JSONL report is stdout.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| return tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    let args: Vec<String> = env::args().collect();
    let root = PathBuf::from(args.get(1).context("usage: collect <root> <out.jsonl> [bin_seconds] [workers]")?);
    let out_path = PathBuf::from(args.get(2).context("usage: collect <root> <out.jsonl> [bin_seconds] [workers]")?);
    let bin_seconds = args.get(3).and_then(|v| return v.parse().ok()).unwrap_or(DEFAULT_BIN_SECONDS);
    let workers = args.get(4).and_then(|v| return v.parse().ok()).unwrap_or(DEFAULT_WORKERS);

    let files = collect_audio_files(&root)?;
    let total = files.len();
    let queue = Arc::new(Mutex::new(VecDeque::from(files)));
    let (sender, receiver) = mpsc::channel::<Result<TrackMetrics>>();
    // Each worker pops a path, measures it, and sends the result or an error string.
    for _ in 0..workers {
        // Per-worker clones get their own names so neither shadows the shared
        // originals the loop keeps cloning from.
        let worker_queue = Arc::clone(&queue);
        let worker_sender = sender.clone();
        thread::spawn(move || {
            loop {
                let popped = worker_queue.lock().expect("queue poisoned").pop_front();
                let Some(path) = popped else { break };
                let message = measure(&path, bin_seconds)
                    .map_err(|error| anyhow!("{}: {error}", path.to_string_lossy()));
                if worker_sender.send(message).is_err() {
                    break;
                }
            }
        });
    }
    drop(sender);

    // Drain results, writing each measured track and counting failures.
    let mut writer = BufWriter::new(File::create(&out_path)?);
    let mut measured = 0usize;
    let mut failed = 0usize;
    for message in receiver {
        match message {
            Ok(metrics) => {
                serde_json::to_writer(&mut writer, &metrics)?;
                writer.write_all(b"\n")?;
                measured += 1;
            }
            Err(error) => {
                tracing::warn!(cause = %error, "decode failed");
                failed += 1;
            }
        }
    }
    writer.flush()?;
    println!("collected {measured}/{total} tracks ({failed} failed) bin_seconds={bin_seconds} -> {}", out_path.to_string_lossy());
    return Ok(())
}
