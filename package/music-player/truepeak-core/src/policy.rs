//! The versioned policy and the identity tuple that keys cache rows.
//!
//! The app ships ONE active policy.
//!  A cached decision is reusable only when the full
//! identity matches:
//!  the `policy_id` (constants,
//!  gain math,
//!  cache
//! interpretation),
//!  the `meter_id` (Catmull-Rom behavior including the chunk-seam and
//! end-of-track rules),
//!  the `decoder_stack_id` (the platform's Symphonia and libopus
//! behavior,
//!  supplied by the platform),
//!  and the `schema_version` (row layout).
//!  Keeping
//! these as separate values,
//!  not collapsed into `policy_id`,
//!  means a decoder bump does
//! not needlessly churn unrelated rows.
//!  The `policy_id` is DERIVED from the policy
//! parameters,
//!  so changing a constant cannot silently reuse a stale cache row.

/// What:
///      `use crate::bucketpolicy::{BucketProbe, BucketTable};`.
///  The per-provenance
///           probe dial and the table of them.
/// Why:
///       The policy carries the allocation layer,
///  and its id must hash every dial.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BucketProbe, BucketTable } from "./bucketpolicy";
/// ```
use crate::bucketpolicy::{BucketProbe, BucketTable};

/// What:
///      `const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;`.
///  The 64-bit FNV-1a offset
///           basis (its standard starting value).
///  `u64` (siblings `u32`/`u128`) is the
///           width of this hash variant.
///  The `_` digit separators are ignored.
/// Why:
///       The seed for the small dependency-free hash that derives the identity ids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FNV_OFFSET = 0xcbf29ce484222325n; // bigint
/// ```
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// What:
///      `const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;`.
///  The 64-bit FNV-1a prime
///           multiplier.
///  `u64` to match `FNV_OFFSET`.
/// Why:
///       The per-byte multiply that spreads input bits across the hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FNV_PRIME = 0x100000001b3n; // bigint
/// ```
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// What:
///      `pub const SCHEMA_VERSION: u32 = 2;`.
///  The cache key-and-row version.
///  `u32`
///           (sibling `u64` overkill) is plenty for a slowly-changing schema counter.
/// Why:
///       A read is a hit only when the stored version matches;
///  bump this when the row columns
///           change OR when the fingerprint that fills the key column changes.
///  Version 2 is the
///           switch from the gxhash `u64` fingerprint to XXH3-64
///           (`doc/decision/music-player-fingerprint-hash.md`):
///  the column keeps its type but
///           holds different values,
///  so version 1 rows can never match again.
///  `DecisionCache::open`
///           deletes every row below this version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SCHEMA_VERSION = 2;
/// ```
pub const SCHEMA_VERSION: u32 = 2;

/// What:
///      `const METER_DESCRIPTION: &str = "...";`.
///  A string naming every behavior
///           that defines the shared meter's numeric output.
///  `&str` (sibling `String`
///           would needlessly own) is a literal baked into the binary.
/// Why:
///       `meter_id` is the hash of this string,
///  so any change to meter behavior must
///           edit this description,
///  which bumps `meter_id` and invalidates stale rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const METER_DESCRIPTION = "catmull-rom ...";
/// ```
const METER_DESCRIPTION: &str =
    "catmull-rom q/h/tq; 4-sample window; per-channel cursor across chunk seams; interpolate only at 4 real samples; no synthetic end padding; v1";

/// What:
///      `const SHORT_SCAN_MAX_SECS: f64 = 90.0;`.
///  Tracks at or below this length are
///           scanned in full for an exact peak;
///  longer tracks are probed.
///  `f64` (sibling
///           `f32`) to compare against the source duration.
/// Why:
///       Short tracks are cheap to scan exactly,
///  so they never carry probe error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHORT_SCAN_MAX_SECS = 90;
/// ```
const SHORT_SCAN_MAX_SECS: f64 = 90.0;

/// What:
///      `const PROBE_WINDOW_SECS: f64 = 0.1;`.
///  The length of one probe bin,
///  the
///           zoom's measurement unit.
///  `f64` (sibling `f32`) for precision.
/// Why:
///       Tenth-second bins maximize distinct regions per decoded second and match
///           the corpus evidence the policy was fitted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PROBE_WINDOW_SECS = 0.1;
/// ```
const PROBE_WINDOW_SECS: f64 = 0.1;

