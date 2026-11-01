//! Per-provenance probe buckets:
//!  the allocation layer of the shipped policy.
//!
//! Buckets come from embedded metadata only (codec,
//!  store identifiers,
//!  iTunNORM,
//! youtube provenance;
//!  never path text).
//!  Measured on the corpus their probe tails
//! diverge hard:
//!  lossless tracks keep their accuracy at a fraction of the coverage
//! (and at an even smaller fraction when frame-size bones seed the probe),
//!  while the
//! untagged lossy bucket carries all the risk and receives the freed coverage.

/// One bucket's probe dial:
///  how much of a long track to measure and the margin to add.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BucketProbe {
    /// Fraction of the track's probe bins the zoom may measure.
    pub coverage_fraction: f64,
    /// Fixed margin added to the probe's sampled peak in dB.
    pub probe_margin_db: f64,
}

/// The per-provenance probe table the policy ships.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BucketTable {
    /// Lossless tracks probed without bones seeds.
    pub lossless: BucketProbe,
    /// Lossless tracks probed with frame-size bones seeds (cheaper at equal accuracy).
    pub lossless_bones: BucketProbe,
    /// Lossy tracks carrying store identifiers (ISRC/UPC/content ids/iTunNORM).
    pub store: BucketProbe,
    /// Lossy tracks carrying youtube provenance (loudness-normalized sources).
    pub youtube: BucketProbe,
    /// Untagged lossy tracks,
    ///  the risk bucket that receives the freed coverage.
    pub bare: BucketProbe,
}

/// A track's zero-cost provenance signals,
///  supplied by the platform's decoder and tags.
///
/// Every flag false is the safe default:
///  the track lands in the bare bucket,
///  which has
/// the most coverage,
///  so an uninformed caller never under-probes anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrackProvenance {
    /// Whether the codec is lossless (FLAC and kin);
    ///  decoders know this for free.
    pub lossless: bool,
    /// Whether store identifiers are embedded (ISRC/UPC/content ids/iTunNORM).
    pub store_tagged: bool,
    /// Whether a youtube provenance tag is embedded (purl or a youtube comment).
    pub youtube_tagged: bool,
}

/// What:
///      `impl TrackProvenance { ... }`.
///  Bucket selection from the signals.
/// Why:
///       Keep the mapping next to the flags it reads.
impl TrackProvenance {
    /// The uninformed provenance:
    ///  every signal false,
    ///  landing in the bare bucket.
    ///
    /// @example `resolve_decision` uses this for callers that pass no provenance.
    pub fn unknown() -> TrackProvenance {
        return TrackProvenance::default()
    }

    /// Select the probe dial for this provenance from `table`.
    ///
    /// What:
    ///  lossless picks the bones dial when bones seeds exist;
    ///  store beats youtube;
    /// everything else is bare.
    ///  Why:
    ///  the priority mirrors how reliable each signal is.
    pub fn select(&self, table: &BucketTable, bones_present: bool) -> BucketProbe {
        if self.lossless {
            if bones_present { return table.lossless_bones } else { return table.lossless }
        } else if self.store_tagged {
            return table.store
        } else if self.youtube_tagged {
            return table.youtube
        } else {
            return table.bare
        }
    }
}
