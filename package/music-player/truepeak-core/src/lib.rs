//! Shared true-peak core for the desktop and Android music players.
//!
//! "True peak" (a.k.a.
//!  inter-sample peak) is the highest level the analog waveform
//! reaches AFTER a DAC reconstructs it between the stored samples;
//!  it can sit above
//! the largest stored sample.
//!  This crate estimates it by oversampling each channel
//! ~4x with a cubic (Catmull-Rom) interpolation and taking the largest magnitude
//! seen,
//!  then turns that estimate into one constant per-track gain that brings the
//! track down to a -1 dBTP ceiling (never up),
//!  so playback cannot overflow the
//! converter.
//!
//! Both flavors used to carry their own copy of this logic,
//!  with the Android copy
//! adding an older windowing policy and a `1.26` safety factor that this crate does
//! NOT ship.
//!  The crate owns the meter,
//!  the gain math,
//!  the decoded-audio source
//! contract,
//!  the window-placement math,
//!  and the versioned policy identity that keys
//! the persistent decision cache.
//!  Platform crates supply decoded audio through
//! [`TruePeakSource`] and drive the policy;
//!  the measurement and gain rules live here.
//!
//! The probe resolver,
//!  the track fingerprint (feature `fingerprint`),
//!  and the Turso decision cache (feature `service`)
//! sit on top of that foundation.
//!  There is no classifier:
//!  the shipped policy is classifier-free.

/// The shared Catmull-Rom true-peak meter and its whole-buffer helper.
mod meter;

/// The decoded-audio source contract and the `AudioSpec` describing a stream.
mod source;

/// The single error type the fallible source methods return.
mod error;

/// Attenuate-only normalization gain and the dB helpers around it.
mod gain;

/// Even window placement across a long track,
///  in interleaved frames.
mod window;

/// The versioned policy and the identity tuple that keys cache rows.
mod policy;

/// The per-provenance probe buckets and the track provenance signals.
mod bucketpolicy;

/// The frontier-zoom probe measuring bins beside the loudest heard so far.
mod probe;

/// FLAC frame-size profiling without decoding:
///  the lossless bones channel.
mod bones;

/// Probe inputs (provenance and bones) derived from the file itself.
mod inputs;

/// The gain decision types the resolver produces.
mod decision;

/// The resolver that drives a source through the policy to a gain decision.
mod resolve;

/// The track fingerprint (the opaque cache key),
///  behind the `fingerprint` feature.
#[cfg(feature = "fingerprint")]
mod fingerprint;

/// The persistent decision cache,
///  backed by Turso,
///  behind the `service` feature.
#[cfg(feature = "service")]
mod cache;

/// The cache-aware resolve composing the cache and resolver,
///  behind the `service` feature.
#[cfg(feature = "service")]
mod service;

/// Re-exports the meter type and its whole-buffer convenience function.
pub use crate::meter::{TruePeakMeter, true_peak_interleaved};

/// Re-exports the decoded-audio source contract and its stream descriptor.
pub use crate::source::{AudioSpec, TruePeakSource};

/// Re-exports the fallible-source error type.
pub use crate::error::TruePeakError;

/// Re-exports the ceiling constant and the gain and dB functions.
pub use crate::gain::{CEILING, normalization_gain, peak_dbtp, probe_estimated_peak};

/// Re-exports the window-placement helpers.
pub use crate::window::{WindowPlacement, window_frame_starts};

/// Re-exports the policy type,
///  its identity tuple,
///  the shipped default policy,
///  and the
/// decoder-stack id derivation.
pub use crate::policy::{CacheIdentity, Policy, default_policy, stack_id};

/// Re-exports the file-derived probe inputs (provenance and bones in one call).
pub use crate::inputs::{probe_inputs_from_bytes, probe_inputs_from_file};

/// Re-exports the per-provenance probe dials and the provenance signals.
pub use crate::bucketpolicy::{BucketProbe, BucketTable, TrackProvenance};

/// Re-exports the lossless bones profiling helpers and their error.
pub use crate::bones::{BonesError, bones_hot_bins, flac_bones_profile};

/// Re-exports the gain decision and its kind.
pub use crate::decision::{Decision, DecisionKind, PeakKind};

/// Re-exports the policy resolver,
///  its provenance-aware form,
///  and the always-exact
/// full-scan resolver (warming upgrade).
pub use crate::resolve::{resolve_decision, resolve_decision_for, resolve_full_scan};

/// Re-exports the track fingerprint function and its domain tag,
///  behind the `fingerprint` feature.
#[cfg(feature = "fingerprint")]
pub use crate::fingerprint::{FINGERPRINT_DOMAIN_TAG, track_fingerprint};

/// Re-exports the decision cache and its error,
///  behind the `service` feature.
#[cfg(feature = "service")]
pub use crate::cache::{CacheError, DecisionCache, WRITE_BATCH_LIMIT};

/// Re-exports the cache-aware resolve,
///  behind the `service` feature.
#[cfg(feature = "service")]
pub use crate::service::cached_or_resolve;

/// Registers the unit-test scratch root, placed in the build directory instead of `/tmp`.
#[cfg(test)]
mod test_scratch;