/// What:
///      `const PASS1_COVERAGE_FRACTION: f64 = 0.1;`.
///  The even pass's share of a long
///           track's bins before the climb spends the rest.
///  `f64` (sibling `f32`).
/// Why:
///       A tenth was the measured sweet spot:
///  denser even passes starve the climb,
///           sparser ones miss whole loud passages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PASS1_COVERAGE_FRACTION = 0.1;
/// ```
const PASS1_COVERAGE_FRACTION: f64 = 0.1;

/// What:
///      `const BONES_EVEN_COVERAGE_FRACTION: f64 = 0.05;`.
///  The even pass used when
///           bones seeds already cover the hot slots.
///  `f64` (sibling `f32`).
/// Why:
///       Bones point the climb at the loud passages,
///  so the safety even pass can be
///           half as dense inside the lossless-bones bucket's smaller budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BONES_EVEN_COVERAGE_FRACTION = 0.05;
/// ```
const BONES_EVEN_COVERAGE_FRACTION: f64 = 0.05;

/// What:
///      `const BONES_TOP_SLOTS: usize = 40;`.
///  How many byte-rate hot slots seed the
///           lossless probe.
///  `usize` (sibling `u32`) to index slot vectors directly.
/// Why:
///       Forty seeds with neighbors cost about one percent coverage and start the
///           climb on the right hills;
///  more seeds measured no better on the corpus.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BONES_TOP_SLOTS = 40;
/// ```
const BONES_TOP_SLOTS: usize = 40;

/// What:
///      `const LOSSLESS_PROBE: BucketProbe = ...;`.
///  The lossless bucket without
///           bones:
///  a tenth coverage with a 0.45 dB margin.
/// Why:
///       Lossless tails are thin;
///  a tenth of the bins already reads them tightly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOSSLESS_PROBE = { coverageFraction: 0.10, probeMarginDb: 0.45 };
/// ```
const LOSSLESS_PROBE: BucketProbe = BucketProbe { coverage_fraction: 0.10, probe_margin_db: 0.45 };

/// What:
///      `const LOSSLESS_BONES_PROBE: BucketProbe = ...;`.
///  The lossless bucket with
///           frame-size bones seeds:
///  seven hundredths coverage at the same margin.
/// Why:
///       Bones locate the loud passages,
///  so the same accuracy costs a third less.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOSSLESS_BONES_PROBE = { coverageFraction: 0.07, probeMarginDb: 0.45 };
/// ```
const LOSSLESS_BONES_PROBE: BucketProbe =
    BucketProbe { coverage_fraction: 0.07, probe_margin_db: 0.45 };

/// What:
///      `const STORE_PROBE: BucketProbe = ...;`.
///  Store-tagged lossy tracks:
///  high
///           coverage with a 0.30 dB margin.
/// Why:
///       Mastered releases probe cleanly,
///  so the deep probe buys a small margin and
///           the loudest playback of any bucket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STORE_PROBE = { coverageFraction: 0.32, probeMarginDb: 0.30 };
/// ```
const STORE_PROBE: BucketProbe = BucketProbe { coverage_fraction: 0.32, probe_margin_db: 0.30 };

/// What:
///      `const YOUTUBE_PROBE: BucketProbe = ...;`.
///  Youtube-provenance lossy tracks:
///           light coverage with the standard 0.50 dB margin.
/// Why:
///       Loudness-normalized sources hide few surprises;
///  light coverage suffices.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const YOUTUBE_PROBE = { coverageFraction: 0.14, probeMarginDb: 0.50 };
/// ```
const YOUTUBE_PROBE: BucketProbe = BucketProbe { coverage_fraction: 0.14, probe_margin_db: 0.50 };

/// What:
///      `const BARE_PROBE: BucketProbe = ...;`.
///  Untagged lossy tracks:
///  the deepest
///           coverage with the 0.50 dB margin.
/// Why:
///       Every measured clamp-tail track is untagged lossy;
///  the coverage freed from
///           the other buckets is spent exactly here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BARE_PROBE = { coverageFraction: 0.34, probeMarginDb: 0.50 };
/// ```
const BARE_PROBE: BucketProbe = BucketProbe { coverage_fraction: 0.34, probe_margin_db: 0.50 };

