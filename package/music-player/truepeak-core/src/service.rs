//! The cache-aware resolve that composes the cache and the resolver (service feature).
//!
//! This is the one call both the foreground current track and the background warming loop
//! make:
//!  return the cached decision,
//!  or open the source,
//!  resolve it under the policy,
//!  store
//! the result,
//!  and return it.
//!  The opener is invoked only on a miss,
//!  so a cache hit never
//! decodes.
//!  The platform still owns concurrency,
//!  priority,
//!  and cancellation;
//!  this is only the
//! per-track get-or-resolve,
//!  which the platform drives in whatever loop it likes.

/// What:
///      `use crate::cache::{CacheError, DecisionCache};`.
///  The cache handle and its error.
/// Why:
///       This reads and writes through the cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CacheError, DecisionCache } from "./cache";
/// ```
use crate::cache::{CacheError, DecisionCache};

/// What:
///      `use crate::decision::Decision;`.
///  The value returned.
/// Why:
///       Both a hit and a fresh resolve produce one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Decision } from "./decision";
/// ```
use crate::decision::Decision;

/// What:
///      `use crate::error::TruePeakError;`.
///  The opener and resolver error.
/// Why:
///       Mapped into a `CacheError` so the return type stays single.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TruePeakError } from "./error";
/// ```
use crate::error::TruePeakError;

/// What:
///      `use crate::policy::Policy;`.
///  The active policy.
/// Why:
///       It supplies the identity for the cache key and the resolve parameters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Policy } from "./policy";
/// ```
use crate::policy::Policy;

/// What:
///      `use crate::bucketpolicy::TrackProvenance;`.
///  The provenance signals that pick
///           a long track's bucket.
/// Why:
///       The caller supplies them;
///  the resolve on a miss is steered by them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TrackProvenance } from "./bucketpolicy";
/// ```
use crate::bucketpolicy::TrackProvenance;

/// What:
///      `use crate::resolve::resolve_decision_for;`.
///  The blocking measurement.
/// Why:
///       Called on a cache miss to produce the decision under the track's bucket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { resolveDecisionFor } from "./resolve";
/// ```
use crate::resolve::resolve_decision_for;

/// What:
///      `use crate::source::TruePeakSource;`.
///  The decoded-audio contract.
/// Why:
///       The opener returns a boxed one on a miss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TruePeakSource } from "./source";
/// ```
use crate::source::TruePeakSource;

/// What:
///      `pub async fn cached_or_resolve<Open>(cache: &DecisionCache, policy: &Policy,
///           fingerprint: u64, decoder_stack_id: u64, open: Open) -> Result<Decision,
///           CacheError> where Open: FnOnce() -> Result<Box<dyn TruePeakSource>,
///           TruePeakError>`.
///  Return the cached decision,
///  or open,
///  resolve,
///  store,
///  and
///           return.
///  `Open` is a one-shot closure (sibling `Fn`/`FnMut`) called only on a
///           miss;
///  `Box<dyn TruePeakSource>` (sibling `&mut dyn`) is an owned source the
///           closure hands over.
/// Why:
///       One entry point for the foreground track and the warming loop,
///  so both share
///           the cache,
///  the policy,
///  and the precedence rules.
/// Gotcha:
///    The resolve performs a BLOCKING full or probe scan,
///  so call this where blocking
///           decode is acceptable (a worker thread or a blocking-tolerant task);
///  the platform
///           owns that scheduling and any concurrency around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cachedOrResolve(cache, policy, fingerprint, decoderStackId, open): Promise<Decision> { ... }
/// ```
pub async fn cached_or_resolve<Open>(
    cache: &DecisionCache,
    policy: &Policy,
    fingerprint: u64,
    decoder_stack_id: u64,
    provenance: TrackProvenance,
    bones_hot_bins: Option<&[usize]>,
    open: Open,
) -> Result<Decision, CacheError>
where
    Open: FnOnce() -> Result<Box<dyn TruePeakSource>, TruePeakError>,
{
    // What:     `let identity = policy.cache_identity(decoder_stack_id);`. The four-part key.
    // Why:      Both the read and the write use it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const identity = policy.cacheIdentity(decoderStackId);
    // ```
    let identity = policy.cache_identity(decoder_stack_id);
    // What:     `if let Some(hit) = cache.get(fingerprint, identity).await? { return Ok(hit); }`.
    //           On a hit, return the stored decision without opening the source; `?`
    //           propagates a cache error.
    // Why:      A cache hit must never decode.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hit = await cache.get(fingerprint, identity); if (hit) return hit;
    // ```
    if let Some(hit) = cache.get(fingerprint, identity).await? {
        // Cache hit: return the stored decision without opening the source.
        tracing::debug!(fingerprint, "cache hit");
        return Ok(hit);
    }
    // Cache miss: open, resolve, and store below.
    tracing::debug!(fingerprint, "cache miss");
    // What:     `let mut source = open().inspect_err(|error| warn!(...)).map_err(|error|
    //           CacheError { message: error.to_string() })?;`. Open the source lazily;
    //           `inspect_err` logs the typed `TruePeakError` before `map_err` folds it into a
    //           `CacheError`, which flattens the variant to a string.
    // Why:      We only decode on a miss, and the fold would otherwise discard the cause.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = open();
    // ```
    let mut source = open()
        .inspect_err(|error| tracing::warn!(cause = %error, "open failed on cache miss"))
        .map_err(|error| return CacheError { message: error.to_string() })?;
    // What:     `let decision = resolve_decision_for(policy, source.as_mut(), provenance,
    //           bones_hot_bins).inspect_err(|error| warn!(...)).map_err(...)?;`. Drive the
    //           source through the policy under the track's bucket; `.as_mut()` lends the
    //           boxed source as `&mut dyn TruePeakSource`. `inspect_err` logs the typed error
    //           before `map_err` folds it into a `CacheError`.
    // Why:      Produce the fresh decision to store and return, without discarding the cause.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const decision = resolveDecision(policy, source);
    // ```
    let decision = resolve_decision_for(policy, source.as_mut(), provenance, bones_hot_bins)
        .inspect_err(|error| tracing::warn!(cause = %error, "resolve failed on cache miss"))
        .map_err(|error| return CacheError { message: error.to_string() })?;
    // What:     `cache.put(fingerprint, identity, &decision).await?;`. Persist it; precedence
    //           in the cache keeps an exact row from being downgraded.
    // Why:      Later plays hit the cache.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await cache.put(fingerprint, identity, decision);
    // ```
    cache.put(fingerprint, identity, &decision).await?;
    // The fresh decision is stored; log what was resolved (its whole Debug form).
    tracing::debug!(decision = ?decision, "resolved and stored");
    // What:     `Ok(decision)`. The fresh decision. Tail -> return.
    // Why:      Hand it to the caller that asked to resolve it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return decision;
    // ```
    return Ok(decision)
}

/// What:
///      `#[cfg(test)] #[path = "service_tests.rs"] mod tests;`.
///  Test-only submodule in
///           the sibling file,
///  gated to test builds of the `service` feature.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // service.integration.test.ts
/// ```
#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
