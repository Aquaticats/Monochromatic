//! Integration test for the cache-aware resolve,
//!  against a throwaway in-memory database.

use super::*;
use crate::bucketpolicy::TrackProvenance;
use crate::decision::DecisionKind;
use crate::policy::Policy;
use crate::source::AudioSpec;
use std::cell::Cell;

// A minimal in-memory source; short buffers full-scan, so no seeking is exercised here.
struct Fake {
    samples: Vec<f32>,
    cursor: usize,
}

impl TruePeakSource for Fake {
    fn spec(&self) -> AudioSpec {
        return AudioSpec { rate: 8, channels: 1, duration_secs: self.samples.len() as f64 / 8.0 }
    }

    fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError> {
        if self.cursor >= self.samples.len() {
            return Ok(Vec::new());
        }
        let end = (self.cursor + 4).min(self.samples.len());
        let block = self.samples[self.cursor..end].to_vec();
        self.cursor = end;
        return Ok(block)
    }

    fn seek_to_frame(&mut self, frame: u64) -> Result<(), TruePeakError> {
        self.cursor = frame as usize;
        return Ok(())
    }
}

fn test_policy() -> Policy {
    // These tests only exercise short-track full scans, so the shipped default's bucket
    // table serves unchanged; only the short cutoff is widened.
    let mut policy = crate::policy::default_policy();
    policy.short_scan_max_secs = 100.0;
    return policy
}

// The first resolve opens the source and stores; the second is a cache hit and never opens.
#[tokio::test]
async fn resolves_on_miss_then_serves_from_cache() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    let policy = test_policy();
    let opens = Cell::new(0u32);

    let first = cached_or_resolve(&cache, &policy, 7, 100, TrackProvenance::unknown(), None, || {
        opens.set(opens.get() + 1);
        return Ok(Box::new(Fake { samples: vec![0.0, 0.9, 0.9, 0.0], cursor: 0 }) as Box<dyn TruePeakSource>)
    })
    .await
    .expect("cache miss should resolve");
    assert_eq!(first.kind, DecisionKind::ShortFullScan);
    assert_eq!(opens.get(), 1); // opened on the miss

    let second = cached_or_resolve(&cache, &policy, 7, 100, TrackProvenance::unknown(), None, || {
        opens.set(opens.get() + 1);
        return Ok(Box::new(Fake { samples: vec![0.0, 0.9, 0.9, 0.0], cursor: 0 }) as Box<dyn TruePeakSource>)
    })
    .await
    .expect("cache hit should resolve");
    assert_eq!(second, first); // same decision, read back from the cache
    assert_eq!(opens.get(), 1); // the hit never opened the source
}