/// What:
///      `const CEILING_DBTP: f64 = -1.0;`.
///  The normalization ceiling in dBTP.
///           `-1.0` is inside the always-allowed `-2..=2` range.
///  `f64` (sibling `f32`)
///           for dB math.
/// Why:
///       Hashed into `policy_id` so a ceiling change re-keys the cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CEILING_DBTP = -1;
/// ```
const CEILING_DBTP: f64 = -1.0;

/// What:
///      `const MAX_TOO_LOUD_DB: f64 = 1.0 / 2.0;`.
///  The `+0.5 dB` too-loud bound,
///           composed from the exempt `-2..=2` range rather than a bare `0.5`.
///  `f64`
///           (sibling `f32`) for dB math.
/// Why:
///       The hard upper error bound;
///  hashed into `policy_id`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_TOO_LOUD_DB = 1 / 2;
/// ```
const MAX_TOO_LOUD_DB: f64 = 1.0 / 2.0;

/// What:
///      `const MAX_TOO_QUIET_DB: f64 = -2.0;`.
///  The `-2.0 dB` too-quiet bound.
///           `-2.0` is the edge of the always-allowed `-2..=2` range.
///  `f64` (sibling
///           `f32`) for dB math.
/// Why:
///       The hard lower error bound;
///  hashed into `policy_id`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_TOO_QUIET_DB = -2;
/// ```
const MAX_TOO_QUIET_DB: f64 = -2.0;

/// What:
///      `fn mix(hash: u64, word: u64) -> u64`.
///  Fold one 64-bit word into a running
///           FNV-1a hash,
///  one little-endian byte at a time.
/// Why:
///       One reusable step so both the word hash and the string hash share the same
///           byte mixing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mix(hash: bigint, word: bigint): bigint { ... }
/// ```
fn mix(hash: u64, word: u64) -> u64 {
    // What:     `word.to_le_bytes().iter().fold(hash, |accumulator, &byte| ...)`.
    //           `.to_le_bytes()` turns the `u64` into its 8 little-endian bytes;
    //           `.iter()` borrows each; `&byte` copies it out; `.fold(seed, closure)`
    //           threads the running hash. `byte as u64` widens for the xor. Tail.
    // Why:      FNV-1a is xor-then-multiply per byte; little-endian fixes the order so
    //           the hash is identical on every platform.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let h = hash;
    // for (const byte of toLeBytes(word)) h = (h ^ BigInt(byte)) * FNV_PRIME;
    // return h;
    // ```
    return word.to_le_bytes()
        .iter()
        .fold(hash, |accumulator, &byte| {
            return (accumulator ^ byte as u64).wrapping_mul(FNV_PRIME)
        })
}

/// What:
///      `fn hash_words(words: &[u64]) -> u64`.
///  FNV-1a over a slice of 64-bit words.
///           `&[u64]` (sibling `Vec<u64>` would own) borrows the caller's array.
/// Why:
///       Derive a stable id from a fixed list of policy parameters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hashWords(words: bigint[]): bigint { ... }
/// ```
fn hash_words(words: &[u64]) -> u64 {
    // What:     `words.iter().fold(FNV_OFFSET, |hash, &word| mix(hash, word))`. Start
    //           from the offset basis and fold each word through `mix`. Tail -> return.
    // Why:      Order-sensitive hash of all parameters into one id.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return words.reduce((hash, word) => mix(hash, word), FNV_OFFSET);
    // ```
    return words.iter().fold(FNV_OFFSET, |hash, &word| return mix(hash, word))
}

/// What:
///      `fn hash_bytes(bytes: &[u8]) -> u64`.
///  FNV-1a over raw bytes.
///  `&[u8]`
///           (sibling `&str` is text-only) borrows the caller's bytes.
/// Why:
///       Hash the meter description string into `meter_id`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hashBytes(bytes: Uint8Array): bigint { ... }
/// ```
fn hash_bytes(bytes: &[u8]) -> u64 {
    // What:     `bytes.iter().fold(FNV_OFFSET, |hash, &byte| (hash ^ byte as u64)
    //           .wrapping_mul(FNV_PRIME))`. The same xor-then-multiply per byte, here
    //           directly over bytes. `.wrapping_mul` multiplies modulo 2^64 instead of
    //           panicking on overflow. Tail -> return.
    // Why:      Stable, dependency-free string hash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return bytes.reduce((hash, byte) => (hash ^ BigInt(byte)) * FNV_PRIME, FNV_OFFSET);
    // ```
    return bytes
        .iter()
        .fold(FNV_OFFSET, |hash, &byte| {
            return (hash ^ byte as u64).wrapping_mul(FNV_PRIME)
        })
}

/// What:
///      `pub fn meter_id() -> u64`.
///  The id of the shared meter's behavior,
///  the hash
///           of `METER_DESCRIPTION`.
/// Why:
///       A cache row is reusable only if it was produced by this exact meter;
///  the id
///           changes whenever the description (and thus the behavior) changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function meterId(): bigint { return hashBytes(utf8(METER_DESCRIPTION)); }
/// ```
pub fn meter_id() -> u64 {
    // What:     `hash_bytes(METER_DESCRIPTION.as_bytes())`. `.as_bytes()` views the
    //           string as its UTF-8 bytes without copying. Tail -> return.
    // Why:      Hash the behavior description into a stable id.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return hashBytes(utf8(METER_DESCRIPTION));
    // ```
    return hash_bytes(METER_DESCRIPTION.as_bytes())
}

/// What:
///      `pub fn stack_id(description: &str) -> u64`.
///  The id of a platform's decoder
///           stack,
///  the hash of its behavior description.
/// Why:
///       Both platforms key their cache identity on a description string exactly the
///           way `meter_id` does;
///  owning the derivation here removes each platform's
///           private hash plumbing and keeps every identity id on one hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stackId(description: string): bigint { return hashBytes(utf8(description)); }
/// ```
pub fn stack_id(description: &str) -> u64 {
    // What:     `hash_bytes(description.as_bytes())`. Tail -> return.
    // Why:      The same dependency-free FNV the other identity ids use.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return hashBytes(utf8(description));
    // ```
    return hash_bytes(description.as_bytes())
}

/// What:
///      `#[derive(Clone, Copy, Debug, PartialEq)] pub struct Policy { ... }`.
///  The
///           shipped policy's tunable parameters.
///  The derives give value copy,
///  debug
///           printing,
///  and equality.
///  All fields are `Copy`,
///  so the whole struct is.
/// Why:
///       One place holding the constants that define normalization behavior and that
///           feed the `policy_id` hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Policy = { shortScanMaxSecs; coverageFraction; probeWindowSecs; probeMarginDb; ceilingDbtp; maxTooLoudDb; maxTooQuietDb };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Policy {
    /// What:
    ///      `pub short_scan_max_secs: f64`.
    ///  Tracks at or below this length are scanned
    ///           in full;
    ///  longer tracks are probed.
    ///  `f64` (sibling `f32`) for the duration
    ///           compare.
    /// Why:
    ///       Short tracks are cheap to scan exactly,
    ///  so they carry no probe error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// shortScanMaxSecs: number;
    /// ```
    pub short_scan_max_secs: f64,
    /// What:
    ///      `pub probe_window_secs: f64`.
    ///  Length of one probe bin,
    ///  the zoom's
    ///           measurement unit.
    ///  `f64` (sibling `f32`) for precision.
    /// Why:
    ///       The bin grid every probe phase (bones,
    ///  even pass,
    ///  climb) measures on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// probeWindowSecs: number;
    /// ```
    pub probe_window_secs: f64,
    /// What:
    ///      `pub pass1_coverage_fraction: f64`.
    ///  The even pass's share of a long
    ///           track's bins before the climb spends the rest.
    ///  `f64` (sibling `f32`).
    /// Why:
    ///       Balances discovering distinct regions against climb depth.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pass1CoverageFraction: number;
    /// ```
    pub pass1_coverage_fraction: f64,
    /// What:
    ///      `pub bones_even_coverage_fraction: f64`.
    ///  The even pass used when bones
    ///           seeds already cover the hot slots.
    ///  `f64` (sibling `f32`).
    /// Why:
    ///       Bones-seeded probes need only a light safety net under the seeds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bonesEvenCoverageFraction: number;
    /// ```
    pub bones_even_coverage_fraction: f64,
    /// What:
    ///      `pub bones_top_slots: usize`.
    ///  How many byte-rate hot slots seed a
    ///           lossless probe.
    ///  `usize` (sibling `u32`) to index slot vectors.
    /// Why:
    ///       The bones budget;
    ///  hashed into `policy_id` like every dial.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bonesTopSlots: number;
    /// ```
    pub bones_top_slots: usize,
    /// What:
    ///      `pub buckets: BucketTable`.
    ///  The per-provenance coverage/margin table.
    /// Why:
    ///       The allocation layer:
    ///  lossless coverage is nearly free to cut,
    ///  and the
    ///           untagged lossy bucket receives what the cut frees.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// buckets: BucketTable;
    /// ```
    pub buckets: BucketTable,
    /// What:
    ///      `pub ceiling_dbtp: f64`.
    ///  The normalization ceiling in dBTP.
    /// Why:
    ///       Part of the gain math;
    ///  a change must re-key the cache.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ceilingDbtp: number;
    /// ```
    pub ceiling_dbtp: f64,
    /// What:
    ///      `pub max_too_loud_db: f64`.
    ///  The `+0.5 dB` too-loud error bound.
    /// Why:
    ///       Hard acceptance bound checked by the corpus verifier.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// maxTooLoudDb: number;
    /// ```
    pub max_too_loud_db: f64,
    /// What:
    ///      `pub max_too_quiet_db: f64`.
    ///  The `-2.0 dB` too-quiet error bound.
    /// Why:
    ///       Hard acceptance bound checked by the corpus verifier.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// maxTooQuietDb: number;
    /// ```
    pub max_too_quiet_db: f64,
}

/// What:
///      `impl Policy { ... }`.
///  The policy's behavior:
///  derive its `policy_id` and
///           bundle the full cache identity.
/// Why:
///       Keep the id derivation next to the fields it hashes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // methods on the Policy shape
/// ```
impl Policy {
    /// What:
    ///      `pub fn policy_id(&self) -> u64`.
    ///  The hash of every tunable parameter,
    ///           so two policies share an id only when all parameters match.
    ///  `&self`
    ///           borrows read-only.
    /// Why:
    ///       Deriving the id from the parameters makes a stale-cache reuse bug
    ///           impossible rather than merely discouraged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// policyId(): bigint { return hashWords([...]); }
    /// ```
    pub fn policy_id(&self) -> u64 {
        // What:     `let words: [u64; 7] = [ ... ];`. A fixed array of the parameters
        //           as 64-bit words. `.to_bits()` reinterprets an `f64` as the `u64`
        //           with the same bit pattern, so floats hash exactly.
        // Why:      A canonical, order-fixed encoding of all parameters to hash.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const words = [f64ToBits(shortScanMaxSecs), f64ToBits(coverageFraction), ...];
        // ```
        let words: [u64; 18] = [
            self.short_scan_max_secs.to_bits(),
            self.probe_window_secs.to_bits(),
            self.pass1_coverage_fraction.to_bits(),
            self.bones_even_coverage_fraction.to_bits(),
            self.bones_top_slots as u64,
            self.buckets.lossless.coverage_fraction.to_bits(),
            self.buckets.lossless.probe_margin_db.to_bits(),
            self.buckets.lossless_bones.coverage_fraction.to_bits(),
            self.buckets.lossless_bones.probe_margin_db.to_bits(),
            self.buckets.store.coverage_fraction.to_bits(),
            self.buckets.store.probe_margin_db.to_bits(),
            self.buckets.youtube.coverage_fraction.to_bits(),
            self.buckets.youtube.probe_margin_db.to_bits(),
            self.buckets.bare.coverage_fraction.to_bits(),
            self.buckets.bare.probe_margin_db.to_bits(),
            self.ceiling_dbtp.to_bits(),
            self.max_too_loud_db.to_bits(),
            self.max_too_quiet_db.to_bits(),
        ];
        // What:     `hash_words(&words)`. `&words` lends the array as a slice. Tail.
        // Why:      Fold all parameters into one id.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return hashWords(words);
        // ```
        return hash_words(&words)
    }

    /// What:
    ///      `pub fn cache_identity(&self, decoder_stack_id: u64) -> CacheIdentity`.
    ///           Bundle the four-part identity a cache row must match.
    ///  The platform
    ///           supplies `decoder_stack_id` (its Symphonia and libopus behavior).
    /// Why:
    ///       A read is a hit only when all four parts match;
    ///  this assembles them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cacheIdentity(decoderStackId): CacheIdentity { ... }
    /// ```
    pub fn cache_identity(&self, decoder_stack_id: u64) -> CacheIdentity {
        // What:     `CacheIdentity { ... }`. The struct literal pairing this policy's id
        //           and meter id with the platform decoder id and the schema version.
        //           Tail -> return.
        // Why:      Hand back the full identity tuple for the cache key.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { policyId: this.policyId(), meterId: meterId(), decoderStackId, schemaVersion: SCHEMA_VERSION };
        // ```
        return CacheIdentity {
            policy_id: self.policy_id(),
            meter_id: meter_id(),
            decoder_stack_id,
            schema_version: SCHEMA_VERSION,
        }
    }
}

/// What:
///      `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct CacheIdentity
///           { ... }`.
///  The four values a cache row must match to be reused.
///  `Eq` (the
///           total-equality marker,
///  valid because every field is an integer) lets it be
///           used as a map key later.
/// Why:
///       Keep decoder identity separate from policy identity so a decoder bump does
///           not churn unrelated rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CacheIdentity = { policyId; meterId; decoderStackId; schemaVersion };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CacheIdentity {
    /// What:
    ///      `pub policy_id: u64`.
    ///  Hash of the policy parameters.
    /// Why:
    ///       Constants,
    ///  gain math,
    ///  and cache interpretation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// policyId: bigint;
    /// ```
    pub policy_id: u64,
    /// What:
    ///      `pub meter_id: u64`.
    ///  Hash of the meter behavior description.
    /// Why:
    ///       Catmull-Rom behavior including chunk-seam and end-of-track rules.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// meterId: bigint;
    /// ```
    pub meter_id: u64,
    /// What:
    ///      `pub decoder_stack_id: u64`.
    ///  Platform-supplied decoder behavior id.
    /// Why:
    ///       Symphonia and libopus versions and their conversion behavior.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// decoderStackId: bigint;
    /// ```
    pub decoder_stack_id: u64,
    /// What:
    ///      `pub schema_version: u32`.
    ///  The row-layout version.
    /// Why:
    ///       A read is a hit only when the stored layout matches.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// schemaVersion: number;
    /// ```
    pub schema_version: u32,
}

/// What:
///      `pub fn default_policy() -> Policy`.
///  The one active policy the app ships,
///           built from the provisional starting constants above.
/// Why:
///       A single entry point for the shipped policy;
///  Stage two replaces the
///           starting constants with the searched values,
///  which re-keys the cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function defaultPolicy(): Policy { return { ... }; }
/// ```
pub fn default_policy() -> Policy {
    // What:     `Policy { ... }`. The struct literal assigning each named starting
    //           constant to its field. Tail -> return.
    // Why:      Hand back the shipped policy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { shortScanMaxSecs: SHORT_SCAN_MAX_SECS, ... };
    // ```
    return Policy {
        short_scan_max_secs: SHORT_SCAN_MAX_SECS,
        probe_window_secs: PROBE_WINDOW_SECS,
        pass1_coverage_fraction: PASS1_COVERAGE_FRACTION,
        bones_even_coverage_fraction: BONES_EVEN_COVERAGE_FRACTION,
        bones_top_slots: BONES_TOP_SLOTS,
        buckets: BucketTable {
            lossless: LOSSLESS_PROBE,
            lossless_bones: LOSSLESS_BONES_PROBE,
            store: STORE_PROBE,
            youtube: YOUTUBE_PROBE,
            bare: BARE_PROBE,
        },
        ceiling_dbtp: CEILING_DBTP,
        max_too_loud_db: MAX_TOO_LOUD_DB,
        max_too_quiet_db: MAX_TOO_QUIET_DB,
    }
}

/// What:
///      `#[cfg(test)] #[path = "policy_tests.rs"] mod tests;`.
///  Test-only submodule
///           in the sibling file `policy_tests.rs`,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // policy.unit.test.ts
/// ```
#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
